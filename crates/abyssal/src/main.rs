pub mod types;
mod error;
mod cli;

use clap::Parser;
pub use error::{Error, ErrorMeta, Result};

#[tokio::main]
async fn main() -> crate::Result<()> {
    let args = cli::AbyssalCli::parse();
    let config = types::Config::load(args.config_files)?;
    println!("{config:?}");
    Ok(())
}
