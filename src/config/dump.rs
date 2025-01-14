use super::super::base::process::PmrProcessInfo;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{fs, io, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DumpData {
    processes: Vec<PmrProcessInfo>,
}

pub struct DumpConfig {
    path: PathBuf,
    data: Mutex<DumpData>,
}

static INSTANCE: OnceCell<DumpConfig> = OnceCell::new();

impl DumpConfig {
    fn new() -> io::Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;

        let config_dir = home_dir.join(".pmr");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let dump_file = config_dir.join("dump.json");
        let data = if dump_file.exists() {
            let file_contents = fs::read_to_string(&dump_file)?;
            // 使用 serde_json::Value 先解析JSON
            let json: serde_json::Value = serde_json::from_str(&file_contents)?;
            
            // 手动构建进程列表
            let processes = if let Some(processes) = json.get("processes").and_then(|v| v.as_array()) {
                processes
                    .iter()
                    .map(|p| PmrProcessInfo {
                        pmr_id: p["pmr_id"].as_u64().unwrap_or(0) as u32,
                        pid: p["pid"].as_u64().unwrap_or(0) as u32,
                        name: p["name"].as_str().unwrap_or("").to_string(),
                        namespace: p["namespace"].as_str().unwrap_or("").to_string(),
                        status: p["status"].as_str().unwrap_or("").to_string(),
                        program: p["program"].as_str().unwrap_or("").to_string(),
                        workdir: p["workdir"].as_str().unwrap_or("").to_string(),
                        args: p["args"]
                            .as_array()
                            .map(|a| {
                                a.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(String::from)
                                    .collect()
                            })
                            .unwrap_or_default(),
                        restarts: p["restarts"].as_u64().unwrap_or(0) as u32,
                    })
                    .collect()
            } else {
                Vec::new()
            };

            DumpData { processes }
        } else {
            let initial_data = DumpData {
                processes: Vec::new(),
            };
            fs::write(&dump_file, serde_json::to_string_pretty(&initial_data)?)?;
            initial_data
        };

        Ok(Self {
            path: dump_file,
            data: Mutex::new(data),
        })
    }

    pub fn get_instance() -> &'static DumpConfig {
        INSTANCE.get_or_init(|| Self::new().expect("Failed to initialize DumpConfig"))
    }

    fn save_data(&self, data: &DumpData) -> io::Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        fs::write(&self.path, json)
    }

    pub fn add_process(
        &self,
        name: String,
        namespace: String,
        workdir: String,
        program: String,
        pid: u32,
        status: String,
        args: Vec<String>,
    ) -> io::Result<()> {
        let mut data = self.data.lock().unwrap();
        let new_id = data.processes.iter().map(|p| p.pmr_id).max().unwrap_or(0) + 1;

        data.processes.push(PmrProcessInfo {
            pmr_id: new_id,
            name,
            namespace,
            pid,
            status,
            workdir,
            program,
            args,
            restarts: 0, // 初始化重启次数为0
        });

        self.save_data(&data)
    }

    pub fn delete_process(&self, id: u32) -> io::Result<()> {
        let mut data = self.data.lock().unwrap();
        data.processes.retain(|p| p.pmr_id != id);
        self.save_data(&data)
    }

    pub fn list_processes(&self) -> io::Result<Vec<PmrProcessInfo>> {
        let data = self.data.lock().unwrap();
        Ok(data.processes.clone())
    }

    pub fn update_process_status(&self, pmr_id: u32, pid: u32, status: String) -> io::Result<()> {
        let mut data = self.data.lock().unwrap();
        if let Some(process) = data.processes.iter_mut().find(|p| p.pmr_id == pmr_id) {
            process.pid = pid;
            process.status = status;
            self.save_data(&data)
        } else {
            Ok(())
        }
    }

    pub fn increment_restarts(&self, pmr_id: u32) -> io::Result<()> {
        let mut data = self.data.lock().unwrap();
        if let Some(process) = data.processes.iter_mut().find(|p| p.pmr_id == pmr_id) {
            process.restarts = process.restarts.saturating_add(1);
            self.save_data(&data)
        } else {
            Ok(())
        }
    }
}
