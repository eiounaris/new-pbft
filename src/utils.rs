use super::store::Transaction;

use serde::Deserialize;
use rsa::{pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey, LineEnding}, RsaPublicKey};
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

/// 加载时间戳（s）
pub fn get_current_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .unwrap();
    since_epoch.as_secs()
}

/// 计算区块哈希
pub fn calculate_block_hash(index: u64, timestamp: u64, operations: &Vec<Transaction>, previous_hash: &str) -> String {
    let block_json_string = serde_json::to_string(&(index, &timestamp, &operations, &previous_hash)).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(block_json_string);
    format!("{:x}", hasher.finalize())
}

/// RsaPublicKey 序列化函数
pub fn serialize_public_key<S>(public_key: &RsaPublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let pem = public_key.to_pkcs1_pem(LineEnding::default()).expect("Failed to convert public key to PEM");
    serializer.serialize_str(&pem)
}

/// RsaPublicKey 反序列化函数
pub fn deserialize_public_key<'de, D>(deserializer: D) -> Result<RsaPublicKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let pem: String = Deserialize::deserialize(deserializer)?;
    RsaPublicKey::from_pkcs1_pem(&pem).map_err(serde::de::Error::custom)
}