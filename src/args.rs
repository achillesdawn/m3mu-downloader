use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {

    #[arg(long)]
    pub url: Option<String>,

    /// path to file
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Path to headers.json file
    #[arg(long)]
    pub headers: Option<PathBuf>,

    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    #[arg(long, value_name = "false")]
    pub concat: bool,


}
