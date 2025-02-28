use crate::network::send_udp_data;
use crate::pbft::Step;
use crate::store::{Block, BlockStore, Transaction};
use crate::utils::get_current_timestamp;
use crate::SystemConfig;
use crate::Client;
use crate::State;
use crate::Pbft;
use crate::key::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tokio::sync::mpsc;

use std::net::SocketAddr;
use std::sync::Arc;
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
    ViewResponse = 9,

    StateRequest = 10,
    StateResponse = 11,

    SyncRequest = 12,
    SyncResponse = 13,

    Unknown = 20,
}

/// 请求消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// 心跳消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hearbeat {
    pub view_number: u64,
    pub sequence_number: u64,
    pub node_id: u64,
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

/// 试图请求消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewRequest {
}

/// 试图请求响应消息（fine）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewResponse {
    pub view_number: u64,
}

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRequest {
}

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateResponse {
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



pub async fn request_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    mut request: Request,
) -> Result<(), String> {
        if client.is_primarry(system_config.view_number) {
            if verify_request(&client.identities[request.node_id as usize].public_key, &mut request)? {
                println!("接收到合法 Request 消息");
                if state.read().await.request_buffer.len() < 2 * (system_config.block_size as usize) {
                    state.write().await.add_request(request);
                    println!("主节点请求缓存区大小：{}", state.read().await.request_buffer.len());
                } else {
                    eprintln!("主节点请求缓冲已满，暂时丢弃该 Request 消息！！！");
                }
                
                let pbft_read = pbft.read().await;
                let state_read = state.read().await;
                if (pbft_read.step == Step::ReceivingPrepare || pbft_read.step == Step::ReceivingCommite) && (get_current_timestamp() - pbft_read.start_time > 1) {
                    pbft.write().await.step = Step::OK;
                }

                if pbft.read().await.step == Step::OK && state_read.request_buffer.len() >= (system_config.block_size as usize) {
                    let mut pbft_write = pbft.write().await;
                    pbft_write.start_time = get_current_timestamp();
                    pbft_write.preprepare = None;
                    pbft_write.prepares.clear();
                    pbft_write.commits.clear();
                    let mut transactions = Vec::new();
                    for request in state_read.request_buffer.iter() {
                        transactions.push(request.transaction.clone());
                    }
                    let mut pre_prepare = PrePrepare {
                        view_number: pbft_write.view_number,
                        sequence_number: pbft_write.sequence_number,
                        digest: Request::digest_requests(&state_read.request_buffer)?,
                        node_id: client.local_node_id,
                        signature: Vec::new(),
                        requests: state_read.request_buffer.clone(),
                        block: state_read.rocksdb.create_block(&transactions)?,
                    };
                    sign_preprepare(&client.private_key, &mut pre_prepare)?;

                    println!("\n发送 PrePrepare 消息");
                    let multicast_addr = format!("{}:{}", system_config.multi_cast_ip, system_config.multi_cast_port).parse::<SocketAddr>().map_err(|e| e.to_string())?;
                    let content = bincode::serialize(&pre_prepare).map_err(|e| e.to_string())?;
                    send_udp_data(&client.local_udp_socket, &multicast_addr, MessageType::PrePrepare, &content).await;
                }
            }
        }

    Ok(())
}

pub async fn preprepare_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    mut preprepare: PrePrepare,
) -> Result<(), String> {
    if !client.is_primarry(system_config.view_number) {
        if verify_preprepare(&client.identities[preprepare.node_id as usize].public_key, &mut preprepare)? {
            println!("接收到合法 PrePrepare 消息");
        }
    }


    Ok(())
}

pub async fn prepare_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    prepare: Prepare,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn commit_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    commit: Commit,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn reply_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    reply: Reply,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn hearbeat_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    mut heartbeat: Hearbeat,
) -> Result<(), String> {
    
    if heartbeat.view_number == system_config.view_number && verify_heartbeat(&client.identities[heartbeat.node_id as usize].public_key, &mut heartbeat)? {
        // println!("接收到合法 Hearbeat 消息");
        reset_sender.send(()).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn view_change_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    reqview_changeuest: ViewChange,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn new_view_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    new_view: NewView,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn view_request_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    view_request: ViewRequest,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn view_response_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    view_response: ViewResponse,
) -> Result<(), String> {
        

    Ok(())
}

pub async fn state_request_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    state_request: StateRequest,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn state_response_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    state_response: StateResponse,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn sync_request_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    sync_request: SyncRequest,
) -> Result<(), String> {
        

    Ok(())
}
pub async fn sync_response_handler(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    sync_response: SyncResponse,
) -> Result<(), String> {
        

    Ok(())
}
