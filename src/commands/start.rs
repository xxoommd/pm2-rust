use super::super::base::process::PmrProcessInfo;
use super::super::config::dump::DumpConfig;
use super::list::list_processes;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Deserialize, Serialize)]
struct Config {
    name: String,
    program: String,
    args: Vec<String>,
}

pub fn start_process(
    config: Option<PathBuf>,
    name: Option<String>,
    target: Option<String>,
    args: Vec<String>,
) {
    let dump_config = DumpConfig::get_instance();

    // 如果指定了target，先检查是否是已存在的进程
    if let Some(ref target_str) = target {
        if let Ok(processes) = dump_config.list_processes() {
            // 尝试将target解析为pmr_id
            if let Ok(pmr_id) = target_str.parse::<u32>() {
                if let Some(process) = processes.iter().find(|p| p.pmr_id == pmr_id) {
                    start_existing_process(process);
                    return;
                }
            }

            // 按名称查找进程
            if let Some(process) = processes.iter().find(|p| p.name == *target_str) {
                start_existing_process(process);
                return;
            }
        }
    }

    // 检查是否已存在同名进程
    let process_name = name.clone().unwrap_or_else(|| {
        target.clone().unwrap_or_else(|| {
            if let Some(ref config_path) = config {
                let mut file = File::open(config_path).expect("Failed to open config file");
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Failed to read config file");
                let config: Config =
                    serde_json::from_str(&contents).expect("Failed to parse config file");
                config.name
            } else {
                "unnamed".to_string()
            }
        })
    });

    if let Ok(processes) = dump_config.list_processes() {
        if let Some(_existing) = processes.iter().find(|p| p.name == process_name) {
            println!("\n进程 '{}' 已经存在:", process_name);
            list_processes(false);
            return;
        }
    }

    if let Some(config_path) = config {
        // 从配置文件启动
        let mut file = File::open(config_path).expect("无法打开配置文件");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("无法读取配置文件");

        let config: Config = serde_json::from_str(&contents).expect("无法解析配置文件");

        let mut cmd = Command::new(&config.program);
        cmd.args(&config.args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                println!("启动进程 '{}' PID: {}", process_name, pid);

                dump_config
                    .add_process(
                        process_name,
                        "default".to_string(),
                        config.program,
                        pid,
                        "running".to_string(),
                        config.args,
                    )
                    .expect("无法将进程添加到配置文件");
            }
            Err(e) => {
                eprintln!("启动进程失败: {}", e);
            }
        }
    } else if let Some(target_program) = target {
        // 直接启动程序
        let mut cmd = Command::new(&target_program);
        cmd.args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                println!("启动进程 '{}' PID: {}", process_name, pid);

                dump_config
                    .add_process(
                        process_name,
                        "default".to_string(),
                        target_program,
                        pid,
                        "running".to_string(),
                        args,
                    )
                    .expect("无法将进程添加到配置文件");
            }
            Err(e) => {
                eprintln!("启动进程失败: {}", e);
            }
        }
    } else {
        eprintln!("错误: 必须指定 --config 或 target");
    }
}

fn start_existing_process(process: &PmrProcessInfo) {
    if process.status == "running" {
        println!("进程 '{}' 已经在运行中，PID: {}", process.name, process.pid);
        return;
    }

    let mut cmd = Command::new(&process.program);
    cmd.args(&process.args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            println!("启动进程 '{}' PID: {}", process.name, pid);

            let dump_config = DumpConfig::get_instance();
            dump_config
                .update_process_status(process.pmr_id, pid, "running".to_string())
                .expect("无法更新进程状态");

            // 显示进程列表
            println!("\n当前进程列表:");
            list_processes(false);
        }
        Err(e) => {
            eprintln!("启动进程 '{}' 失败: {}", process.name, e);
        }
    }
}
