use super::super::base::process::PmrProcessInfo;
use super::super::config::dump::DumpConfig;
use super::list::list_processes;
use super::start::start_process;
use super::stop::stop_process;
use std::path::PathBuf;

pub fn restart_process(config: Option<PathBuf>, target: Option<String>, args: Vec<String>) {
    let dump_config = DumpConfig::get_instance();

    // 如果指定了target，先检查是否是已存在的进程
    if let Some(ref target_str) = target {
        if let Ok(processes) = dump_config.list_processes() {
            // 尝试将target解析为pmr_id
            if let Ok(pmr_id) = target_str.parse::<u32>() {
                if let Some(process) = processes.iter().find(|p| p.pmr_id == pmr_id) {
                    restart_existing_process(process);
                    return;
                }
            }

            // 按名称查找进程
            if let Some(process) = processes.iter().find(|p| p.name == *target_str) {
                restart_existing_process(process);
                return;
            }
        }
    }

    // 如果不是重启已存在的进程，就当作普通的启动处理
    start_process(config, None, target, args);
}

fn restart_existing_process(process: &PmrProcessInfo) {
    println!("正在重启进程 '{}'...", process.name);
    
    // 先停止进程
    stop_process(&process.pmr_id.to_string(), false);

    // 重新启动进程
    let mut cmd = std::process::Command::new(&process.program);
    cmd.args(&process.args)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            println!("进程 '{}' 重启成功，新 PID: {}", process.name, pid);

            let dump_config = DumpConfig::get_instance();
            dump_config
                .update_process_status(process.pmr_id, pid, "running".to_string())
                .expect("无法更新进程状态");

            // 显示进程列表
            println!("\n当前进程列表:");
            list_processes(false);
        }
        Err(e) => {
            eprintln!("重启进程 '{}' 失败: {}", process.name, e);
        }
    }
}
