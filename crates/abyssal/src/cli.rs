use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct AbyssalCli {
    /// Paths to configuration files (TOML)
    /// Files specified later will overwrite those specified earlier
    #[arg(short = 'c', long = "config")]
    pub config_files: Vec<String>,

    /// Path to generate an openapi.json file at.
    /// If omitted (default) no file will be generated.
    #[arg(short, long)]
    pub specification_path: Option<PathBuf>
}
