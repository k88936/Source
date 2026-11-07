//! One page of a Cloudreve folder, listed through the same use case the album
//! picker calls, printed as JSON.
//!
//! `cargo run -p source-core --example list_remote_folders -- cloudreve://my`

use std::sync::Arc;

use anyhow::Result;
use source_core::app::folders::RemoteFolderPage;
use source_core::{CloudreveStorage, FoldersService, SqliteCatalog, UploadSessionPort};

#[path = "example_utils/mod.rs"]
mod example_utils;

use example_utils::env::Settings;
use example_utils::login::sign_in;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let settings = Settings::from_env()?;

    let catalog = SqliteCatalog::open(&settings.catalog)
        .await
        .map_err(|cause| anyhow::anyhow!("打开索引失败: {cause:?}"))?;
    let sessions: Arc<dyn UploadSessionPort> = Arc::new(catalog.clone());
    let auth = sign_in(&settings).await?;
    let uri = std::env::args().nth(1).unwrap_or(settings.root);
    let cloud = CloudreveStorage::new(Arc::new(auth), sessions);
    let folders = FoldersService::new(cloud, catalog);

    let page: RemoteFolderPage = folders
        .list_remote_folders(&uri, None)
        .await
        .map_err(|cause| anyhow::anyhow!("列目录失败: {cause:?}"))?;
    println!("{}", serde_json::to_string(&page)?);
    Ok(())
}
