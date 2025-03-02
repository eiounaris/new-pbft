use crate::config::ConstantConfig;
use crate::store::BlockStore;
use crate::utils::get_current_timestamp;
use crate::key::sign_request;
use crate::network::send_udp_data;
use crate::message::MessageType;
use crate::message::Request;
use crate::client::Client;
use crate::state::State;
use crate::store::Transaction;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tokio::sync::RwLock;

use std::net::SocketAddr;
use std::sync::Arc;

struct AppState {
    constant_config: Arc<ConstantConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>,
}

// 实现 RESTful API 路由配置
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 获取最新区块
        .route("/last_block", web::get().to(get_last_block))
        // 获取单个索引区块
        .route("/block/{index}", web::get().to(get_block_by_index))
        // 创建新区块
        .route("/send_request", web::post().to(send_request));
}

// 以下是各个 endpoint 的处理函数框架
// GET /last_block
async fn get_last_block(data: web::Data<AppState>) -> impl Responder {
    // 这里应返回全部区块数据
    let state_read = data.state.read().await;
    let last_block = state_read.rocksdb.get_last_block().unwrap().unwrap();
    HttpResponse::Ok().json(&last_block)
}

// GET /block/{index}
async fn get_block_by_index(
    index: web::Path<u64>,
    data: web::Data<AppState>,
) -> impl Responder {
    let state_read = data.state.read().await;
    let found_block = state_read.rocksdb.get_block_by_index(index.into_inner()).unwrap();
    match found_block {
        Some(block) => {
            HttpResponse::Ok().json(&block)
        },
        None => {
            HttpResponse::NotFound().body("查无区块")
        }
    }
}

// POST /send_request
async fn send_request(
    transaction: web::Json<Transaction>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut request = Request {
        transaction: transaction.into_inner(),
        timestamp: get_current_timestamp().unwrap(),
        node_id: data.client.local_node_id,
        signature: Vec::new(),
    };
    sign_request(&data.client.private_key, &mut request).unwrap();
    
    let multicast_addr = data.constant_config.multi_cast_addr
        .parse::<SocketAddr>().unwrap();
    let content = bincode::serialize(&request).unwrap();

    send_udp_data(
        &data.client.local_udp_socket,
        &multicast_addr,
        MessageType::Request,
        &content,
    ).await;
    HttpResponse::Created().json(true)
}

pub async fn actix_web_runweb_run(
    constant_config: Arc<ConstantConfig>,
    client: Arc<Client>, 
    state: Arc<RwLock<State>>, 
) -> Result<(), String>{
    // 初始化内存存储
    let app_state = web::Data::new(AppState {
        constant_config,
        client,
        state,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes)
    })
        .bind("0.0.0.0:8080").unwrap()
        .run()
        .await.map_err(|e| e.to_string())?;
    
    Ok(())
}






/*
curl http://localhost:8080/last

curl http://localhost:8080/block/index

curl -X POST http://localhost:8080/block \
  -H "Content-Type: application/json" \
  -d '"Operation1"'
*/
