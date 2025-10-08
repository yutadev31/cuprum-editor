use api::{CuprumApi, DefaultCuprumApiProvider, Mode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut api = CuprumApi::new(DefaultCuprumApiProvider::new());
    api.change_mode(Mode::Insert(false)).await?;

    Ok(())
}
