use std::collections::HashMap;
use std::sync::{Arc, Mutex};


#[derive(Debug, Clone)]
pub struct GSFHub {
  pub id: String,
  pub name: String,
  pub version: String,
  pub host: String,
  pub port: String,
  pub repositories: String,
  pub relay_host: String,
  pub relay_port: String,
}

#[derive(Debug, Clone)]
pub struct GSFServer {
  pub id: String,
  pub name: String,
  pub version: String,
  pub host: String,
  pub port: String,
  pub repositories: String
}


// Define the API interface
pub trait Store {
  fn new() -> Self;
  fn add_hub(&self, hub: GSFHub);
  fn get_hub(&self, id: String) -> Option<GSFHub>;
  fn get_hubs(&self) -> Vec<GSFHub>;
  fn update_hub(&self, hub: GSFHub);
  fn remove_hub(&self, id: String);

  fn add_server(&self, hub: GSFServer);
  fn get_server(&self, id: String) -> Option<GSFServer>;
  fn get_servers(&self) -> Vec<GSFServer>;
  fn update_server(&self, hub: GSFServer);
  fn remove_server(&self, id: String);
}

#[derive(Debug)]
pub struct InMemoryStore {
    hubs: Arc<Mutex<HashMap<String, GSFHub>>>,
    servers: Arc<Mutex<HashMap<String, GSFServer>>>,
}

impl Default for InMemoryStore {
  fn default() -> Self {
    InMemoryStore {
      hubs: Arc::new(Mutex::new(HashMap::new())),
      servers: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

impl Store for InMemoryStore {
  fn new() -> Self {
    InMemoryStore {
        hubs: Arc::new(Mutex::new(HashMap::new())),
        servers: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  fn add_hub(&self, gsfhub: GSFHub) {
    let mut hubs = self.hubs.lock().unwrap();
    println!("Added hub: {:?}", gsfhub);
    hubs.insert(gsfhub.id.clone(), gsfhub);

  }

  fn get_hub(&self, id: String) -> Option<GSFHub> {
    let hubs = self.hubs.lock().unwrap();
    hubs.get(&id).cloned()
  }

  fn get_hubs(&self) -> Vec<GSFHub> {
    let hubs = self.hubs.lock().unwrap();
    hubs.values().cloned().collect()
  }

  fn update_hub(&self, gsfhub: GSFHub) {
    let mut hubs = self.hubs.lock().unwrap();
    hubs.insert(gsfhub.id.clone(), gsfhub);
  }

  fn remove_hub(&self, id: String) {
    let mut hubs = self.hubs.lock().unwrap();
    hubs.remove(&id);
  }

  fn add_server(&self, gsfserver: GSFServer) {
    let mut servers = self.servers.lock().unwrap();
    println!("Added server: {:?}", gsfserver);
    servers.insert(gsfserver.id.clone(), gsfserver);
  }

  fn get_server(&self, id: String) -> Option<GSFServer> {
    let servers = self.servers.lock().unwrap();
    servers.get(&id).cloned()
  }

  fn get_servers(&self) -> Vec<GSFServer> {
    let servers = self.servers.lock().unwrap();
    servers.values().cloned().collect()
  }

  fn update_server(&self, gsfserver: GSFServer) {
    let mut servers = self.servers.lock().unwrap();
    servers.insert(gsfserver.id.clone(), gsfserver);
  }

  fn remove_server(&self, id: String) {
    let mut servers = self.servers.lock().unwrap();
    servers.remove(&id);
  }

}