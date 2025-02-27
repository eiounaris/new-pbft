use super::store::{Transaction, Block};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 消息类型（待调整）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request = 0,
    PrePrepare = 1,
    Prepare = 2,
    Commit = 3,
    Reply = 4,

    Hearbeat = 5,
    ViewChange = 6,
    NewView = 7,
    
    ViewRequest = 8,
    ViewReply = 9,

    StateRequest = 10,
    StateReply = 11,

    SyncRequest = 12,
    SyncResponse = 13,

    Unknown = 20,
}

/// 请求消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub transaction: Transaction,
    pub timestamp: u64,
    pub node_id: u64,
    pub signature: Vec<u8>, // -> all
}
impl Request {
    pub fn digest_requests(requests: &Vec<Request>) -> Result<Vec<u8>, String> {
        Ok(Sha256::digest(bincode::serialize(&requests).map_err(|e| e.to_string())?).to_vec())
    }
}

/// 预准备消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrePrepare {
    pub view_number: u64,
    pub sequence_number: u64,
    pub digest: Vec<u8>, // -> requests
    pub node_id: u64,
    pub signature: Vec<u8>, // -> view_number, sequence_number, digest, node_id
    pub requests: Vec<Request>,
    pub block: Block,
}

/// 准备消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prepare {
    pub view_number: u64,
    pub sequence_number: u64,
    pub digest: Vec<u8>, // -> PrePrepare.digest
    pub node_id: u64,
    pub signature: Vec<u8>, // -> all
}

/// 提交消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub view_number: u64,
    pub sequence_number: u64,
    pub digest: Vec<u8>, // -> PrePrepare.digest
    pub node_id: u64,
    pub signature: Vec<u8>, // -> all
}

/// 回应消息（PBFT 论文中涉及，目前暂时保留，该场景使用不到）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub view_number: u64,
    pub timestamp: u64,
    pub client_id: u64,
    pub node_id: u64,
    pub result: String, // -> PrePrepare.digest
    pub signature: Vec<u8>, // -> all
}

/// 视图切换消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewChange {
    pub view_number: u64, 
    pub sequence_number: u64,
    pub next_view_number: u64, 
    pub node_id: u64,
    pub signature: Vec<u8>, // -> all
}

/// 新试图消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewView {
    pub view_number: u64,
    pub sequence_number: u64,
    pub next_view_number: u64, 
    pub node_id: u64,
    pub signature: Vec<u8>, // -> all
}

/// 心跳消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hearbeat {
    pub view_number: u64,
    pub sequence_number: u64,
    pub node_id: u64,
    pub signature: Vec<u8>, // -> all
}

/// 同步请求消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub view_number: u64,
    pub sequence_number: u64,
    pub node_id: u64,
    pub from_index: u64,
    pub to_index: u64,
    pub signature: Vec<u8>, // -> all
}

/// 同步响应消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub view_number: u64,
    pub sequence_number: u64,
    pub node_id: u64,
    pub blocks: Vec<Block>,
    pub signature: Vec<u8>, // -> all
}