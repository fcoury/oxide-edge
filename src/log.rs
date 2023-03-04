use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn log(file_path: impl Into<String>, data: &[u8]) {
    let file_path = file_path.into();
    let file = tokio::fs::File::create(file_path).await;
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
