// src/state.rs
use crate::message::Request;
use crate::store::RocksDBBlockStore;

// ---

/// 状态（fine）
pub struct State {
    pub request_buffer: Vec<Request>,
    pub rocksdb: RocksDBBlockStore,
}
impl State {
    /// 初始化状态
    pub fn new(database_name: &str) -> Result<Self, String> {
        Ok(State {
            request_buffer: Vec::new(),
            rocksdb : RocksDBBlockStore::new(database_name)?,
        })
    }
    
    // ---
    
    /// 添加待处理请求添加到请求缓冲池
    pub fn add_request(&mut self, request: Request) {
        self.request_buffer.push(request);
    }
}