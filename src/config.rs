use serde::{Deserialize, Serialize};
use rsa::{pkcs1::DecodeRsaPublicKey, RsaPublicKey};
use tokio::fs;
// ---

/// 身份配置（fine）
#[derive(Deserialize)]
pub struct Identity {
    pub node_id: u64,
    pub ip: String,
    pub port: u64,
    #[serde(deserialize_with = "deserialize_public_key")]
    pub public_key: RsaPublicKey,
}
impl Identity {
    /// 从文件加载节点身份信息
    pub async fn load_identity_config(filepath: String) -> Result<Vec<Identity>, String> {
        let identitys_config_json = fs::read_to_string(filepath).await
            .map_err(|e| e.to_string())?;
        let identitys: Vec<Identity> = serde_json::from_str(&identitys_config_json)
            .map_err(|e| e.to_string())?;
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

/// 持久配置（fine）
#[derive(Deserialize)]
pub struct ConstantConfig {
    pub database_name: String,
    pub multi_cast_addr: String,
    pub multi_cast_ip: String,
    pub multi_cast_port: u64,
    pub block_size: u64,
    pub variable_config_path: String,
}
impl ConstantConfig {
    /// 从文件加载持久配置
    pub async fn load_constant_config(filepath: String) -> Result<ConstantConfig, String> {
        let constant_config_json = fs::read_to_string(filepath).await
            .map_err(|e| e.to_string())?;
        let constant_config: ConstantConfig = serde_json::from_str(&constant_config_json)
            .map_err(|e| e.to_string())?;
        Ok(constant_config)
    }
}

/// 动态配置（fine）
#[derive(Deserialize, Serialize)]
pub struct VariableConfig {
    pub view_number: u64,
}
impl VariableConfig {
    /// 从文件加载动态配置
    pub async fn load_variable_config(filepath: String) -> Result<VariableConfig, String> {
        let variable_config_json = fs::read_to_string(filepath).await
            .map_err(|e| e.to_string())?;
        let variable_config: VariableConfig = serde_json::from_str(&variable_config_json)
            .map_err(|e| e.to_string())?;
        Ok(variable_config)
    }
}