use std::str::FromStr;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, runtime};
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator, Sampler, Tracer};
use opentelemetry_semantic_conventions::resource::{DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use tonic::metadata::{Ascii, MetadataValue};
use tonic::Request;
use tracing::span;
use tracing_core::Level;
use tracing_core::span::Id;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::gitstafette_info::GetInfoRequest;

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
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_endpoint("http://localhost:4317"))
        .install_batch(runtime::Tokio)
        .unwrap()

}

// Initialize tracing-subscriber and return OtelGuard for opentelemetry-related termination processing
pub fn init_tracing_subscriber(service_name_suffix: String) -> OtelGuard {
    // let meter_provider = init_meter_provider();

    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            Level::INFO,
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(init_tracer(service_name_suffix)))
        .init();

    OtelGuard {  }
}

pub struct OtelGuard {

}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}
