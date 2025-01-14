use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmrProcessInfo {
    pub pmr_id: u32, // 自增ID
    pub pid: u32,    // 进程PID
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub program: String,
    pub workdir: String,
    pub args: Vec<String>,
}
