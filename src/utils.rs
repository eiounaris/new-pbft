use crate::store::Transaction;

use sha2::{Sha256, Digest};

use std::time::{SystemTime, UNIX_EPOCH};

// ---

/// 加载时间戳（s）
pub fn get_current_timestamp() -> Result<u64, String> {
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    Ok(since_epoch.as_secs())
}

// ---

/// 计算区块哈希
pub fn calculate_block_hash(
    index: u64, 
    timestamp: u64, 
    operations: &Vec<Transaction>, 
    previous_hash: &[u8]
) -> Result<Vec<u8>, String> {
    let block_bincode = bincode::serialize(&(index, &timestamp, &operations, &previous_hash))
        .map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(&block_bincode);
    Ok(hasher.finalize().as_slice().to_vec())
}