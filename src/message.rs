use crate::config::{ConstantConfig, VariableConfig};
use crate::network::send_udp_data;
use crate::pbft::Step;
use crate::store::{Block, BlockStore, Transaction};
use crate::utils::get_current_timestamp;
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
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    mut request: Request,
) -> Result<(), String> {
        if client.is_primarry(variable_config.read().await.view_number) 
            && verify_request(&client.identities[request.node_id as usize].public_key, &mut request)?
        {
            println!("接收 Request 消息");

            let mut state_write = state.write().await;
            let mut pbft_write = pbft.write().await;

            if state_write.request_buffer.len() < 3 * (constant_config.block_size as usize) {
                state_write.add_request(request);
                println!("主节点请求缓存区大小：{}", state_write.request_buffer.len());
            } else {
                eprintln!("缓冲已满");
            }

            if (pbft_write.step == Step::ReceivingPrepare || pbft_write.step == Step::ReceiveingCommit)
                && (get_current_timestamp().unwrap() - pbft_write.start_time > 1)
            {
                pbft_write.step = Step::OK;
            }

            if pbft_write.step != Step::OK || state_write.request_buffer.len() < constant_config.block_size as usize {
                return Ok(());
            }
            let content = {
                pbft_write.step = Step::ReceivingPrepare;
                pbft_write.start_time = get_current_timestamp().unwrap();
                pbft_write.prepares.clear();
                pbft_write.commits.clear();

                let transactions: Vec<_> = state_write.request_buffer.iter()
                    .map(|req| req.transaction.clone())
                    .collect();

                let mut preprepare = PrePrepare {
                    view_number: pbft_write.view_number,
                    sequence_number: pbft_write.sequence_number,
                    digest: Request::digest_requests(&state_write.request_buffer)?,
                    node_id: client.local_node_id,
                    signature: Vec::new(),
                    requests: state_write.request_buffer.clone(),
                    block: state_write.rocksdb.create_block(&transactions)?,
                };

                sign_preprepare(&client.private_key, &mut preprepare)?;
                pbft_write.preprepare = Some(preprepare.clone());

                let content = bincode::serialize(&preprepare).map_err(|e| e.to_string())?;
                content
            };

            println!("发送 PrePrepare 消息");
            let multicast_addr = constant_config.multi_cast_addr
                .parse::<SocketAddr>()
                .map_err(|e| e.to_string())?;
            send_udp_data(&client.local_udp_socket, &multicast_addr, MessageType::PrePrepare, &content).await;
        }

    Ok(())
}

pub async fn preprepare_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    mut preprepare: PrePrepare,
) -> Result<(), String> {
    if !client.is_primarry(variable_config.read().await.view_number) {
        if verify_preprepare(&client.identities[preprepare.node_id as usize].public_key, &mut preprepare)? {
            println!("接收 PrePrepare 消息");
            reset_sender.send(()).await.unwrap(); // 重置视图切换计时器

            let mut pbft_write = pbft.write().await;

            if (pbft_write.step == Step::ReceivingPrepare || pbft_write.step == Step::ReceiveingCommit)
                && (get_current_timestamp().unwrap() - pbft_write.start_time > 1) 
            {
                pbft_write.step = Step::OK;
            }

            if pbft_write.step != Step::OK {
                return Ok(());
            }
            let content = {
                pbft_write.step = Step::ReceivingPrepare;
                pbft_write.start_time = get_current_timestamp().unwrap();
                pbft_write.preprepare = Some(preprepare.clone());
                pbft_write.prepares.clear();
                pbft_write.commits.clear();
                
                let mut prepare = Prepare {
                    view_number: pbft_write.view_number,
                    sequence_number: pbft_write.sequence_number,
                    digest: preprepare.digest,
                    node_id: client.local_node_id,
                    signature: Vec::new(),
                };
                sign_prepare(&client.private_key, &mut prepare)?;
                pbft_write.prepares.insert(client.local_node_id);

                let content = bincode::serialize(&prepare).map_err(|e| e.to_string())?;
                content
            };

            println!("发送 Prepare 消息");
            let multicast_addr = constant_config.multi_cast_addr
                .parse::<SocketAddr>()
                .map_err(|e| e.to_string())?;
            send_udp_data(&client.local_udp_socket, &multicast_addr, MessageType::Prepare, &content).await;
        }
    }

    Ok(())
}

pub async fn prepare_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    mut prepare: Prepare,
) -> Result<(), String> {
    if verify_prepare(&client.identities[prepare.node_id as usize].public_key, &mut prepare)? {
        println!("接收 Prepare 消息");

        let mut pbft_write = pbft.write().await;

        if pbft_write.step != Step::ReceivingPrepare || pbft_write.prepares.contains(&prepare.node_id) {
            return Ok(())
        }
        pbft_write.prepares.insert(prepare.node_id);

        if pbft_write.prepares.len() < 2 * ((client.identities.len() - 1) / 3) {
            return Ok(())
        }
        pbft_write.step = Step::ReceiveingCommit;
        let content = {
            let mut commit = Commit {
                view_number: pbft_write.view_number,
                sequence_number: pbft_write.sequence_number,
                digest: prepare.digest,
                node_id: client.local_node_id,
                signature: Vec::new(),
            };
            sign_commit(&client.private_key, &mut commit)?;
            pbft_write.commits.insert(client.local_node_id);
            
            let content = bincode::serialize(&commit).map_err(|e| e.to_string())?;
            content
        };

        println!("发送 Commit 消息");
        let multicast_addr = constant_config.multi_cast_addr
            .parse::<SocketAddr>()
            .map_err(|e| e.to_string())?;
        send_udp_data(&client.local_udp_socket, &multicast_addr, MessageType::Commit, &content).await;
    }

    Ok(())
}
pub async fn commit_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    mut commit: Commit,
) -> Result<(), String> {
    if verify_commit(&client.identities[commit.node_id as usize].public_key, &mut commit)? {
        println!("接收 Commit 消息");

        let mut state_write = state.write().await;
        let mut pbft_write = pbft.write().await;
        
        if pbft_write.step != Step::ReceiveingCommit || pbft_write.commits.contains(&commit.node_id) {
            return Ok(())
        }
        pbft_write.commits.insert(commit.node_id);

        if pbft_write.commits.len() < 2 * ((client.identities.len() - 1) / 3) + 1 {
            return Ok(())
        }
        println!("至少 2f + 1 个节点达成共识");

        pbft_write.sequence_number += 1;
        if let Some(preprepare) = &pbft_write.preprepare {
            state_write.rocksdb.put_block(&preprepare.block)?;
            if client.is_primarry(variable_config.read().await.view_number) {
                state_write.request_buffer.drain(0..preprepare.requests.len());
            }
        }
        
        pbft_write.step = Step::OK;
    }

    Ok(())
}
pub async fn reply_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    reply: Reply,
) -> Result<(), String> {
    println!("接收到 Reply 消息");

    Ok(())
}
pub async fn hearbeat_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    mut heartbeat: Hearbeat,
) -> Result<(), String> {
    if heartbeat.view_number == variable_config.read().await.view_number && verify_heartbeat(&client.identities[heartbeat.node_id as usize].public_key, &mut heartbeat)? {
        // println!("接收到合法 Hearbeat 消息");
        reset_sender.send(()).await.map_err(|e| e.to_string())?; // 重置视图切换计时器
    }
    Ok(())
}

pub async fn view_change_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    reqview_changeuest: ViewChange,
) -> Result<(), String> {
    println!("接收到 ViewChange 消息");

    Ok(())
}
pub async fn new_view_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    new_view: NewView,
) -> Result<(), String> {
    println!("接收到 NewView 消息");

    Ok(())
}
pub async fn view_request_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    view_request: ViewRequest,
) -> Result<(), String> {
    println!("接收到 ViewRequest 消息");

    Ok(())
}
pub async fn view_response_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    view_response: ViewResponse,
) -> Result<(), String> {
    println!("接收到 ViewResponse 消息");

    Ok(())
}

pub async fn state_request_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    state_request: StateRequest,
) -> Result<(), String> {
    println!("接收到 StateRequest 消息");

    Ok(())
}
pub async fn state_response_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    state_response: StateResponse,
) -> Result<(), String> {
    println!("接收到 StateReply 消息");

    Ok(())
}
pub async fn sync_request_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    sync_request: SyncRequest,
) -> Result<(), String> {
    println!("接收到 SyncRequest 消息");

    Ok(())
}
pub async fn sync_response_handler(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
    sync_response: SyncResponse,
) -> Result<(), String> {
    println!("接收到 SyncResponse 消息");

    Ok(())
}
