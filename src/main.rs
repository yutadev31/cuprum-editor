use clap::Parser;

#[derive(Parser)]
struct Cli {
    files: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    Ok(())
}
