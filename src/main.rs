mod log;

use clap::Parser;
use editor::EditorApplication;

use crate::log::init_logger;

#[derive(Parser)]
struct Cli {
    files: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;

    let cli = Cli::parse();
    EditorApplication::main(cli.files).await?;

    Ok(())
}
