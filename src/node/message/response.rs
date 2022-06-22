use serde_json::Value;

use crate::error::UnicomError;

#[derive(Debug)]
pub struct UnicomResponse{
    pub data: Vec<u8>,
}


impl UnicomResponse {
    pub fn empty() -> UnicomResponse{
        UnicomResponse { data: "{}".into() }
    }

    pub fn from_json(data: &Value) -> Result<UnicomResponse, UnicomError>{
        Ok(UnicomResponse { data: serde_json::to_string(data)?.as_bytes().to_vec() })
    }

    pub fn from_string(data: String) -> UnicomResponse{
        UnicomResponse { data: data.as_bytes().to_vec() }
    }
}

