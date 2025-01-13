use super::super::config::dump::DumpConfig;
use super::list::list_processes;
use std::process::Command;

pub fn stop_process(target: &str, show_list: bool) {
    let dump_config = DumpConfig::get_instance();
    match dump_config.list_processes() {
        Ok(processes) => {
            let mut found = false;

            // 首先尝试将target解析为pmr_id
            if let Ok(pmr_id) = target.parse::<u32>() {
                if let Some(process) = processes.iter().find(|p| p.pmr_id == pmr_id) {
                    if process.pid > 0 {
                        // 根据操作系统使用不同的命令终止进程
                        let output = if cfg!(target_os = "windows") {
                            Command::new("taskkill")
                                .args(&["/PID", &process.pid.to_string(), "/F"])
                                .output()
                        } else {
                            Command::new("kill")
                                .args(&["-9", &process.pid.to_string()])
                                .output()
                        }.expect("无法执行进程终止命令");

                        if output.status.success() {
                            println!("已停止进程 '{}' (PID: {})", process.name, process.pid);
                            dump_config
                                .update_process_status(process.pmr_id, 0, "stopped".to_string())
                                .expect("无法更新进程状态");
                            found = true;
                        } else {
                            eprintln!(
                                "停止进程失败 '{}' (PID: {}): {}",
                                process.name,
                                process.pid,
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                    } else {
                        println!("进程 '{}' 已经停止", process.name);
                        found = true;
                    }
                }
            }

            // 如果不是pmr_id，尝试按name查找
            if !found {
                if let Some(process) = processes.iter().find(|p| p.name == target) {
                    if process.pid > 0 {
                        // 根据操作系统使用不同的命令终止进程
                        let output = if cfg!(target_os = "windows") {
                            Command::new("taskkill")
                                .args(&["/PID", &process.pid.to_string(), "/F"])
                                .output()
                        } else {
                            Command::new("kill")
                                .args(&["-9", &process.pid.to_string()])
                                .output()
                        }.expect("无法执行进程终止命令");

                        if output.status.success() {
                            println!("已停止进程 '{}' (PID: {})", process.name, process.pid);
                            dump_config
                                .update_process_status(process.pmr_id, 0, "stopped".to_string())
                                .expect("无法更新进程状态");
                        } else {
                            eprintln!(
                                "停止进程失败 '{}' (PID: {}): {}",
                                process.name,
                                process.pid,
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                    } else {
                        println!("进程 '{}' 已经停止", process.name);
                    }
                } else {
                    eprintln!("未找到进程: {}", target);
                }
            }

            // 根据show_list参数决定是否显示进程列表
            if show_list {
                println!("\n当前进程列表:");
                list_processes(false);
            }
        }
        Err(e) => {
            eprintln!("读取进程列表失败: {}", e);
        }
    }
}
