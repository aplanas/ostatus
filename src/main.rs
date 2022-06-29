use std::fs;
use std::path;

use clap::Parser;

/// Track and report of the OS status
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Check the system and log the status
    #[clap(short, long, value_parser)]
    update: bool,

    /// Config file name
    #[clap(short, long, value_parser)]
    config: Option<path::PathBuf>,

    /// Directory where to store the information
    #[clap(short, long, value_parser)]
    directory: Option<String>,
}

fn run() -> ostatus::GenericResult<()> {
    let args = Args::parse();

    let mut cfgs = ostatus::find_configs()?;
    if let Some(config) = args.config {
        cfgs.push(config);
    }

    let roles = ostatus::Roles::from_config(&cfgs)?;

    let status_dir = args
        .directory
        .unwrap_or_else(|| ostatus::STATUS_DIR.to_string());
    if args.update {
        if path::Path::new(&status_dir).exists() {
            fs::remove_dir_all(&status_dir)?;
        }
        ostatus::create_status_file(roles, &status_dir)?;
    }

    // TODO Show the status file.

    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {}", e);
            1
        }
    });
}
