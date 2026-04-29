use figment::{Figment, providers::{Env, Format, Toml}};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SipConfig {
    pub database_url: String,
    pub redis_url: String,
    pub minio_endpoint: String,
    pub minio_bucket: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiration_seconds")]
    pub jwt_expiration_seconds: u64,
    #[serde(default = "default_refresh_expiration_seconds")]
    pub refresh_expiration_seconds: u64,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
    #[serde(default = "default_server_host")]
    pub server_host: String,
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default = "default_app_env")]
    pub app_env: String,
}

fn default_jwt_expiration_seconds() -> u64 {
    900
}

fn default_refresh_expiration_seconds() -> u64 {
    604800
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "llama3.1:8b".to_string()
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_server_port() -> u16 {
    8000
}

fn default_app_env() -> String {
    "development".to_string()
}

pub fn load_config() -> Result<SipConfig, figment::Error> {
    Figment::new()
        .merge(Toml::file("Sip.toml"))
        .merge(Env::prefixed("SIP_"))
        .extract()
}
