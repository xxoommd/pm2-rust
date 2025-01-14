use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod base;
mod commands;
mod config;
use commands::delete::delete_process;
use commands::list::list_processes;
use commands::restart::restart_process;
use commands::start::start_process;
use commands::stop::stop_process;
use config::dump::DumpConfig;

fn config_init() -> std::io::Result<()> {
    // 使用DumpConfig初始化配置
    let _ = DumpConfig::get_instance();
    Ok(())
}

#[derive(Parser)]
#[command(name = "pmr")]
#[command(about = "Process Manager in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a process
    Start {
        /// Config file path
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Process name
        #[arg(short, long)]
        name: Option<String>,

        /// Namespace for the process
        #[arg(long, default_value = "default")]
        namespace: String,

        /// Target (can be pmr_id, name, or program to run)
        target: Option<String>,

        /// Arguments for the program
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// List running processes
    #[command(alias = "ls")]
    List {
        /// Show all system processes
        #[arg(long)]
        system: bool,
    },

    /// Delete a process
    #[command(alias = "rm")]
    Delete {
        /// Process ID or name
        target: String,
    },

    /// Stop a process
    Stop {
        /// Process ID or name
        target: String,
    },

    /// Restart a process
    Restart {
        /// Config file path
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Namespace for the process
        #[arg(long, default_value = "default")]
        namespace: String,

        /// Target (can be pmr_id, name, or program to run)
        target: Option<String>,

        /// Arguments for the program
        #[arg(last = true)]
        args: Vec<String>,
    },
}

fn main() {
    if let Err(e) = config_init() {
        eprintln!("Failed to initialize .pmr directory: {}", e);
        return;
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            config,
            name,
            namespace,
            target,
            args,
        } => {
            if config.is_none() && target.is_none() {
                eprintln!("错误: 必须指定 --config 或 target");
                return;
            }
            start_process(config, name, namespace, target, args);
        }
        Commands::List { system } => {
            list_processes(system);
        }
        Commands::Delete { target } => {
            delete_process(&target);
        }
        Commands::Stop { target } => {
            stop_process(&target, true);
        }
        Commands::Restart {
            config,
            namespace,
            target,
            args,
        } => {
            if config.is_none() && target.is_none() {
                eprintln!("错误: 必须指定 --config 或 target");
                return;
            }
            restart_process(config, Some(namespace), target, args);
        }
    }
}
