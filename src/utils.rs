use crate::pbft::Pbft;
use crate::state::State;

use super::store::Transaction;


use reqwest::Client;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use std::sync::Arc;
use tokio::net::unix::SocketAddr;
use tokio::sync::RwLock;
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

/// PBFT 初始化函数
pub async  fn init() -> Result<(Arc<UdpSocket>, Arc<Client>, Arc<Vec<SocketAddr>>, Arc<RwLock<State>>, Arc<RwLock<Pbft>>, tokio::sync::mpsc::Sender<()>, tokio::sync::mpsc::Receiver<()>), String> {
   todo!()
}