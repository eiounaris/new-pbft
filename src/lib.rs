#![allow(dead_code, unused_variables)]

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

use store::{BlockStore, Transaction};
use utils::get_current_timestamp;
use key::{load_private_key, load_public_key, sign_heartbeat, sign_request};
use network::send_udp_data;
use message::{MessageType, Request, PrePrepare, Prepare, Commit,Reply, Hearbeat, ViewChange, NewView, ViewRequest, ViewResponse, StateRequest, StateResponse, SyncRequest, SyncResponse};
use client::Client;
use config::{SystemConfig, Identity};
use state::State;
use pbft::{Pbft, Step};

use tokio::{ net::UdpSocket, io::AsyncBufReadExt};
use tokio::time::{interval, Duration, sleep};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use dotenv::dotenv;

use std::env;
use std::sync::Arc;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::net::SocketAddr;


/// PBFT 初始化函数（fine）
pub async fn init() -> Result<(Arc<SystemConfig>, Arc<Client>, Arc<RwLock<State>>, Arc<RwLock<Pbft>>, tokio::sync::mpsc::Sender<()>, tokio::sync::mpsc::Receiver<()>), String> {
    // 加载环境变量
    dotenv().ok();
    let local_node_id = env::var("local_node_id").map_err(|e| e.to_string())?.parse::<u64>().map_err(|e| e.to_string())?;
    let identity_config_path = env::var("identity_config_path").map_err(|e| e.to_string())?;
    let system_config_path = env::var("system_config_path").map_err(|e| e.to_string())?;
    let private_key_path = env::var("private_key_path").map_err(|e| e.to_string())?;
    let public_key_path = env::var("public_key_path").map_err(|e| e.to_string())?;
    println!("{local_node_id:?}, {identity_config_path:?}, {system_config_path:?}, {private_key_path:?}, {public_key_path:?}");
    // 加载初始信息
    let identities = Identity::load_identity(identity_config_path).await?;
    let system_config = SystemConfig::load_system_config(system_config_path).await?;
    let private_key = load_private_key(&private_key_path).await?;
    let public_key = load_public_key(&public_key_path).await?;
    let local_identitiy = identities.iter().find(|identity| identity.node_id == local_node_id).unwrap_or(&identities[0]);
    // 创建广播套接字
    let udp_socket = UdpSocket::bind(format!("{}:{}", "0.0.0.0", system_config.multi_cast_port)).await.map_err(|e| e.to_string())?;
    // let multicast_addr = Ipv4Addr::from_str(&system_config.multi_cast_ip).map_err(|e| e.to_string())?;
    let multicast_addr = Ipv4Addr::from_str(&system_config.multi_cast_ip).unwrap();
    let interface  = Ipv4Addr::new(0,0,0,0);
    udp_socket.join_multicast_v4(multicast_addr, interface ).map_err(|e| e.to_string())?;
    udp_socket.set_multicast_loop_v4(false).map_err(|e| e.to_string())?;
    let udp_socket = Arc::new(udp_socket);
    // 输出本地节点初始化信息
    println!("\n本地节点 {} 启动，地址：{}", local_node_id, format!("{}:{}", local_identitiy.ip, local_identitiy.port));
    // 创建 client
    let client = Client::new(local_node_id, udp_socket.clone(), private_key, public_key, identities);
    // 创建 state
    let state = State::new(&system_config.database_name)?;
    // 创建 pbft
    let pbft = Pbft::new(system_config.view_number, state.rocksdb.get_last_block()?.unwrap().index, client.identities.len() as u64);
    // 创建一个通道用于发送重置信号
    let (reset_sender, reset_receiver) = tokio::sync::mpsc::channel(1);

    Ok((Arc::new(system_config), Arc::new(client), Arc::new(RwLock::new(state)), Arc::new(RwLock::new(pbft)), reset_sender, reset_receiver))
}


/// 任务: 发送命令行指令数据，用于测试tps（待修改为高性能 Restful API 供本机用户调用）
pub async fn send_message(client: Arc<Client>, system_config: Arc<SystemConfig>) -> Result<(), std::string::String> {
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
        println!("{:?}", key::verify_request(&client.public_key, &mut request)?);
        if line == "test" {
            print!("请输入测试次数(默认1次）：");
            let mut count = String::new();
            std::io::stdin().read_line(&mut count).unwrap();
            let count: u32 = count.trim().parse().unwrap_or(1);

            print!("请输入请求间隔时间（毫秒）(默认1000毫秒）：");
            let mut interval_ms = String::new();
            std::io::stdin().read_line(&mut interval_ms).unwrap();
            let interval_ms: u64 = interval_ms.trim().parse().unwrap_or(1000);
            let interval = Duration::from_millis(interval_ms);

            for i in 0..count {
                send_udp_data(
                    &client.local_udp_socket,
                    &format!("{}:{}", system_config.multi_cast_ip, system_config.multi_cast_port).parse().map_err(|e: std::net::AddrParseError| e.to_string())?,
                    MessageType::Request,
                    &bincode::serialize(&request).map_err(|e| e.to_string())?,
                ).await;
                sleep(interval).await;
                println!("第 {} 次请求完成", i + 1);
            }
        } else {
            send_udp_data(
                &client.local_udp_socket,
                &format!("{}:{}", system_config.multi_cast_ip, system_config.multi_cast_port).parse().map_err(|e: std::net::AddrParseError| e.to_string())?,
                MessageType::Request,
                &bincode::serialize(&request).map_err(|e| e.to_string())?,
            ).await;
            let mut request = bincode::deserialize::<Request>(&bincode::serialize(&request).unwrap()).unwrap();
            if crate::key::verify_request(&client.identities[request.node_id as usize].public_key, &mut request)? {
                println!("debug`")
            }
        }
    }
    Ok(())
}


/// 任务: 接收并处理数据
pub async fn handle_message(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
) -> Result<(), String> {
    let mut buf = Box::new([0u8; 102400]);
    loop {
        let (udp_data_size, src_socket_addr) = client.local_udp_socket.recv_from(buf.as_mut_slice()).await.map_err(|e| e.to_string())?;
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
            9 => MessageType::ViewResponse,

            10 => MessageType::StateRequest,
            11 => MessageType::StateResponse,

            12 => MessageType::SyncRequest,
            13 => MessageType::SyncResponse,
            
            _ => {
                eprintln!("\nReiceive unknown message type");
                continue;
            },
        };

        // 提取消息内容（剩余的字节）
        let content = &buf[1..udp_data_size];

        // 分别处理对应消息
        match message_type {
            // 处理请求消息
            MessageType::Request => {
                println!("接收到 Request 消息");
                if let Ok(request) = bincode::deserialize::<Request>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::request_handler(system_config, client, state, pbft, reset_sender, request).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::PrePrepare => {
                println!("接收到 PrePrepare 消息");
                if let Ok(preprepare) = bincode::deserialize::<PrePrepare>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::preprepare_handler(system_config, client, state, pbft, reset_sender, preprepare).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::Prepare => {
                println!("接收到 Prepare 消息");
                if let Ok(prepare) = bincode::deserialize::<Prepare>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::prepare_handler(system_config, client, state, pbft, reset_sender, prepare).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::Commit => {
                println!("接收到 Commit 消息");
                if let Ok(commit) = bincode::deserialize::<Commit>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::commit_handler(system_config, client, state, pbft, reset_sender, commit).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::Reply => {
                println!("接收到 Reply 消息");
                if let Ok(reply) = bincode::deserialize::<Reply>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::reply_handler(system_config, client, state, pbft, reset_sender, reply).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::Hearbeat => {
                // println!("接收到 Hearbeat 消息");
                if let Ok(hearbeat) = bincode::deserialize::<Hearbeat>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::hearbeat_handler(system_config, client, state, pbft, reset_sender, hearbeat).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::ViewChange => {
                println!("接收到 ViewChange 消息");
                if let Ok(view_change) = bincode::deserialize::<ViewChange>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::view_change_handler(system_config, client, state, pbft, reset_sender, view_change).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::NewView => {
                println!("接收到 NewView 消息");
                if let Ok(new_view) = bincode::deserialize::<NewView>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::new_view_handler(system_config, client, state, pbft, reset_sender, new_view).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::ViewRequest => {
                println!("接收到 ViewRequest 消息");
                if let Ok(view_request) = bincode::deserialize::<ViewRequest>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::view_request_handler(system_config, client, state, pbft, reset_sender, view_request).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::ViewResponse => {
                println!("接收到 ViewResponse 消息");
                if let Ok(view_response) = bincode::deserialize::<ViewResponse>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::view_response_handler(system_config, client, state, pbft, reset_sender, view_response).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::StateRequest => {
                println!("接收到 StateRequest 消息");
                if let Ok(state_request) = bincode::deserialize::<StateRequest>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::state_request_handler(system_config, client, state, pbft, reset_sender, state_request).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::StateResponse => {
                println!("接收到 StateReply 消息");
                if let Ok(state_response) = bincode::deserialize::<StateResponse>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::state_response_handler(system_config, client, state, pbft, reset_sender, state_response).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::SyncRequest => {
                println!("接收到 SyncRequest 消息");
                if let Ok(sync_request) = bincode::deserialize::<SyncRequest>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::sync_request_handler(system_config, client, state, pbft, reset_sender, sync_request).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::SyncResponse => {
                println!("接收到 SyncResponse 消息");
                if let Ok(sync_response) = bincode::deserialize::<SyncResponse>(content).map_err(|e| e.to_string()) {
                    tokio::spawn({
                        let system_config: Arc<SystemConfig> = system_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::sync_response_handler(system_config, client, state, pbft, reset_sender, sync_response).await {
                                eprintln!("\n{e:?}");
                            }
                        }
                    });
                }
            },
            MessageType::Unknown => {
                eprintln!("\nReiceive unknown message type");
                continue;
            }
        }
    }
}


/// 主节点定时心跳函数
pub async fn heartbeat(
    system_config : Arc<SystemConfig>,
    client: Arc<Client>, 
    pbft: Arc<RwLock<Pbft>>,
) -> Result<(), String> {
    let mut interval = interval(Duration::from_secs(1)); // 硬编码心跳间隔
    interval.reset(); // 确保 interval 不会立即执行
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if client.is_primarry(system_config.view_number) {
                    // println!("主节点发送 Hearbeat 消息");
                    let mut heartbeat = Hearbeat {
                        view_number: system_config.view_number,
                        sequence_number: pbft.read().await.sequence_number,
                        node_id: client.local_node_id,
                        signature: Vec::new(),
                    };
                    sign_heartbeat(&client.private_key, &mut heartbeat)?;
                    send_udp_data(
                        &client.local_udp_socket,
                        &format!("{}:{}", system_config.multi_cast_ip, system_config.multi_cast_port).parse().map_err(|e: std::net::AddrParseError| e.to_string())?,
                        MessageType::Hearbeat,
                        &bincode::serialize(&heartbeat).map_err(|e| e.to_string())?,
                    ).await;
                }
            }
        }
    }
}

/// 从节点定时视图切换函数
pub async fn view_change(
    system_config: Arc<SystemConfig>,
    client: Arc<Client>, 
    pbft: Arc<RwLock<Pbft>>,
    mut reset_receiver: mpsc::Receiver<()>,
) -> Result<(), String> {
    let mut interval = tokio::time::interval(Duration::from_secs(2)); // 硬编码视图切换时间
    interval.reset(); // 确保 interval 不会立即执行
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if !client.is_primarry(system_config.view_number) {
                    println!("从节点发送 ViewChange 消息")
                }
            }
            _ = reset_receiver.recv() => {
                interval.reset();
            }
        }
    }
}


// 节点启动，获取视图编号（fine）
pub async fn view_request (
    system_config: Arc<SystemConfig>,
    client: Arc<Client>, 
    pbft: Arc<RwLock<Pbft>>,
) -> Result<(), String> {
    sleep(Duration::from_secs(1)).await; // 硬编码，一秒之后获取视图编号
    println!("发送 ViewRequest 消息");
    let multicast_addr = format!("{}:{}", system_config.multi_cast_ip, system_config.multi_cast_port).parse::<SocketAddr>().map_err(|e| e.to_string())?;
    let content = Vec::new();        
    send_udp_data(&client.local_udp_socket, &multicast_addr, MessageType::ViewRequest, &content).await;
    sleep(Duration::from_secs(1)).await; // 硬编码，一秒之后切换状态
    if pbft.read().await.step == Step::ReceivingViewResponse {
        let mut pbft = pbft.write().await;
        pbft.view_change_mutiple_set.clear();
        pbft.step = Step::OK
    }
    Ok(())
}
