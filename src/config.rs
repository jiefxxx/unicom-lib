use std::{collections::HashMap, path::Path};
use crate::node::{endpoint::{ApiConfig, EndPointKind}, NodeConfig};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub unix_stream_path: String,
    pub server_addr: String,
    pub template_dir: String,
    pub app_dir: String,
    pub session_path: String,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ManifestTag{
    value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ManifestEndpoint{
    regex: String,
    kind: String,
    path: Option<String>,
    api: Option<String>,
    template: Option<String>,
    apis: Option<HashMap<String, ApiConfig>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest{
    name: String,
    templates: Option<String>,
    tags: Option<HashMap<String, String>>,
    endpoints: Option<Vec<ManifestEndpoint>>,
}

impl TryInto<NodeConfig> for Manifest{
    type Error = String;

    fn try_into(self) -> Result<NodeConfig, Self::Error> {
        let mut config = NodeConfig::new(&self.name);
        if self.templates.is_some(){
            let size = self.templates.as_ref().unwrap().len();
            for entry in WalkDir::new(self.templates.unwrap())
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok()) {
                
                if !entry.file_type().is_file(){
                    continue
                }
                let (_, name) = entry.path().to_str().unwrap().split_at(size);
                let terra_path = Path::new(&self.name).join(name);
                let absolute_path = entry.path().canonicalize().unwrap();


                config.add_template(absolute_path.to_str().unwrap(), terra_path.to_str().unwrap());
                
            }
        }

        config.tags = self.tags.unwrap_or_default();

        for endpoint in self.endpoints.unwrap_or_default(){
            config.add_endpoint(&endpoint.regex, {
                match endpoint.kind.as_str() {
                    "static" => {
                        if endpoint.path.is_none(){
                            return Err(format!("Endpoint path is None {:?}", endpoint))
                        }
                        EndPointKind::Static{path:endpoint.path.unwrap()}
                    },
                    "rest" => {
                        if endpoint.api.is_none(){
                            return Err(format!("Endpoint api is None {:?}", endpoint))
                        }
                        EndPointKind::Rest { api: endpoint.api.unwrap() }
                    },
                    "dynamic" => {
                        if endpoint.api.is_none(){
                            return Err(format!("Endpoint api is None {:?}", endpoint))
                        }
                        EndPointKind::Dynamic { api: endpoint.api.unwrap() }
                    },
                    "view" => {
                        if endpoint.template.is_none(){
                            return Err(format!("Endpoint template is None {:?}", endpoint))
                        }
                        EndPointKind::View { apis: endpoint.apis.unwrap_or_default(), template: endpoint.template.unwrap() }
                    },
                    _ => return Err(format!("Endpoint kind unknown {:?}", endpoint))
                }
            })
        }

        Ok(config)
    }
} 
