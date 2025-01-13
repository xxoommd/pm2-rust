use crate::config::dump::DumpConfig;
use serde::{Deserialize, Serialize};
use sysinfo::{System, Users};
use tabled::{Table, Tabled};

#[derive(Serialize, Deserialize)]
pub struct PmrProcess {
    pub pmr_id: u32,
    pub pid: u32,
    pub name: String,
    pub namespace: String,
    pub program: String,
    pub args: Vec<String>,
    pub status: String,
}

#[derive(Tabled)]
struct ProcessInfo {
    id: String,
    name: String,
    namespace: String,
    version: String,
    pid: String,
    uptime: String,
    restarts: String,
    status: String,
    cpu: String,
    mem: String,
    user: String,
}

pub fn read_pmr_processes() -> Vec<PmrProcess> {
    let dump_config = DumpConfig::get_instance();
    match dump_config.list_processes() {
        Ok(processes) => processes
            .into_iter()
            .map(|p| PmrProcess {
                pmr_id: p.pmr_id,
                pid: p.pid,
                name: p.name,
                namespace: p.namespace,
                program: p.program,
                args: p.args,
                status: p.status,
            })
            .collect(),
        Err(e) => {
            eprintln!("Failed to read processes: {}", e);
            Vec::new()
        }
    }
}

pub fn list_processes(system: bool) {
    if system {
        let mut sys = System::new_all();
        let users = Users::new_with_refreshed_list();
        sys.refresh_all();

        let processes: Vec<ProcessInfo> = sys
            .processes()
            .iter()
            .map(|(&pid, process)| ProcessInfo {
                id: "0".to_string(),
                name: process.name().to_string_lossy().to_string(),
                namespace: "default".to_string(),
                version: "N/A".to_string(),
                pid: pid.to_string(),
                uptime: process.run_time().to_string(),
                restarts: "0".to_string(),
                status: process.status().to_string(),
                cpu: format!("{:.1}%", process.cpu_usage()),
                mem: format!("{:.1} MB", process.memory() as f64 / 1024.0 / 1024.0),
                user: process
                    .user_id()
                    .and_then(|uid| users.get_user_by_id(uid))
                    .map_or("N/A".to_string(), |u| u.name().to_string()),
            })
            .collect();

        let table = Table::new(processes).to_string();
        println!("{}", table);
    } else {
        let pmr_processes = read_pmr_processes();
        let processes: Vec<ProcessInfo> = pmr_processes
            .iter()
            .map(|p| ProcessInfo {
                id: p.pmr_id.to_string(),
                name: p.name.clone(),
                namespace: "pmr".to_string(),
                version: "N/A".to_string(),
                pid: p.pid.to_string(),
                uptime: "N/A".to_string(),
                restarts: "0".to_string(),
                status: if p.pid > 0 {
                    "running".to_string()
                } else {
                    "stopped".to_string()
                },
                cpu: "N/A".to_string(),
                mem: "N/A".to_string(),
                user: "N/A".to_string(),
            })
            .collect();

        let table = Table::new(processes).to_string();
        println!("{}", table);
    }
}
