use api::{CuprumApi, DefaultCuprumApiProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut api = CuprumApi::new(DefaultCuprumApiProvider::default());
    api.change_mode(api::Mode::Insert(false)).await?;

    loop {}
}
