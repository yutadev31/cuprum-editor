mod log;

use clap::Parser;

use crate::log::init_logger;

#[derive(Parser)]
struct Cli {
    files: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;

    let cli = Cli::parse();
    editor::main(cli.files).await?;

    Ok(())
}
