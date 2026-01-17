mod config_parser;
mod generated;
mod parser;

use config_parser::{ConfigItem, ConfigParser, ConfigValue};
use generated::container::{
    GetContainersRequest, GetContainersResponse, list_containers_server::ListContainersServer,
};
use generated::hello::{HelloRequest, HelloResponse, hello_world_server::HelloWorldServer};
use tonic::{Request, Response, Status, transport::Server};

use crate::generated::container::Container;

#[derive(Debug, Default)]
pub struct MyHelloWorld {}

#[tonic::async_trait]
impl generated::hello::hello_world_server::HelloWorld for MyHelloWorld {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let name = request.into_inner().name;
        let reply = HelloResponse {
            message: format!("Hello, {}!", name),
        };
        Ok(Response::new(reply))
    }
}

#[derive(Default, Debug)]
pub struct ListContainers {
    parser: ConfigParser,
}

impl ListContainers {
    pub fn new() -> Self {
        Self {
            parser: ConfigParser::new(),
        }
    }

    fn config_item_to_container(&self, item: &ConfigItem) -> Container {
        let name = item.name.clone();
        let id = item.values.get("ip4.addr").and_then(|v| match v {
            ConfigValue::String(ip) => ip.split('.').last().and_then(|s| s.parse::<i32>().ok()),
            _ => None,
        });

        let dataset = item
            .values
            .get("path")
            .and_then(|v| match v {
                ConfigValue::String(path) => Some(path.clone()),
                _ => None,
            })
            .unwrap_or_else(|| format!("zpool/datasets/containers/{}", name));

        let addresses = vec![format!("{}.local", name)];

        // For this example, we'll assume containers are not running
        // In a real implementation, you'd check the actual status
        let running = false;

        Container {
            name,
            id,
            dataset,
            addresses,
            running,
        }
    }
}

#[tonic::async_trait]
impl generated::container::list_containers_server::ListContainers for ListContainers {
    async fn get_containers(
        &self,
        _request: Request<GetContainersRequest>,
    ) -> Result<Response<GetContainersResponse>, Status> {
        let mut containers = Vec::new();

        // Read all .conf files in the examples directory
        let examples_dir = std::path::Path::new("examples");
        if let Ok(entries) = std::fs::read_dir(examples_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("conf") {
                    if let Ok(config_items) = self.parser.parse_file(&path) {
                        for item in config_items {
                            containers.push(self.config_item_to_container(&item));
                        }
                    }
                }
            }
        }

        let reply = GetContainersResponse { containers };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let hello_world = MyHelloWorld::default();
    let list_containers = ListContainers::new();

    println!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(HelloWorldServer::new(hello_world))
        .add_service(ListContainersServer::new(list_containers))
        .serve(addr)
        .await?;

    Ok(())
}
