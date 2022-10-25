#[macro_use]
extern crate serde_derive;

pub mod node;
pub mod error;
pub mod arch;
pub mod config;


use std::{sync::Arc, collections::HashMap};

use arch::unix::{write_init, UnixMessage, write_message, read_message};
use async_trait::async_trait;
use config::{Manifest, Config};
use error::{UnicomError, UnicomErrorKind};
use node::{api::{ApiMethod, MethodKind}, message::request::UnicomRequest, utils::pending::PendingController, NodeConfig};
use serde_json::{Map, Value};
use tokio::{sync::{Mutex, Notify}, net::{unix::{OwnedWriteHalf, OwnedReadHalf}, UnixStream}};

#[async_trait]
pub trait UnicomApi: Sync + Send {
    fn name(&self) -> String;
    fn description(&self) -> Vec<ApiMethod>;
    async fn api_get(&self, server: &Arc<ServerConnection>, request: &UnicomRequest) -> Result<Vec<u8>, UnicomError>;
    async fn api_put(&self, server: &Arc<ServerConnection>, request: &UnicomRequest) -> Result<Vec<u8>, UnicomError>;
    async fn api_post(&self, server: &Arc<ServerConnection>, request: &UnicomRequest) -> Result<Vec<u8>, UnicomError>;
    async fn api_delete(&self, server: &Arc<ServerConnection>, request: &UnicomRequest) -> Result<Vec<u8>, UnicomError>;
}

pub struct ServerConnection{
    stream_path: String,
    api: HashMap<u16, Arc<dyn UnicomApi>>,
    manifest: Manifest,
    counter: u16,
    writer: Mutex<Option<OwnedWriteHalf>>,
    pub pending: PendingController,
}

impl ServerConnection{
    pub fn new(path: Option<&str>) -> ServerConnection{
        let stream_path: String;
        if path.is_none(){
            let content = std::fs::read_to_string("/etc/unicom/config.toml").expect("Failed to read unicom config");
            let config: Config = toml::from_str(&content).expect("Failed to parse unicom config");
            stream_path = config.unix_stream_path.clone();
        }
        else{
            stream_path = path.unwrap().to_string();
        }

        let content = std::fs::read_to_string("manifest.toml").expect("Failed to read manifest");
        let manifest: Manifest = toml::from_str(&content).expect("Failed to parse manifest");
        

        ServerConnection { 
            stream_path, 
            api: HashMap::new(),
            counter: 0, 
            manifest,
            writer: Mutex::new(None) ,
            pending: PendingController::new(),
        }
    }

    pub fn add_api(&mut self, api: Arc<dyn UnicomApi>){
        self.api.insert(self.counter, api);
        self.counter += 1;
    }

    fn gen_config(&self) -> NodeConfig{
        let mut config: NodeConfig = self.manifest.clone().try_into().expect("Failed to generate config from manifest");
        for (id, api) in &self.api{
            config.add_api((*id).into(), &api.name(), api.description());
        }
        config
    }

    async fn connect(&self) -> OwnedReadHalf{
        let (reader, mut writer) = UnixStream::connect(&self.stream_path).await.unwrap().into_split();
        write_init(&mut writer, &self.gen_config()).await.expect("write init error");
        let mut data = self.writer.lock().await;
        *data = Some(writer);
        reader
    }

    pub async fn request(&self, node: &str, name: &str, parameters: Map<String, Value>) -> Result<Vec<u8>, UnicomError>{
        let mut data = UnicomRequest::new();
        data.node_name = node.to_string();
        data.name = name.to_string();
        data.parameters = parameters;
        let (id, notify) = self.pending.create().await;

        write_message(&mut self.writer.lock().await.as_mut().unwrap(), UnixMessage::Request { id, data }).await?;

        notify.notified().await;

        self.pending.get(id).await
    }

    pub async fn run(server: &Arc<ServerConnection>) -> Arc<Notify>{
        let mut reader = server.connect().await;
        let notify = Arc::new(Notify::new());
        let notify_back = notify.clone();
        let server = server.clone();
        tokio::spawn(async move {
            loop {
                let mess = match read_message(&mut reader).await {
                    Ok(mess) => mess,
                    Err(e) => {
                        println!("error read message {:?}",e);
                        notify_back.notify_one();
                        return
                    },
                };
                let notify_back = notify_back.clone();
                let server = server.clone();
                tokio::spawn(async move {
                    match mess {
                        UnixMessage::Error { id, error } => {
                            if id == 0{
                                println!("config error : {:?}", error);
                                notify_back.notify_one();
                                return
                            }
                            server.pending.update(id, Err(error)).await.unwrap()
                        },
                        UnixMessage::Response { id, data } => server.pending.update(id, Ok(data)).await.unwrap(),
                        UnixMessage::Request { id, data } => {
                            let handler = match server.api.get(&(data.id as u16)){
                                Some(handler) => handler,
                                None => {
                                    let error = UnicomError::new(UnicomErrorKind::NotFound, &format!("api id not found {:?}", data));
                                    write_message(&mut server.writer.lock().await.as_mut().unwrap(), 
                                    UnixMessage::Error { id, error: error.into() }).await.unwrap();
                                    return
                                },
                            };
                            let ret = match data.method{
                                MethodKind::GET => handler.api_get(&server, &data),
                                MethodKind::PUT => handler.api_put(&server, &data),
                                MethodKind::POST => handler.api_post(&server, &data),
                                MethodKind::DELETE => handler.api_delete(&server, &data),
                            }.await;
            
                            match ret{
                                Ok(data) => {
                                    write_message(&mut server.writer.lock().await.as_mut().unwrap(), 
                                UnixMessage::Response { id, data }).await.unwrap();
            
                                },
                                Err(error) => {
                                    write_message(&mut server.writer.lock().await.as_mut().unwrap(), 
                                    UnixMessage::Error { id, error: error.into() }).await.unwrap();
                                },
                            }
                            
                        },
                        UnixMessage::Quit => notify_back.notify_one(),
                    }
                });
                
            }
        });

        notify
    }

}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum UserLevel {
    Admin,
    Root,
    Normal,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User{
    pub name: String,
    pub level: UserLevel,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InputFile{
    pub path: String
}


