#[derive(Debug, Deserialize)]
pub struct Config {
    pub unix_stream_path: String,
    pub server_addr: String,
    pub template_dir: String,
    pub app_dir: String,
    pub session_path: String,
}

pub fn read_config() -> Config{
    let content = if std::path::Path::new("./config.toml").exists(){
        std::fs::read_to_string("./config.toml").unwrap()
    }
    else{
        std::fs::read_to_string("/etc/unicom/config.toml").unwrap()
    };
    toml::from_str(&content).unwrap()
    
}