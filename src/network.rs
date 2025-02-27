use super::client::Client;
use super::message::Request;
use super::message::MessageType;
use super::utils::get_current_timestamp;
use super::store::Transaction;
use crate::config::SystemConfig;
use crate::key::sign_request;
use crate::pbft::Pbft;
use crate::state::State;

use tokio::{ net::UdpSocket, sync::Mutex, io::AsyncBufReadExt};
use tokio::time::{interval, Duration, sleep};
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use std::{ collections::{HashMap, HashSet}, fs::File, io::Read, net::{SocketAddr, Ipv4Addr}, sync::Arc, time::{SystemTime, UNIX_EPOCH} };


/// 发送UDP数据
pub async fn send_udp_data(local_udp_socket: &UdpSocket, target_udp_socket: &SocketAddr, message_type: MessageType, content: &[u8]) {
    let mut message = Box::new(Vec::new());
    message.push(message_type as u8);
    message.extend_from_slice(&content);
    local_udp_socket.send_to(&message, target_udp_socket).await.unwrap();
}

/// 任务: 发送命令行指令数据（待修改为Restful API 供本机用户调用）
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
    tx: mpsc::Sender<()>,
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

/// 从节点定时视图切换函数
pub async fn view_change(
    local_udp_socket: Arc<UdpSocket>, 
    node_info: &Client, 
    pbft_state: Arc<RwLock<Pbft>>,
    mut rx: mpsc::Receiver<()>,
) -> Result<(), String> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    interval.reset(); // 确保 interval 不会立即执行
    loop {
        tokio::select! {
            _ = interval.tick() => {
                
            }
            _ = rx.recv() => {
                interval.reset(); // 收到重置信号时，重置定时器
            }
        }
    }
}


// 节点启动发送获取所有节点视图编号函数
pub async fn determining_primary_node(
    local_udp_socket: Arc<UdpSocket>, 
    node_info: &Client, 
    pbft_state: Arc<RwLock<Pbft>>,
) -> Result<(), String> {
    todo!()
}
