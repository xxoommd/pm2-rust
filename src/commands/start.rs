use super::super::base::process::PmrProcessInfo;
use super::super::config::log_path;
use super::super::config::dump::DumpConfig;
use super::list::list_processes;
use super::stop::stop_process;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
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
    namespace: String,
    target: Option<String>,
    args: Vec<String>,
) -> io::Result<()> {
    let dump_config = DumpConfig::get_instance();
    // 获取当前工作目录
    let workdir = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .to_string_lossy()
        .to_string();

    // 获取进程名称
    let process_name = name.unwrap_or_else(|| {
        target
            .as_ref()
            .map(|s| s.split('/').last().unwrap_or(s))
            .unwrap_or("unknown")
            .to_string()
    });

    // 如果指定了target，先检查是否是已存在的进程
    if let Some(ref target_str) = target {
        if let Ok(processes) = dump_config.list_processes() {
            // 尝试将target解析为pmr_id
            if let Ok(pmr_id) = target_str.parse::<u32>() {
                if let Some(process) = processes.iter().find(|p| p.pmr_id == pmr_id) {
                    start_existing_process(process)?;
                    return Ok(());
                }
            }

            // 按名称查找进程
            if let Some(process) = processes.iter().find(|p| p.name == *target_str) {
                start_existing_process(process)?;
                return Ok(());
            }
        }
    }

    // 如果指定了配置文件，从配置文件启动
    if let Some(config_path) = config {
        if !config_path.exists() {
            eprintln!("配置文件不存在: {:?}", config_path);
            return Ok(());
        }

        // 读取配置文件
        let mut file = File::open(&config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = serde_json::from_str(&contents).expect("无法解析配置文件");

        let new_id = dump_config.add_process(
            process_name.clone(),
            namespace.clone(),
            workdir.clone(),
            config.program.clone(),
            0,
            "starting".to_string(),
            config.args.clone(),
        )?;

        // 获取日志文件路径
        let log_path = log_path::get_log_path(new_id)?;

        // 打开日志文件（追加模式）
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        // 使用同一个文件句柄来重定向标准输出和标准错误
        let stdout_log = log_file.try_clone()?;
        let stderr_log = log_file.try_clone()?;

        // 启动进程
        let mut cmd = Command::new(&config.program);
        cmd.args(&config.args)
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log));

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                println!("启动进程 '{}' PID: {}", process_name, pid);

                dump_config
                    .update_process_status(new_id, pid, "running".to_string())
                    .expect("无法更新进程状态");

                list_processes(false);
            }
            Err(e) => {
                eprintln!("启动进程失败: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("启动进程失败: {}", e),
                ));
            }
        }
    } else if let Some(target_program) = target {
        // 直接启动程序
        let new_id = dump_config.add_process(
            process_name.clone(),
            namespace.clone(),
            workdir.clone(),
            target_program.clone(),
            0,
            "starting".to_string(),
            args.clone(),
        )?;

        // 获取日志文件路径
        let log_path = log_path::get_log_path(new_id)?;

        // 打开日志文件（追加模式）
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        // 使用同一个文件句柄来重定向标准输出和标准错误
        let stdout_log = log_file.try_clone()?;
        let stderr_log = log_file.try_clone()?;

        // 启动进程
        let mut cmd = Command::new(&target_program);
        cmd.args(&args)
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log));

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                println!("启动进程 '{}' PID: {}", process_name, pid);

                dump_config
                    .update_process_status(new_id, pid, "running".to_string())
                    .expect("无法更新进程状态");
                list_processes(false);
            }
            Err(e) => {
                eprintln!("启动进程失败: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("启动进程失败: {}", e),
                ));
            }
        }
    } else {
        eprintln!("错误: 必须指定 --config 或 target");
    }

    Ok(())
}

fn start_existing_process(process: &PmrProcessInfo) -> io::Result<()> {
    if process.status == "running" {
        println!("进程 '{}' 已经在运行中，PID: {}", process.name, process.pid);
        return Ok(());
    }

    // 先停止进程
    stop_process(&process.pmr_id.to_string(), false);

    // 获取日志文件路径
    let log_path = log_path::get_log_path(process.pmr_id)?;

    // 打开日志文件（追加模式）
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // 使用同一个文件句柄来重定向标准输出和标准错误
    let stdout_log = log_file.try_clone()?;
    let stderr_log = log_file.try_clone()?;

    // 重新启动进程
    let mut cmd = Command::new(&process.program);
    cmd.args(&process.args)
        .stdout(Stdio::from(stdout_log))
        .stderr(Stdio::from(stderr_log));

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            println!("启动进程 '{}' PID: {}", process.name, pid);

            let dump_config = DumpConfig::get_instance();
            dump_config
                .update_process_status(process.pmr_id, pid, "running".to_string())
                .expect("无法更新进程状态");

            // 显示进程列表
            list_processes(false);
            Ok(())
        }
        Err(e) => {
            eprintln!("启动进程失败: {}", e);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("启动进程失败: {}", e),
            ))
        }
    }
}
