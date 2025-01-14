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

// 将秒数转换为可读的时间格式
fn time_to_readable(seconds: u64) -> String {
    if seconds == 0 {
        return "0s".to_string();
    }

    let days = seconds / (24 * 3600);
    let hours = (seconds % (24 * 3600)) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut result = String::new();
    if days > 0 {
        result.push_str(&format!("{}d ", days));
    }
    if hours > 0 {
        result.push_str(&format!("{}h ", hours));
    }
    if minutes > 0 {
        result.push_str(&format!("{}m ", minutes));
    }
    if secs > 0 && days == 0 && hours == 0 {
        // 只在没有天和小时的时候显示秒
        result.push_str(&format!("{}s", secs));
    }

    result.trim().to_string()
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
    let mut sys = System::new_all();
    let users = Users::new_with_refreshed_list();
    sys.refresh_all();

    if system {
        let processes: Vec<ProcessInfo> = sys
            .processes()
            .iter()
            .map(|(&pid, process)| ProcessInfo {
                id: "0".to_string(),
                name: process.name().to_string_lossy().to_string(),
                namespace: "default".to_string(),
                version: "N/A".to_string(),
                pid: pid.to_string(),
                uptime: time_to_readable(process.run_time()),
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
            .map(|p| {
                let status = if p.pid > 0 {
                    // 检查进程是否真的在运行
                    if let Some(sys_proc) = sys.process(sysinfo::Pid::from(p.pid as usize)) {
                        let run_time = sys_proc.run_time();
                        ProcessInfo {
                            id: p.pmr_id.to_string(),
                            name: p.name.clone(),
                            namespace: p.namespace.clone(),
                            version: "N/A".to_string(),
                            pid: p.pid.to_string(),
                            uptime: time_to_readable(run_time),
                            restarts: "0".to_string(),
                            status: "running".to_string(),
                            cpu: format!("{:.1}%", sys_proc.cpu_usage()),
                            mem: format!("{:.1} MB", sys_proc.memory() as f64 / 1024.0 / 1024.0),
                            user: sys_proc
                                .user_id()
                                .and_then(|uid| users.get_user_by_id(uid))
                                .map_or("N/A".to_string(), |u| u.name().to_string()),
                        }
                    } else {
                        // 进程不存在，但PID > 0，说明进程已经退出
                        ProcessInfo {
                            id: p.pmr_id.to_string(),
                            name: p.name.clone(),
                            namespace: p.namespace.clone(),
                            version: "N/A".to_string(),
                            pid: p.pid.to_string(),
                            uptime: "0s".to_string(),
                            restarts: "0".to_string(),
                            status: "stopped".to_string(),
                            cpu: "0%".to_string(),
                            mem: "0 MB".to_string(),
                            user: "N/A".to_string(),
                        }
                    }
                } else {
                    // 原本就是停止状态
                    ProcessInfo {
                        id: p.pmr_id.to_string(),
                        name: p.name.clone(),
                        namespace: p.namespace.clone(),
                        version: "N/A".to_string(),
                        pid: "0".to_string(),
                        uptime: "0s".to_string(),
                        restarts: "0".to_string(),
                        status: "stopped".to_string(),
                        cpu: "0%".to_string(),
                        mem: "0 MB".to_string(),
                        user: "N/A".to_string(),
                    }
                };
                status
            })
            .collect();

        let table = Table::new(processes).to_string();
        println!("{}", table);
    }
}
