use std::collections::HashMap;

use fsm::*;
use server_state::*;

#[derive(Serialize, Clone)]
pub struct FsmInfo {
    pub regions: Vec<FsmInfoRegion>,
    pub name: &'static str
}

pub struct FsmDataState {
    pub name: &'static str,
    pub structures: HashMap<FsmEventStructKind, ::serde_json::Value>
}

#[derive(Serialize, Clone)]
pub struct PageIndex {
    pub name: &'static str,
    pub fsm: Vec<FsmInfo>
}
