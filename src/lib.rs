#![allow(dead_code, unused_variables)]

pub mod store;
pub mod network;
pub mod message;
// pub mod restapi;
pub mod key;
pub mod client;
pub mod pbft;
pub mod utils;
pub mod config;
pub mod state;

use store::{BlockStore, Transaction};
use utils::get_current_timestamp;
use key::{load_private_key, load_public_key, sign_heartbeat, sign_request, sign_new_view};
use network::send_udp_data;
use message::{MessageType, Request, PrePrepare, Prepare, Commit,Reply, Hearbeat, ViewChange, NewView, ViewRequest, ViewResponse, StateRequest, StateResponse, SyncRequest, SyncResponse};
use client::Client;
use config::{ConstantConfig, VariableConfig, Identity};
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
pub async fn init() -> Result<(
    Arc<ConstantConfig>, 
    Arc<RwLock<VariableConfig>>, 
    Arc<Client>, Arc<RwLock<State>>, 
    Arc<RwLock<Pbft>>, 
    tokio::sync::mpsc::Sender<()>, 
    tokio::sync::mpsc::Receiver<()>
), String> {

    // 加载环境变量
    dotenv().ok();
    let local_node_id = env::var("local_node_id")
        .map_err(|e| e.to_string())?
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    let identity_config_path = env::var("identity_config_path")
        .map_err(|e| e.to_string())?;
    let constant_config_path = env::var("constant_config_path")
        .map_err(|e| e.to_string())?;
    let variable_config_path = env::var("variable_config_path")
        .map_err(|e| e.to_string())?;
    let private_key_path = env::var("private_key_path")
        .map_err(|e| e.to_string())?;
    let public_key_path = env::var("public_key_path")
        .map_err(|e| e.to_string())?;

    // 加载初始信息
    let identities = Identity::load_identity_config(identity_config_path).await?;
    let constant_config = ConstantConfig::load_constant_config(constant_config_path).await?;
    let variable_config = VariableConfig::load_variable_config(variable_config_path).await?;
    let private_key = load_private_key(&private_key_path).await?;
    let public_key = load_public_key(&public_key_path).await?;
    let local_identitiy = identities.iter()
        .find(|identity| identity.node_id == local_node_id)
        .ok_or_else(|| "节点身份文件缺失当前节点信息！！！")?;

    // 创建广播套接字
    let udp_socket = UdpSocket::bind(format!("{}:{}", "0.0.0.0", constant_config.multi_cast_port)).await
        .map_err(|e| e.to_string())?;
    let multicast_addr = Ipv4Addr::from_str(&constant_config.multi_cast_ip)
        .map_err(|e| e.to_string())?;
    let interface  = Ipv4Addr::new(0,0,0,0);
    udp_socket.join_multicast_v4(multicast_addr, interface )
        .map_err(|e| e.to_string())?;
    udp_socket.set_multicast_loop_v4(false)
        .map_err(|e| e.to_string())?;

    // 输出本地节点初始化信息
    println!("本地节点 {} 启动，地址：{}", local_node_id, format!("{}:{}", local_identitiy.ip, local_identitiy.port));

    // 创建 client
    let client = Client::new(local_node_id, udp_socket, private_key, public_key, identities);
    // 创建 state
    let state = State::new(&constant_config.database_name)?;
    // 创建 pbft
    let pbft = Pbft::new(variable_config.view_number, state.rocksdb.get_last_block()?.index);
    // 创建 channel
    let (reset_sender, reset_receiver) = tokio::sync::mpsc::channel(1);

    // 返回初始化信息
    Ok((
        Arc::new(constant_config), 
        Arc::new(RwLock::new(variable_config)), 
        Arc::new(client), 
        Arc::new(RwLock::new(state)), 
        Arc::new(RwLock::new(pbft)), 
        reset_sender, 
        reset_receiver
    ))
}









// 节点启动，获取视图编号（fine）
pub async fn view_request (
    constant_config : Arc<ConstantConfig>,
    client: Arc<Client>, 

    pbft: Arc<RwLock<Pbft>>,
) -> Result<(), String> {

    println!("广播 ViewRequest 消息");

    let multicast_addr = constant_config.multi_cast_addr
        .parse::<SocketAddr>()
        .map_err(|e| e.to_string())?;

    send_udp_data(
        &client.local_udp_socket, 
        &multicast_addr, 
        MessageType::ViewRequest, 
        &Vec::new()
    ).await?;

    sleep(Duration::from_secs(1)).await; // 硬编码，一秒之后检查状态

    let mut pbft_write = pbft.write().await;

    if pbft_write.step == Step::ReceivingViewResponse
        || pbft_write.step == Step::ReceivingStateResponse
        || pbft_write.step == Step::ReceivingSyncResponse
    {
        println!("当前状态为：{:?}，区块链同步可能存在异常", pbft_write.step);

        pbft_write.step = Step::Ok
    }

    Ok(())
}












/// 主节点定时心跳函数
pub async fn heartbeat(
    constant_config : Arc<ConstantConfig>,
    client: Arc<Client>, 

    variable_config : Arc<RwLock<VariableConfig>>,
    pbft: Arc<RwLock<Pbft>>,
) -> Result<(), String> {

    let mut interval = interval(Duration::from_secs(1)); // 硬编码心跳间隔
    interval.reset(); // 确保 interval 不会立即执行

    loop {
        tokio::select! {
            _ = interval.tick() => {

                let pbft_read = pbft.read().await;

                if client.is_primarry(pbft_read.view_number) {
                    // println!("主节点发送 Hearbeat 消息");
                    let mut heartbeat = Hearbeat {
                        view_number: pbft_read.view_number,
                        sequence_number: pbft_read.sequence_number,
                        node_id: client.local_node_id,
                        signature: Vec::new(),
                    };

                    sign_heartbeat(&client.private_key, &mut heartbeat)?;

                    send_udp_data(
                        &client.local_udp_socket,
                       &constant_config.multi_cast_addr.parse().map_err(|e: std::net::AddrParseError| e.to_string())?,
                        MessageType::Hearbeat,
                        &bincode::serialize(&heartbeat).map_err(|e| e.to_string())?,
                    ).await?;
                }
            }
        }
    }
}












/// 从节点定时视图切换函数
pub async fn view_change(
    constant_config : Arc<ConstantConfig>,
    client: Arc<Client>, 

    variable_config : Arc<RwLock<VariableConfig>>,
    pbft: Arc<RwLock<Pbft>>,

    mut reset_receiver: mpsc::Receiver<()>,
) -> Result<(), String> {

    let mut interval = tokio::time::interval(Duration::from_secs(2)); // 硬编码视图切换时间
    interval.reset(); // 确保 interval 不会立即执行

    loop {
        tokio::select! {
            _ = interval.tick() => {

                let mut pbft_write = pbft.write().await;

                if !client.is_primarry(pbft_write.view_number) {

                    if pbft_write.step == Step::ReceivingViewResponse
                        || pbft_write.step == Step::ReceivingStateResponse
                        || pbft_write.step == Step::ReceivingSyncResponse
                    {
                        return Ok(())
                    }
                    
                    if pbft_write.step != Step::NoPrimary && pbft_write.step != Step::ReceivingViewChange {
                        pbft_write.step = Step::NoPrimary;
                        continue
                    }

                    if pbft_write.step == Step::ReceivingViewChange && (get_current_timestamp().unwrap() - pbft_write.start_time > 1) {
                        pbft_write.step = Step::NoPrimary;
                    }

                    if pbft_write.step != Step::NoPrimary {
                        continue
                    }

                    drop(pbft_write);

                    let num: u64 = rand::random::<u64>() % 1000; // 生成 0 到 1000 之间的随机整数
                    sleep(Duration::from_millis(num)).await;

                    let mut pbft_write = pbft.write().await;

                    if pbft_write.step != Step::NoPrimary {
                        continue
                    }

                    pbft_write.step = Step::ReceivingViewChange;
                    pbft_write.start_time = get_current_timestamp().unwrap();
                    pbft_write.view_change_collect_map.clear();
                    pbft_write.new_view_number = pbft_write.view_number + client.local_node_id;

                    println!("从节点发送 NewView 消息");

                    let mut new_view = NewView {
                        view_number: pbft_write.view_number,
                        sequence_number: pbft_write.sequence_number,
                        node_id: client.local_node_id,
                        signature: Vec::new()
                    };

                    drop(pbft_write);

                    sign_new_view(&client.private_key, &mut new_view)?;

                    send_udp_data(
                        &client.local_udp_socket,
                       &constant_config.multi_cast_addr.parse().map_err(|e: std::net::AddrParseError| e.to_string())?,
                        MessageType::NewView,
                        &bincode::serialize(&new_view).map_err(|e| e.to_string())?,
                    ).await?;
                }
            }
            _ = reset_receiver.recv() => {
                interval.reset();
            }
        }
    }
}














/// 任务: 发送命令行指令数据，用于测试tps（待修改为高性能 Restful API 供本机用户调用）
pub async fn send_message(
    constant_config: Arc<ConstantConfig>, 
    client: Arc<Client>, 

    state: Arc<RwLock<State>>
) -> Result<(), String> {

    let stdin = tokio::io::stdin();
    let reader = tokio::io::BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() == 1 && parts[0] == "last" {
            let block = state.read().await.rocksdb.get_last_block()?;
            println!("索引：{:?}, 前哈希：{:?}，哈希 ：{:?}", block.index, block.previous_hash, block.hash);
            continue;
        }

        if parts.len() == 2 && parts[0] == "index" {
            let Ok(index) = parts[1].parse::<u64>() else { continue };
            let Some(block) = state.read().await.rocksdb.get_block_by_index(index)? else {continue};
            println!("索引：{:?}, 前哈希：{:?}，哈希 ：{:?}", block.index, block.previous_hash, block.hash);
            continue;
        }

        let mut request = Request {
            transaction: Transaction::Tx0,
            timestamp: get_current_timestamp().unwrap(),
            node_id: client.local_node_id,
            signature: Vec::new(),
        };

        sign_request(&client.private_key, &mut request)?;

        let multicast_addr = constant_config.multi_cast_addr
            .parse::<SocketAddr>()
            .map_err(|e| e.to_string())?;
        
        let content: Vec<u8> = bincode::serialize(&request)
            .map_err(|e| e.to_string())?;
        
        if parts.len() == 3 && parts[0] == "test" {
            
            let Ok(count) = parts[1].parse::<u64>() else { continue };
            let Ok(interval_us) = parts[2].parse::<u64>() else { continue };
            let interval = Duration::from_micros(interval_us);

            let old_index = state.read().await.rocksdb.get_last_block()?.index;

            let multicast_addr = Arc::new(multicast_addr);
            let content = Arc::new(content);

            for _ in 0..(count+99)/100 {
                tokio::spawn({
                    let multicast_addr = multicast_addr.clone();
                    let client = client.clone();
                    let content = content.clone();
                    async move {
                        for _ in 0..100 {
                            send_udp_data(
                                &client.local_udp_socket,
                                &multicast_addr,
                                MessageType::Request,
                                &content,
                            ).await.unwrap();

                            sleep(interval).await;
                        }
                    }
                });
                
                sleep(interval * 100).await;
            }

            sleep(Duration::from_secs(1)).await;

            let end_block = state.read().await.rocksdb.get_last_block()?;
            
            if let Some(begin_block) = state.read().await.rocksdb.get_block_by_index(old_index + 1)? {
                println!("begin_index: {}, end_index: {}", begin_block.index, end_block.index);
                println!("begin_timestamp: {}, end_timestamp: {}", begin_block.timestamp, end_block.timestamp);
                println!("blocksize: {}", constant_config.block_size);
                println!("tps = {}", (
                    end_block.index - begin_block.index) as f64  
                    * end_block.transactions.len() as f64 
                    / (end_block.timestamp - begin_block.timestamp + 1) as f64
                );
            } else {
                eprintln!("测试未生成新区快");
            }
        } else {
            send_udp_data(
                &client.local_udp_socket,
                &multicast_addr,
                MessageType::Request,
                &content,
            ).await?;
        }
    }

    Ok(())
}


















/// 任务: 接收并处理数据
pub async fn handle_message(
    constant_config : Arc<ConstantConfig>,
    variable_config : Arc<RwLock<VariableConfig>>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
    pbft: Arc<RwLock<Pbft>>,
    reset_sender: mpsc::Sender<()>,
) -> Result<(), String> {

    let mut buf = Box::new([0u8; 102400]);
    
    loop {
        let (udp_data_size, src_socket_addr) = client.local_udp_socket
            .recv_from(buf.as_mut_slice()).await
            .map_err(|e| e.to_string())?;

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
                eprintln!("Reiceive unknown message type");
                continue;
            },
        };

        // 提取消息内容（剩余的字节）
        let content = &buf[1..udp_data_size];

        // 分别处理对应消息
        match message_type {

            MessageType::Request => {
                if let Ok(request) = 
                    bincode::deserialize::<Request>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::request_handler(
                                constant_config, 
                                client, 
                                variable_config, 
                                state, 
                                pbft, 
                                reset_sender, 
                                request
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::PrePrepare => {
                if let Ok(preprepare) = 
                    bincode::deserialize::<PrePrepare>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::preprepare_handler(
                                constant_config, 
                                client, 

                                variable_config, 
                                state, 
                                pbft, 
                                reset_sender, 
                                preprepare
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::Prepare => {
                if let Ok(prepare) = 
                    bincode::deserialize::<Prepare>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::prepare_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                prepare
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::Commit => {
                if let Ok(commit) = 
                    bincode::deserialize::<Commit>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::commit_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                commit
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::Reply => {
                if let Ok(reply) = 
                    bincode::deserialize::<Reply>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::reply_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                reply
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::Hearbeat => {
                if let Ok(hearbeat) = 
                    bincode::deserialize::<Hearbeat>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::hearbeat_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                hearbeat
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::ViewChange => {
                if let Ok(view_change) = 
                    bincode::deserialize::<ViewChange>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::view_change_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                view_change
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::NewView => {
                if let Ok(new_view) = 
                    bincode::deserialize::<NewView>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::new_view_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                new_view
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::ViewRequest => {
                if let Ok(view_request) = 
                    bincode::deserialize::<ViewRequest>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::view_request_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                view_request, 
                                src_socket_addr
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::ViewResponse => {
                if let Ok(view_response) = 
                    bincode::deserialize::<ViewResponse>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::view_response_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                view_response, 
                                src_socket_addr
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },
            
            MessageType::StateRequest => {
                if let Ok(state_request) = 
                    bincode::deserialize::<StateRequest>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::state_request_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                state_request,
                                src_socket_addr
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::StateResponse => {
                if let Ok(state_response) = 
                    bincode::deserialize::<StateResponse>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::state_response_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                state_response,
                                src_socket_addr
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::SyncRequest => {
                if let Ok(sync_request) = 
                    bincode::deserialize::<SyncRequest>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::sync_request_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                sync_request,
                                src_socket_addr,
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::SyncResponse => {
                if let Ok(sync_response) = 
                    bincode::deserialize::<SyncResponse>(content).map_err(|e| e.to_string()) 
                {
                    tokio::spawn({
                        let constant_config = constant_config.clone();
                        let variable_config = variable_config.clone();
                        let client = client.clone();
                        let state = state.clone();
                        let pbft = pbft.clone();
                        let reset_sender = reset_sender.clone();
                        async move {
                            if let Err(e) = message::sync_response_handler(
                                constant_config, 
                                variable_config, 
                                client, 
                                state, 
                                pbft, 
                                reset_sender, 
                                sync_response
                            ).await {
                                eprintln!("{e:?}");
                            }
                        }
                    });
                }
            },

            MessageType::Unknown => {
                eprintln!("Reiceive unknown message type");
                continue;
            }
        }
    }
}