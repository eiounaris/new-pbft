use super::message::PrePrepare;

use std::collections::{HashMap, HashSet};

/// PBFT 共识过程（待调整）
#[derive(Debug, PartialEq)]
pub enum Step {
    Initializing = 1,
    Initialized = 2,

    OK = 3,
    PrePrepared = 4,
    Prepared = 5,
    Commited = 6,
}

/// 存储 pbft 共识过程状态信息（待调整）
pub struct Pbft {
    pub view_number: u64,
    pub sended_view_number: u64,
    pub sequence_number: u64,
    pub step: Step,
    pub start_time: u64,
    pub nodes_number: u64,
    pub preprepare: Option<PrePrepare>,
    pub prepares: HashSet<u64>,
    pub commits: HashSet<u64>,
    pub view_change_mutiple_set: HashMap<u64, HashSet<u64>>, 
}