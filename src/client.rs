use super::config::Identity;

use rsa::{RsaPrivateKey, RsaPublicKey};

/// 节点运行信息
pub struct Client {
    pub local_node_id: u64,
    pub local_socket_addr: std::net::SocketAddr,
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
    pub all_indetitys: Vec<Identity>,
}
impl Client {
    /// 初始化构造函数
    pub fn new(
        local_node_id: u64,
        local_socket_addr: std::net::SocketAddr,
        private_key: RsaPrivateKey,
        public_key: RsaPublicKey,
        all_indetitys: Vec<Identity>,
    ) -> Self {
        Client {
            local_node_id,
            local_socket_addr,
            private_key,
            public_key,
            all_indetitys,
        }
    }

    /// 判断是否为主节点
    pub fn is_primarry(&self, view_number: u64) -> bool {
        view_number % self.all_indetitys.len() as u64 == self.local_node_id
    }
}