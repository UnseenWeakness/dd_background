use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
// 配置结构体
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
}

// 数据库配置子结构体
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub dbname: String,
}

// 从 YAML 文件加载配置
pub fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_yaml::from_reader(reader)?;
    Ok(config)
}
