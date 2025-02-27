mod store;
mod network;
mod message;
mod web;
mod key;
mod client;
mod pbft;
mod utils;
mod config;
mod state;

use crate::message::Request;
use crate::store::Transaction;
use crate::utils::get_current_timestamp;
use crate::key::sign_request;
use crate::network::send_udp_data;
use crate::message::MessageType;
use crate::client::Client;
use crate::config::SystemConfig;
use crate::state::State;
use pbft::Pbft;



use tokio::{ net::UdpSocket, sync::Mutex, io::AsyncBufReadExt};
use tokio::time::{interval, Duration, sleep};
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use std::sync::Arc;

/// 任务: 发送命令行指令数据，用于测试tps（待修改为高性能 Restful API 供本机用户调用）
pub async fn send_message(local_udp_socket: Arc<UdpSocket>, client: &Client, system_config: Arc<SystemConfig>) -> Result<(), std::string::String> {
    let stdin = tokio::io::stdin();
    let reader = tokio::io::BufReader::new(stdin);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let mut request = Request {
            transaction: Transaction::Tx0,
            timestamp: get_current_timestamp(),
            node_id: client.local_node_id,
            signature: Vec::new(),
        };
        sign_request(&client.private_key, &mut request)?;
        if line == "test" {
            println!("请输入测试次数：(默认1次）");
            let mut count = String::new();
            std::io::stdin().read_line(&mut count).unwrap();
            let count: u32 = count.trim().parse().unwrap_or(1);

            println!("请输入请求间隔时间（毫秒）：(默认1000毫秒）");
            let mut interval_ms = String::new();
            std::io::stdin().read_line(&mut interval_ms).unwrap();
            let interval_ms: u64 = interval_ms.trim().parse().unwrap_or(1000);
            let interval = Duration::from_millis(interval_ms);

            for i in 0..count {
                send_udp_data(
                    &local_udp_socket,
                    &system_config.multi_cast_socket.parse().unwrap(),
                    MessageType::Request,
                    &bincode::serialize(&request).map_err(|e| e.to_string())?,
                ).await;
                sleep(interval).await;
                println!("第 {} 次请求完成", i + 1);
            }
        } else {
            send_udp_data(
                &local_udp_socket,
                &system_config.multi_cast_socket.parse().unwrap(),
                MessageType::Request,
                &bincode::serialize(&request).map_err(|e| e.to_string())?,
            ).await;
        }
    }
    Ok(())
}


/// 任务: 接收并处理数据
pub async fn handle_message(
    local_udp_socket: Arc<UdpSocket>, 
    clien: &Client, 
    state: Arc<Mutex<State>>, 
    pbft: Arc<Mutex<Pbft>>,
    reset_sender: mpsc::Sender<()>,
) -> Result<(), String> {
    let mut buf = Box::new([0u8; 102400]);
    loop {
        let (udp_data_size, src_socket_addr) = local_udp_socket.recv_from(buf.as_mut_slice()).await.map_err(|e| e.to_string())?;
        // 提取消息类型（第一个字节）
        let message_type = match buf[0] {
            0 => MessageType::Request,
            1 => MessageType::PrePrepare,
            2 => MessageType::Prepare,
            3 => MessageType::Commit,
            4 => MessageType::Reply,
            
            5 => MessageType::Hearbeat,
            6 => MessageType::ViewChange,
            7 => MessageType::NewView,

            8 => MessageType::ViewRequest,
            9 => MessageType::ViewReply,

            10 => MessageType::StateRequest,
            11 => MessageType::StateReply,

            12 => MessageType::SyncRequest,
            13 => MessageType::SyncResponse,
            
            _ => {
                eprintln!("\nReiceive unknown message type");
                continue;
            },
        };

        // 分别处理对应消息
        match message_type {
            // 处理请求消息
            MessageType::Request => {

            },
            MessageType::PrePrepare => {

            },
            MessageType::Prepare => {

            },
            MessageType::Commit => {

            },
            MessageType::Reply => {

            },
            MessageType::Hearbeat => {

            },
            MessageType::ViewChange => {

            },
            MessageType::NewView => {

            },
            MessageType::ViewRequest => {

            },
            MessageType::ViewReply => {

            },
            MessageType::StateRequest => {

            },
            MessageType::StateReply => {

            },
            MessageType::SyncRequest => {

            },
            MessageType::SyncResponse => {

            },
            MessageType::Unknown => {
                eprintln!("\nReiceive unknown message type");
                continue;
            }
        }
    }
}


/// 主节点定时心跳函数
pub async fn primary_heartbeat(
    local_udp_socket: Arc<UdpSocket>, 
    client: &Client, 
    pbft: Arc<Mutex<Pbft>>,
) -> Result<(), String> {
    let mut interval = interval(Duration::from_secs(1));
    interval.reset(); // 确保 interval 不会立即执行
    loop {
        tokio::select! {
            _ = interval.tick() => {
                
            }
        }
    }
}

/// 从节点定时发送视图切换
pub async fn view_change(
    local_udp_socket: Arc<UdpSocket>, 
    client: &Client, 
    pbft: Arc<RwLock<Pbft>>,
    mut reset_receiver: mpsc::Receiver<()>,
) -> Result<(), String> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2)); // 硬编码视图切换时间
    interval.reset(); // 确保 interval 不会立即执行
    loop {
        tokio::select! {
            _ = interval.tick() => {
                
            }
            _ = reset_receiver.recv() => {
                interval.reset();
            }
        }
    }
}


// 节点启动，获取视图编号
pub async fn view_request (
    local_udp_socket: Arc<UdpSocket>, 
    client: &Client, 
    pbft: Arc<RwLock<Pbft>>,
) -> Result<(), String> {
    todo!()
}
