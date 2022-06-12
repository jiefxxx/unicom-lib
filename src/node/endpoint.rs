use std::collections::HashMap;

use serde_json::{Map, Value};

use super::api::MethodKind;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EndPoint{
    pub regex: String,
    pub kind: EndPointKind,
}

impl EndPoint{
    pub fn new(regex: &str, kind: EndPointKind) -> EndPoint{
        EndPoint{
            regex: regex.to_owned(),
            kind,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiConfig{
    pub node: String,
    pub api: String,
    pub method: Option<MethodKind>,
    pub parameters: Option<Map<String, Value>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum EndPointKind{
    Static{
        path: String
    },

    Dynamic{
        api: String
    },

    View{
        apis: HashMap<String, ApiConfig>,
        template: String,
    },

    Rest{
        api: String,
    }

}

impl EndPointKind {
    pub fn static_path(path: &str) -> EndPointKind{
        EndPointKind::Static { path: path.to_owned() }
    }

    pub fn rest(api: &str) -> EndPointKind{
        EndPointKind::Rest { api: api.to_owned() }
    }

    pub fn view(template: &str, apis: HashMap<String, ApiConfig>) -> EndPointKind{
        EndPointKind::View { apis, template: template.to_owned() }
    }    
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Template{
    pub file: String,
    pub path: String,
}

impl Template{
    pub fn new(file: &str, path: &str) -> Template{
        Template{
            file: file.to_owned(),
            path: path.to_owned(),
        }
    }
}

