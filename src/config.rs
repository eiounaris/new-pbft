use serde::Deserialize;
use rsa::{pkcs1::DecodeRsaPublicKey, RsaPublicKey};

// ---

/// 身份配置（fine）
#[derive(Deserialize)]
pub struct Identity {
    pub node_id: u64,
    pub ip: String,
    pub port: u16,
    #[serde(deserialize_with = "deserialize_public_key")]
    pub public_key: RsaPublicKey,
}
impl Identity {
    /// 从文件加载节点身份信息
    pub fn load_identity(filepath: String) -> Result<Vec<Identity>, String> {
        let identity_json = std::fs::read_to_string(filepath).map_err(|e| e.to_string())?;
        let identitys: Vec<Identity> = serde_json::from_str(&identity_json).map_err(|e| e.to_string())?;
        Ok(identitys)
    }
}

/// RsaPublicKey 反序列化函数
pub fn deserialize_public_key<'de, D>(deserializer: D) -> Result<RsaPublicKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let pem: String = Deserialize::deserialize(deserializer)?;
    RsaPublicKey::from_pkcs1_pem(&pem).map_err(serde::de::Error::custom)
}

// ---

/// 系统配置（fine）
#[derive(Deserialize)]
pub struct SystemConfig {
    pub view_number: u64,
    pub database_name: String,
    pub multi_cast_ip: String,
    pub multi_cast_port: u64,
    pub block_size: u64,
}
impl SystemConfig {
    /// 从文件加载系统配置
    pub fn load_system_config(filepath: String) -> Result<SystemConfig, String> {
        let system_config_json = std::fs::read_to_string(filepath).map_err(|e| e.to_string())?;
        let system_config: SystemConfig = serde_json::from_str(&system_config_json).map_err(|e| e.to_string())?;
        Ok(system_config)
    }
}