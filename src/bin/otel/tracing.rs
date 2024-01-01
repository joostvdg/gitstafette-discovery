use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{ WithExportConfig};
use opentelemetry_sdk::{Resource, runtime};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator, Sampler, Tracer};
use opentelemetry_semantic_conventions::resource::{DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION};
use opentelemetry_semantic_conventions::SCHEMA_URL;

// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource(service_name_suffix: String) -> Resource {

    let pkg_name: &'static str = env!("CARGO_PKG_NAME");
    let service_name = format!("{}-{}", pkg_name, service_name_suffix);
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, service_name),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT, "develop"),
        ],
        SCHEMA_URL,
    )
}

fn init_tracer(service_name_suffix: String) -> Tracer {
    let trace_exporter = opentelemetry_otlp::new_exporter().tonic().with_endpoint("http://localhost:4317");

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                // Customize sampling strategy
                .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                    1.0,
                ))))
                // If export trace to AWS X-Ray, you can use XrayIdGenerator
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource(service_name_suffix)),
        )
        .with_batch_config(BatchConfig::default())
        .with_exporter(trace_exporter)
        .install_batch(runtime::Tokio)
        .unwrap()

}

// Initialize tracing-subscriber and return OtelGuard for opentelemetry-related termination processing
pub fn init_tracing_subscriber(service_name_suffix: String)  {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = init_tracer(service_name_suffix);
    global::set_tracer_provider(tracer.provider().unwrap());
}

