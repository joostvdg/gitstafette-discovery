use clap::{Parser, Subcommand};

use gitstafette_discovery::{
    discovery_client::DiscoveryClient, GetHubsRequest, GitstafetteHub, RegisterHubRequest,GitstafetteServer, GetServersRequest, RegisterServerRequest
};

use gitstafette_info:: {
    info_client::InfoClient, GetInfoRequest, InstanceType
};

// https://timvw.be/2022/04/28/notes-on-using-grpc-with-rust-and-tonic/
#[path = "gitstafette_discovery.rs"]
pub mod gitstafette_discovery;

#[path = "gitstafette_info.rs"]
pub mod gitstafette_info;


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
            loop {
                let server = format!("{}://{}:{}", info_protocol, info_host, info_port);
                let mut info_client: InfoClient<tonic::transport::Channel> = InfoClient::connect(server).await?;
                let info_request = GetInfoRequest{ client_id: "myself".to_string(), client_endpoint: "127.0.0.1:50051".to_string() };

                let request: tonic::Request<GetInfoRequest> = tonic::Request::new(info_request);
                let response = info_client.get_info(request);
                let result = response.await;
                let info = result.unwrap();
                println!("Service Info={:?}", info);

                // depending on the response, we should register the server/hub? to the Discovery Server
                if info.get_ref().alive {

                    let server_info_opt = info.get_ref().server.as_ref();
                    let relay_info_opt =  info.get_ref().relay.as_ref();


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

                        // if let Some(i) = path.find('?') {
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

                        let request= RegisterHubRequest {
                            hub: Some(hub),
                        };
                        register_hub(&mut discovery_client, request).await;
                    }
                    else {
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

                        let request= RegisterServerRequest {
                            server: Some(gsf_server),
                        };
                        register_server(&mut discovery_client, request).await;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
        None => {}
    }
    Ok(())
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