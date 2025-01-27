use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Sets a custom config.toml file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub url: Option<String>,

    /// Path to headers.json file
    #[arg(long)]
    pub headers: Option<PathBuf>,

    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    #[arg(long, value_name = "false")]
    pub concat: bool,
}
