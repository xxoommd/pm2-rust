use super::super::config::dump::DumpConfig;
use super::list::list_processes;
use super::stop::stop_process;

pub fn delete_process(target: &str) {
    let dump_config = DumpConfig::get_instance();
    match dump_config.list_processes() {
        Ok(processes) => {
            let mut found = false;

            // 首先尝试将target解析为pmr_id
            if let Ok(pmr_id) = target.parse::<u32>() {
                if let Some(process) = processes.iter().find(|p| p.pmr_id == pmr_id) {
                    // 如果进程还在运行，先停止它
                    if process.pid > 0 {
                        stop_process(target, false);
                    }

                    // 从配置中删除进程
                    dump_config
                        .delete_process(pmr_id)
                        .expect("Failed to delete process from dump file");
                    println!("Successfully deleted process '{}'", process.name);
                    found = true;
                }
            }

            // 如果不是pmr_id，尝试按name查找
            if !found {
                if let Some(process) = processes.iter().find(|p| p.name == target) {
                    // 如果进程还在运行，先停止它
                    if process.pid > 0 {
                        stop_process(&process.pmr_id.to_string(), false);
                    }

                    // 从配置中删除进程
                    dump_config
                        .delete_process(process.pmr_id)
                        .expect("Failed to delete process from dump file");
                    println!("Successfully deleted process '{}'", process.name);
                    found = true;
                }
            }

            if found {
                // 显示进程列表
                println!("\nCurrent process list:");
                list_processes(false);
            } else {
                eprintln!("No process found with id or name: {}", target);
            }
        }
        Err(e) => {
            eprintln!("Failed to read processes: {}", e);
        }
    }
}
