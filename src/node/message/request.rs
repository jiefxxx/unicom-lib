use serde_json::{Map, Value};

use crate::error::{UnicomError, UnicomErrorKind};

use super::super::api::MethodKind;

#[derive(Debug, Deserialize, Serialize)]
pub struct UnicomRequest{
    pub id: u64,
    pub name: String,
    pub node_name: String,
    pub method: MethodKind,
    pub parameters: Map<String,Value>,
}

impl UnicomRequest{
    pub fn new() -> UnicomRequest{
        UnicomRequest{
            id: 0,
            name: String::new(),
            node_name: String::new(),
            method: MethodKind::GET,
            parameters: Map::new(),
        }
    }
    pub fn from_utf8(message: Vec<u8>) -> Result<UnicomRequest, UnicomError>{
        if let Ok(message) = String::from_utf8(message){
            match serde_json::from_str(&message) {
                Ok(v) => Ok(v),
                Err(e) => Err(UnicomError::new(UnicomErrorKind::ParseError, &e.to_string())),
            }
        }
        else{
            Err(UnicomError::new(UnicomErrorKind::ParseError, "Could not parse body to String"))
        }
    }
}




