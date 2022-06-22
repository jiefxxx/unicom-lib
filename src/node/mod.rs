use std::{collections::HashMap, sync::Arc};

use serde_json::{Map, Value};

use crate::error::{UnicomError, UnicomErrorKind};

use self::{ api::{Api, MethodKind, ApiMethod}, message::{request::UnicomRequest, UnicomMessage}, 
        message::response::UnicomResponse, endpoint::{EndPoint, Template, EndPointKind}};

use async_trait::async_trait;

pub mod api;
pub mod message;
pub mod endpoint;
pub mod utils;

pub struct Node{
    pub name: String,
    
    api: Vec<Api>,
    
    tags: HashMap<String, String>,

    connector: Arc<dyn NodeConnector>,
}

impl Node{
    pub async fn new(config: &NodeConfig, connector:  Arc<dyn NodeConnector>) -> Result<Node, UnicomError>{

        Ok(Node{
            name: config.name.clone(),
            api: config.api.clone(),
            tags: config.tags.clone(),
            connector,
        })

    }

    pub fn api(&self, name: &str) -> Result<&Api, UnicomError>{
        for api in &self.api{
            if api.name == name {
                return Ok(api)
            }
        }
        Err(UnicomError::new(UnicomErrorKind::NotFound, &format!("Api {} not found", name)))
    }

    pub async fn request(&self, api: &Api, method: MethodKind, parameters: Map<String, Value>) -> Result<UnicomResponse, UnicomError>{
        api.get_method(&method)?.generate_parameters(&parameters)?;

        Ok(self.connector.request(UnicomRequest{
            id: api.id,
            parameters,
            method,
            name: String::new(),
            node_name: String::new(),
        }).await?)
    }

    pub async fn response(&self, request_id: u64, data: Vec<u8>) -> Result<(), UnicomError> {
        Ok(self.connector.response(request_id, UnicomResponse{
            data,
        }).await?)
    }

    pub async fn error(&self, request_id: u64, error: UnicomError) -> Result<(), UnicomError>{
        self.connector.error(request_id, error).await
    }

    pub async fn next(&self) -> Result<UnicomMessage, UnicomError>{
        self.connector.next().await
    }

    pub async fn quit(&self) -> Result<(), UnicomError>{
        self.connector.quit().await
    }

    pub async fn get_tag(&self, tag: &str) -> Option<&String>{
        self.tags.get(tag)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NodeConfig{
    pub name: String,
    pub api: Vec<Api>,
    pub tags: HashMap<String, String>,
    pub endpoints: Vec<EndPoint>,
    pub templates: Vec<Template>,
    
}

impl NodeConfig{
    pub fn new(name: &str) -> NodeConfig{
        NodeConfig { 
            name: name.to_owned(), 
            api: Vec::new(), 
            tags: HashMap::new(), 
            endpoints: Vec::new(), 
            templates: Vec::new()
        }
    }

    pub fn from_utf8(message: Vec<u8>) -> Result<NodeConfig, UnicomError>{
        Ok(serde_json::from_str(&String::from_utf8(message)?)?)
    }

    pub fn add_api(&mut self, id: u64, name: &str, methods : Vec<ApiMethod>){
        self.api.push(Api::new(id, name, methods))
    }

    pub fn add_template(&mut self, file: &str, path: &str){
        self.templates.push(Template::new(file, path))
    }

    pub fn add_endpoint(&mut self, regex: &str, kind: EndPointKind){
        self.endpoints.push(EndPoint::new(regex, kind))
    }
    


}

#[async_trait]
pub trait NodeConnector: Send + Sync{
    async fn init(&self) -> Result<NodeConfig, UnicomError>;
    async fn request(&self, request: UnicomRequest) -> Result<UnicomResponse, UnicomError>;
    async fn response(&self, request_id: u64, response: UnicomResponse) -> Result<(), UnicomError>;
    async fn error(&self, request_id: u64, error: UnicomError) -> Result<(), UnicomError>;
    async fn next(&self) -> Result<UnicomMessage, UnicomError>;
    async fn quit(&self) -> Result<(), UnicomError>;

    //async fn wait_request(&self) -> Result<UnicomRequest, UnicomError>;
}