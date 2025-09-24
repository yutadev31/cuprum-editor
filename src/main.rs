mod log;

use clap::Parser;
use editor::Editor;

use crate::log::init_logger;

#[derive(Parser)]
struct Cli {
    files: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    init_logger()?;

    let cli = Cli::parse();
    Editor::new(cli.files)?.run()?;

    Ok(())
}
