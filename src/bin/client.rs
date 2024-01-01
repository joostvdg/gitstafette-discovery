use std::error::Error;
use autometrics::{autometrics, prometheus_exporter};
use clap::{Parser, Subcommand};
use opentelemetry::{global, propagation::Injector};
use opentelemetry::{
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};

use tonic::transport::Channel;

use gitstafette_discovery::{
    discovery_client::DiscoveryClient, GetHubsRequest, GitstafetteHub, RegisterHubRequest,GitstafetteServer, GetServersRequest, RegisterServerRequest
};

use gitstafette_info:: {
    info_client::InfoClient, GetInfoRequest, InstanceType
};

// https://timvw.be/2022/04/28/notes-on-using-grpc-with-rust-and-tonic/
#[allow(clippy::derive_partial_eq_without_eq)] // tonic don't derive Eq for generated types. We shouldn't manually change it.
#[path = "gitstafette_discovery.rs"]
pub mod gitstafette_discovery;

#[allow(clippy::derive_partial_eq_without_eq)] // tonic don't derive Eq for generated types. We shouldn't manually change it.
#[path = "gitstafette_info.rs"]
pub mod gitstafette_info;

mod otel;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Gitstatfette Discovery Server Hostname
    #[arg(long, default_value = "[::1]")]
    hostname: String,

    /// Gitstatfette Discovery Server Port
    #[arg(long, default_value = "50051")]
    port: String,

    /// Gitstatfette Discovery Server Protocol
    /// (http or https)
    /// (default: http)
    #[arg(long, default_value = "http")]
    protocol: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// registers a Gitstafette hub
    RegisterHub {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        host: String,
        #[arg(long)]
        port: String,
        #[arg(long)]
        repositories: String,
        #[arg(long)]
        relay_host: String,
        #[arg(long)]
        relay_port: String,
    },
    // retrieves all registered Gitstafette Hubs
    GetHubs {
        #[arg(short, long, default_value = "true")]
        print: bool,
    },
    // retrieve all registered Gitstafette Servers
    GetServers,
    /// registers a Gitstafette Server
    RegisterServer {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        host: String,
        #[arg(long)]
        port: String,
        #[arg(long)]
        repositories: String,
    },

    /// loops asking a local Gistafette Info server and registers it to the Discovery Server
    InfoRegistrationLoop {
        #[arg(long)]
        info_host: String,
        #[arg(long)]
        info_port: String,
        #[arg(long)]
        info_protocol: String,
    },
}

#[autometrics]
async fn parse_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();


    let server = format!("{}://{}:{}", cli.protocol, cli.hostname, cli.port);

    // You can check the value provided by positional arguments, or option arguments
    println!("Discovery Server Address={server}");

    let mut discovery_client: DiscoveryClient<tonic::transport::Channel> = DiscoveryClient::connect(server).await?;

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::RegisterHub{id, name, version, host, port, repositories, relay_host, relay_port }) => {
            println!("registering hub: {}", *id);
            // create request
            let request= RegisterHubRequest {
                hub: Some(GitstafetteHub {
                    id: id.to_string(),
                    name: name.to_string(),
                    version: version.to_string(),
                    host: host.to_string(),
                    port: port.to_string(),
                    repositories: repositories.to_string(),
                    relay_host: relay_host.to_string(),
                    relay_port: relay_port.to_string(),
                }),
            };
            register_hub(&mut discovery_client, request).await;
        }
        Some(Commands::GetHubs{print}) => {
            if *print {
                println!("retrieving hubs");
                get_hubs(&mut discovery_client).await;
            }
        }
        Some(Commands::GetServers) => {
            println!("retrieving servers");
            get_servers(&mut discovery_client).await;
        }
        Some(Commands::RegisterServer { id, name, version, host, port, repositories }   ) => {
            println!("registering server: {}", *id);
            // create request
            let request= RegisterServerRequest {
                server: Some(GitstafetteServer {
                    id: id.to_string(),
                    name: name.to_string(),
                    version: version.to_string(),
                    host: host.to_string(),
                    port: port.to_string(),
                    repositories: repositories.to_string(),
                }),
            };
            register_server(&mut discovery_client, request).await;
        }
        Some(Commands::InfoRegistrationLoop { info_host, info_port, info_protocol }) => {
            println!("starting info registration loop");
            sync_local_status_to_discovery_server(&mut discovery_client, info_host, info_port, info_protocol).await?;
        }
        None => {}
    }

    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

// #[autometrics]
async fn sync_local_status_to_discovery_server(mut discovery_client: &mut DiscoveryClient<Channel>, info_host: &String, info_port: &String, info_protocol: &String) -> Result<(), Box<dyn Error>> {
    prometheus_exporter::init();

    otel::tracing::init_tracing_subscriber("client".to_string());

    let server = format!("{}://{}:{}", info_protocol, info_host, info_port);
    let mut info_client: InfoClient<tonic::transport::Channel> = InfoClient::connect(server).await?;

    loop {
        let tracer = global::tracer("example/client");
        let span = tracer
            .span_builder(String::from("GSF-Discovery/client"))
            .with_kind(SpanKind::Client)
            .with_attributes(vec![KeyValue::new("component", "grpc")])
            .start(&tracer);
        let cx = Context::current_with_span(span);

        let info_request = GetInfoRequest { client_id: "myself".to_string(), client_endpoint: "127.0.0.1:50051".to_string() };
        let mut request: tonic::Request<GetInfoRequest> = tonic::Request::new(info_request);

        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut MetadataMap(request.metadata_mut()))
        });

        let response = info_client.get_info(request);
        let result = response.await;
        if result.is_err() {
            let status_code = result.unwrap_err().code();
            cx.span().add_event(
                "Got response!".to_string(),
                vec![KeyValue::new("status", status_code.to_string())],
            );
            continue;
        } else {
            cx.span().add_event(
                "Got response!".to_string(),
                vec![KeyValue::new("status", "OK".to_string())],
            );
        }

        let info = result.unwrap();

        // depending on the response, we should register the server/hub? to the Discovery Server
        if info.get_ref().alive {
            let server_info_opt = info.get_ref().server.as_ref();
            let relay_info_opt = info.get_ref().relay.as_ref();

            cx.span().add_event("local service is alive".to_string(), vec![]);

            if InstanceType::Hub == InstanceType::try_from(info.get_ref().instance_type).unwrap() {
                println!("registering hub: {}", info.get_ref().name);
                // create request

                let mut hub = GitstafetteHub {
                    id: "".to_string(),
                    name: info.get_ref().name.to_string(),
                    version: info.get_ref().version.to_string(),
                    host: "".to_string(),
                    port: "".to_string(),
                    repositories: "".to_string(),
                    relay_host: "".to_string(),
                    relay_port: "".to_string(),
                };

                if let Some(server_info) = server_info_opt {
                    hub.host = server_info.hostname.to_string();
                    hub.port = server_info.port.to_string();
                    if let Some(repositories) = server_info.repositories.as_ref() {
                        hub.repositories = repositories.to_string();
                    }
                }
                if let Some(relay_info) = relay_info_opt {
                    hub.relay_host = relay_info.hostname.to_string();
                    hub.relay_port = relay_info.port.to_string();
                }

                let request = RegisterHubRequest {
                    hub: Some(hub),
                };
                register_hub(&mut discovery_client, request).await;
                cx.span().add_event("registered hub".to_string(), vec![]);
            } else {
                println!("registering server: {}", info.get_ref().name);
                // create request
                let mut gsf_server = GitstafetteServer {
                    id: "".to_string(),
                    name: info.get_ref().name.to_string(),
                    version: info.get_ref().version.to_string(),
                    repositories: "".to_string(),
                    host: "".to_string(),
                    port: "".to_string(),
                };

                if let Some(server_info) = server_info_opt {
                    gsf_server.host = server_info.hostname.to_string();
                    gsf_server.port = server_info.port.to_string();
                    if let Some(repositories) = server_info.repositories.as_ref() {
                        gsf_server.repositories = repositories.to_string();
                    }
                }

                let request = RegisterServerRequest {
                    server: Some(gsf_server),
                };
                register_server(&mut discovery_client, request).await;
                cx.span().add_event("registered server".to_string(), vec![]);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("-----------------");
    println!("Parsing CLI");
    parse_cli().await
}

async fn register_hub(discovery_client: &mut DiscoveryClient<tonic::transport::Channel>, register_hub_request: RegisterHubRequest) {
    let request = tonic::Request::new(register_hub_request);

    let response = discovery_client.register_hub(request);
    let something = response.await;
    println!("RESPONSE={:?}", something.unwrap());
}

async fn get_hubs(discovery_client: &mut DiscoveryClient<tonic::transport::Channel>) {
    let request = tonic::Request::new(GetHubsRequest {
        client_id: "test".to_string(),
        name: "test".to_string(),
        host: "test".to_string(),
        port: "0".to_string(),
    });

    let response = discovery_client.get_hubs(request);
    let something = response.await;
    println!("RESPONSE={:?}", something.unwrap());
}

/// queries the Discovery Server Endpoint for its registered Gitstafette Servers
/// and returns them as a vector of GitstafetteServer
/// # Arguments
/// * `discovery_client` - DiscoveryClient
/// # Returns
/// * `Vec<GitstafetteServer>` - vector of GitstafetteServer
/// # Example
/// ```
/// let servers = get_servers(&mut discovery_client).await;
/// ```
/// # Panics
/// Panics if the Discovery Server is not reachable
/// # Errors
/// Returns an error if the Discovery Server is not reachable
/// # Remarks
/// This function is used by the Gitstafette Relay to retrieve the Gitstafette Servers
/// from the Discovery Server
async fn get_servers(discovery_client: &mut DiscoveryClient<tonic::transport::Channel>) -> Vec<GitstafetteServer> {
    let request = tonic::Request::new(GetServersRequest {
        client_id: "test".to_string(),
        name: "test".to_string(),
        host: "test".to_string(),
        port: "0".to_string(),
    });

    let response = discovery_client.get_servers(request);
    let something = response.await;
    println!("RESPONSE={:?}", something.unwrap());
    vec![]
}

/// registers a Gitstafette Server
/// # Arguments
/// * `discovery_client` - DiscoveryClient
/// * `register_server_request` - RegisterServerRequest
async fn register_server(discovery_client: &mut DiscoveryClient<tonic::transport::Channel>, register_server_request: RegisterServerRequest) {
    let request: tonic::Request<RegisterServerRequest> = tonic::Request::new(register_server_request);

    let response = discovery_client.register_server(request);
    let something = response.await;
    println!("RESPONSE={:?}", something.unwrap());
}