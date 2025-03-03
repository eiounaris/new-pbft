use crate::message::PrePrepare;
use crate::utils::get_current_timestamp;

use std::collections::{HashMap, HashSet};

/// PBFT 共识过程（待调整）
#[derive(PartialEq, Debug)]
pub enum Step {
    Initing = 8,
    
    ReceivingViewResponse = 0,
    ReceivingStateResponse = 1,
    ReceiveingSyncResponse = 2,

    NoPrimary = 9,
    ViewChanging = 3,
    ReceivingViewChange = 4,

    Ok = 5,
    ReceivingPrepare = 6,
    ReceiveingCommit = 7,
}

/// 存储 pbft 共识过程状态信息（待调整）
pub struct Pbft {
    pub view_number: u64,
    pub sended_view_number: u64,
    pub sequence_number: u64,
    pub step: Step,
    pub start_time: u64,
    pub preprepare: Option<PrePrepare>,
    pub prepares: HashSet<u64>,
    pub commits: HashSet<u64>,
    pub view_change_mutiple_set: HashMap<u64, HashSet<u64>>,
    pub new_view_number: u64,
}
impl Pbft {
    /// 初始化pbft共识状态
    pub fn new(
        view_number: u64,
        sequence_number: u64,
    ) -> Self {
        Pbft {
            view_number: view_number,
            sended_view_number: view_number,
            sequence_number: sequence_number,
            step: Step::Initing,
            start_time: get_current_timestamp().unwrap(),
            preprepare: None,
            prepares: HashSet::new(),
            commits: HashSet::new(),
            view_change_mutiple_set: HashMap::new(),
            new_view_number: view_number,
        }
    }
}