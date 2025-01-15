use super::super::base::process::PmrProcessInfo;
use super::super::config::dump::DumpConfig;
use super::super::config::log_path;
use super::list::list_processes;
use super::start::start_process;
use super::stop::stop_process;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use std::process::Stdio;

pub fn restart_process(
    config: Option<PathBuf>,
    namespace: Option<String>,
    target: Option<String>,
    args: Vec<String>,
) -> io::Result<()> {
    let dump_config = DumpConfig::get_instance();

    // 如果指定了target，先检查是否是已存在的进程
    if let Some(ref target_str) = target {
        if let Ok(processes) = dump_config.list_processes() {
            // 尝试将target解析为pmr_id
            if let Ok(pmr_id) = target_str.parse::<u32>() {
                if let Some(process) = processes.iter().find(|p| p.pmr_id == pmr_id) {
                    restart_existing_process(process)?;
                    return Ok(());
                }
            }

            // 按名称查找进程
            if let Some(process) = processes.iter().find(|p| p.name == *target_str) {
                restart_existing_process(process)?;
                return Ok(());
            }
        }
    }

    // 如果不是重启已存在的进程，就当作普通的启动处理
    start_process(config, namespace, "default".to_string(), target, args)?;
    Ok(())
}

fn restart_existing_process(process: &PmrProcessInfo) -> io::Result<()> {
    println!("正在重启进程 '{}'...", process.name);

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
    let mut cmd = std::process::Command::new(&process.program);
    cmd.args(&process.args)
        .stdout(Stdio::from(stdout_log))
        .stderr(Stdio::from(stderr_log));

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            println!("进程 '{}' 重启成功，新 PID: {}", process.name, pid);

            let dump_config = DumpConfig::get_instance();
            dump_config
                .update_process_status(process.pmr_id, pid, "running".to_string())
                .expect("无法更新进程状态");

            // 增加重启次数
            dump_config
                .increment_restarts(process.pmr_id)
                .expect("无法更新重启次数");

            // 显示进程列表
            list_processes(false);
            Ok(())
        }
        Err(e) => {
            eprintln!("重启进程失败: {}", e);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("重启进程失败: {}", e),
            ))
        }
    }
}
