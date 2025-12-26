use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct AbyssalCli {
    #[arg(short = 'c', long = "config")]
    pub config_files: Vec<String>,
}
