#![allow(dead_code, unused_variables)]

use crate::store::Transaction;

use sha2::{Sha256, Digest};

use std::time::{SystemTime, UNIX_EPOCH};

/// 加载时间戳（s）
pub fn get_current_timestamp() -> Result<u64, String> {
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    Ok(since_epoch.as_secs())
}

/// 计算区块哈希
pub fn calculate_block_hash(index: u64, timestamp: u64, operations: &Vec<Transaction>, previous_hash: &str) -> Result<String, String> {
    let block_json_string = serde_json::to_string(&(index, &timestamp, &operations, &previous_hash)).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(block_json_string);
    Ok(format!("{:x}", hasher.finalize()))
}