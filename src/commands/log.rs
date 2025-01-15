use super::super::config::dump::DumpConfig;
use super::super::config::log_path;
use ctrlc;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::thread;
use std::time::Duration;

pub fn tail_log(target: String) -> io::Result<()> {
    ctrlc::set_handler(move || {
        println!("\n退出日志查看");
        std::process::exit(0);
    })
    .expect("无法设置Ctrl+C处理器");

    let dump_config = DumpConfig::get_instance();

    // 解析目标ID
    let pmr_id = match target.parse::<u32>() {
        Ok(id) => id,
        Err(_) => {
            // 如果不是数字，尝试按名称查找
            match dump_config.list_processes() {
                Ok(processes) => {
                    if let Some(process) = processes.iter().find(|p| p.name == target) {
                        process.pmr_id
                    } else {
                        eprintln!("找不到进程: {}", target);
                        return Ok(());
                    }
                }
                Err(e) => {
                    eprintln!("无法获取进程列表: {}", e);
                    return Ok(());
                }
            }
        }
    };

    // 获取日志文件路径
    let log_path = match log_path::get_log_path(pmr_id) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("无法获取日志文件路径: {}", e);
            return Ok(());
        }
    };

    // 检查日志文件是否存在
    if !log_path.exists() {
        eprintln!("日志文件不存在: {:?}", log_path);
        return Ok(());
    }

    println!("正在查看日志文件: {:?}", log_path);
    println!("按 Ctrl+C 退出日志查看...");

    // 打开日志文件
    let mut file = match File::open(&log_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("无法打开日志文件: {}", e);
            return Ok(());
        }
    };

    // 移动到文件末尾
    if let Err(e) = file.seek(SeekFrom::End(0)) {
        eprintln!("无法定位到文件末尾: {}", e);
        return Ok(());
    }

    let mut reader = BufReader::new(file);
    let mut buffer = String::new();

    // 持续读取新的日志内容
    loop {
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // 没有新的内容，等待一下
                thread::sleep(Duration::from_millis(100));
            }
            Ok(_) => {
                // 打印新的内容
                print!("{}", buffer);
                buffer.clear();
            }
            Err(e) => {
                eprintln!("读取日志时出错: {}", e);
                break;
            }
        }
    }

    Ok(())
}
