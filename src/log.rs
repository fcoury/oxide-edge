use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn log(id: impl Into<String>, kind: &str, name: impl Into<String>, data: &[u8]) {
    let id = id.into();
    let name = name.into();
    let file = tokio::fs::File::create(format!("dump/{id}-{name}.{kind}")).await;
    if let Err(e) = file {
        error!("failed to create file: {}", e);
        return;
    }
    let mut file = file.unwrap();
    let result = file.write_all(data).await;
    if let Err(e) = result {
        error!("failed to write to file: {}", e)
    }
}
