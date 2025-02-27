use crate::utils::{serialize_public_key, deserialize_public_key};

use serde::{Deserialize, Serialize};
use rsa::RsaPublicKey;

/// 节点身份（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub node_id: u64,
    pub ip: String,
    pub port: u16,
    #[serde(serialize_with = "serialize_public_key", deserialize_with = "deserialize_public_key")]
    pub public_key: RsaPublicKey,
}
impl Identity {
    pub fn build_identity(filepath: String) -> Result<Vec<Identity>, String> {
        let identity_json = std::fs::read_to_string(filepath).map_err(|e| e.to_string())?;
        let identitys: Vec<Identity> = serde_json::from_str(&identity_json).map_err(|e| e.to_string())?;
        Ok(identitys)
    }
}

/// 系统配置（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub view_number: u64,
    pub database_name: String,
    pub multi_cast_socket: String,
    pub block_size: u64,
}
impl Config {
    pub fn build_config(filepath: String) -> Result<Config, String> {
        let config_json = std::fs::read_to_string(filepath).map_err(|e| e.to_string())?;
        let config: Config = serde_json::from_str(&config_json).map_err(|e| e.to_string())?;
        Ok(config)
    }
}