use tonic::{transport::Server, Request, Response, Status};

use gitstafette_discovery::{GetHubsRequest, GetHubsResponse,RegisterHubRequest,RegisterHubResponse, RegisterServerRequest, RegisterServerResponse, GetServersRequest, GetServersResponse, GitstafetteHub, GitstafetteServer, RegisterResponse,
   discovery_server::{Discovery, DiscoveryServer}};

use crate::store::inmemory::*;

pub mod store;

pub mod gitstafette_discovery {
  tonic::include_proto!("gitstafette_discovery");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "[::1]:50051".parse().unwrap();
    let discovery_service = DiscoveryServer::new(DiscoveryService{store: InMemoryStore::new()});

    println!("Gistafette Discovery server listening on {}", address);
    Server::builder()
      .add_service(discovery_service)
      .serve(address)
      .await?;

    Ok(())
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
