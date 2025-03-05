// src/pbft.rs
use crate::message::PrePrepare;
use crate::utils::get_current_timestamp;

use std::collections::{HashMap, HashSet};

// ---

/// PBFT 共识过程（待调整）
#[derive(PartialEq, Debug)]
pub enum Step {
    ReceivingViewResponse = 0,
    ReceivingStateResponse = 1,
    ReceivingSyncResponse = 2,

    NoPrimary = 9, 
    ReceivingViewChange = 4,

    Ok = 5,
    ReceivingPrepare = 6,
    ReceiveingCommit = 7,
}

/// 存储 pbft 共识过程状态信息（待调整）
pub struct Pbft {
    pub view_number: u64,
    pub sequence_number: u64,
    pub step: Step,
    pub start_time: u64,
    pub preprepare: Option<PrePrepare>,
    pub prepares: HashSet<u64>,
    pub commits: HashSet<u64>,
    pub new_view_number: u64,
    pub view_change_collect_map: HashMap<u64, HashSet<u64>>,
}
impl Pbft {
    /// 初始化pbft共识状态
    pub fn new(
        view_number: u64,
        sequence_number: u64,
    ) -> Self {
        Pbft {
            view_number: view_number,
            sequence_number: sequence_number,
            step: Step::ReceivingViewResponse,
            start_time: get_current_timestamp().unwrap(),
            preprepare: None,
            prepares: HashSet::new(),
            commits: HashSet::new(),
            view_change_collect_map: HashMap::new(), // 考虑使用 HashSet::new(),
            new_view_number: view_number,
        }
    }
}