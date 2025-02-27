
use super::message::Request;
use super::store::RocksDBBlockStore;

/// 状态（fine）
pub struct ReplicationState {
    pub request_buffer: Vec<Request>,
    pub rocksdb: RocksDBBlockStore,
}