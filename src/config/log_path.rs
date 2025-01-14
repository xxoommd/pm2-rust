use std::fs;
use std::io;
use std::path::PathBuf;
use dirs;

pub fn get_log_path(pmr_id: u32) -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    let log_dir = home_dir.join(".pmr").join("logs");

    // 确保日志目录存在
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }

    Ok(log_dir.join(format!("{}.log", pmr_id)))
}
