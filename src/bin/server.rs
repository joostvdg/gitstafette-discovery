use std::error::Error;
use tonic::{transport::Server, Request, Response, Status};
use std::net::SocketAddr;
use autometrics::{autometrics, prometheus_exporter};
use autometrics::prometheus_exporter::PrometheusResponse;
use axum::{routing::get, Router};
use clap::{Parser};

use gitstafette_discovery::{GetHubsRequest, GetHubsResponse,RegisterHubRequest,RegisterHubResponse, RegisterServerRequest, RegisterServerResponse, GetServersRequest, GetServersResponse, GitstafetteHub, GitstafetteServer, RegisterResponse,
   discovery_server::{Discovery, DiscoveryServer}};

use gitstafette_info::{GetInfoRequest, GetInfoResponse, InstanceType, ServerInfo, info_server::{Info, InfoServer}};

use crate::store::inmemory::*;



mod store;

// https://timvw.be/2022/04/28/notes-on-using-grpc-with-rust-and-tonic/
#[path = "gitstafette_discovery.rs"]
pub mod gitstafette_discovery;

#[path = "gitstafette_info.rs"]
pub mod gitstafette_info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Gitstatfette Discovery Server Listen Address
    #[arg(short, long, default_value = "[::1]")]
    listener_address: String,

    /// Gitstatfette Discovery Server Port
    #[arg(short, long, default_value = "50051")]
    port: String,

}

#[tokio::main]
pub async fn main() {
  // Set up the exporter to collect metrics
  prometheus_exporter::init();

  let cli = Cli::parse();
  let address = format!("{}:{}", cli.listener_address, cli.port);
  let discovery_service = DiscoveryServer::new(DiscoveryService{store: InMemoryStore::new()});
  let info_service = InfoServer::new(InfoService{});
  let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
  health_reporter.set_serving::<DiscoveryServer<DiscoveryService>>().await;

  // create SocketAddr from address
  let socket_address = address.parse().unwrap();
  println!("Gistafette Discovery server listening on {}", address);
  tokio::spawn(async move {
  Server::builder()
    .add_service(health_service)
    .add_service(discovery_service)
    .add_service(info_service)
    .serve(socket_address)
    .await
    .expect("gRPC server failed");
  });

  // Web server with Axum
  let web_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
  let app = Router::new()
      .route("/", get(handler))
      .route(
        "/metrics",
        get(|| async { prometheus_exporter::encode_http_response() }),
  );

  axum::Server::bind(&web_addr)
      .serve(app.into_make_service())
      .await
      .expect("Web server failed");

}

#[autometrics]
async fn handler() -> &'static str {
  "Hello, World!"
}

#[derive(Debug, Default)]
pub struct DiscoveryService {
  store: InMemoryStore
}

// rpc RegisterHub(RegisterHubRequest) returns (RegisterHubResponse) {}
// rpc RegisterServer(RegisterServerRequest) returns (RegisterServerResponse) {}

// rpc GetHubs(GetHubsRequest) returns (GetHubsResponse) {}
// rpc GetServers(GetServersRequest) returns (GetServersResponse) {}

#[tonic::async_trait]
impl Discovery for DiscoveryService {

  async fn register_server(&self, request: Request<RegisterServerRequest>) -> Result<Response<RegisterServerResponse>, Status> {
    println!("Got a request: {:?}", request);
    let response: RegisterResponse = gitstafette_discovery::RegisterResponse {
      success: true,
      message: "Hub registered".to_string(),
      error: "".to_string(),
      error_code: "".to_string(),
    };

    let server = request.into_inner().server.unwrap();
    let server_internal = GSFServer {
      id: server.id.to_string(),
      name: server.name.to_string(),
      version: server.version.to_string(),
      host: server.host.to_string(),
      port: server.port.to_string(),
      repositories: server.repositories.to_string(),
    };
    self.store.add_server(server_internal);
    return Ok(Response::new(RegisterServerResponse{
      response: Some(response),
    }));
  }

  async fn register_hub(&self, request: Request<RegisterHubRequest>) -> Result<Response<RegisterHubResponse>, Status> {
    println!("Got a request: {:?}", request);
    
    let response: RegisterResponse = gitstafette_discovery::RegisterResponse {
      success: true,
      message: "Hub registered".to_string(),
      error: "".to_string(),
      error_code: "".to_string(),
    };

    let hub = request.into_inner().hub.unwrap();
    let hub_internal = GSFHub {
      id: hub.id.to_string(),
      name: hub.name.to_string(),
      version: hub.version.to_string(),
      host: hub.host.to_string(),
      port: hub.port.to_string(),
      repositories: hub.repositories.to_string(),
      relay_host: hub.relay_host.to_string(),
      relay_port: hub.relay_port.to_string(),
    };
    self.store.add_hub(hub_internal);

    return Ok(Response::new(RegisterHubResponse{
      response: Some(response),
    }));
  }

  async fn get_servers(&self, request: Request<GetServersRequest>) -> Result<Response<GetServersResponse>, Status> {
    println!("Got a request: {:?}", request);

    let mut servers: Vec<GitstafetteServer> = Vec::new();
    for internal_server in self.store.get_servers() {
      let server = GitstafetteServer {
        id: internal_server.id.to_string(),
        name: internal_server.name.to_string(),
        version: internal_server.version.to_string(),
        host: internal_server.host.to_string(),
        port: internal_server.port.to_string(),
        repositories: internal_server.repositories.to_string(),
      };
      servers.push(server);
    }

    return Ok(Response::new(GetServersResponse {
      servers,
    }));
  }

  async fn get_hubs(&self, request: Request<GetHubsRequest>) -> Result<Response<GetHubsResponse>, Status> {
    println!("Got a request: {:?}", request);

    let mut hubs: Vec<GitstafetteHub> = Vec::new();
    for internal_hub in self.store.get_hubs() {
      let hub = GitstafetteHub {
        id: internal_hub.id.to_string(),
        name: internal_hub.name.to_string(),
        version: internal_hub.version.to_string(),
        host: internal_hub.host.to_string(),
        port: internal_hub.port.to_string(),
        repositories: internal_hub.repositories.to_string(),
        relay_host: internal_hub.relay_host.to_string(),
        relay_port: internal_hub.relay_port.to_string(),
      };
      hubs.push(hub);
    }

    return Ok(Response::new(GetHubsResponse {
      hubs,
    }));
  }
}


#[derive(Debug, Default)]
pub struct InfoService {

}

#[tonic::async_trait]
impl Info for InfoService {

    async fn get_info(&self, request: Request<GetInfoRequest>) -> Result<Response<GetInfoResponse>, Status> {
      println!("Got a request: {:?}", request);

      // collect Hostname if its set, else use localhost
      let hostname_env = std::env::var("HOSTNAME");
      let hostname = hostname_env.unwrap_or_else(|_| "localhost".to_string());

      let server_info = ServerInfo {
        hostname: hostname.to_string(),
        ip: "127.0.0.1".to_string(),
        port: "50051".to_string(),
        protocol: "http".to_string(),
        repositories: None,
      };

      let response = GetInfoResponse {
        alive: true,
        instance_type: InstanceType::Discovery.into(),
        version: "0.1.0".to_string(),
        name: "Gitstafette Discovery".to_string(),
        server: Some(server_info),
        relay: None,
      };

      return Ok(Response::new(response));
    }
}