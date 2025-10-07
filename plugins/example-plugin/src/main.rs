use std::time::Duration;

use api::{CuprumApi, DefaultCuprumApiProvider};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut api = CuprumApi::new(DefaultCuprumApiProvider::default());
    api.change_mode(api::Mode::Insert(false)).await?;

    sleep(Duration::from_millis(5000)).await;
    Ok(())
}
