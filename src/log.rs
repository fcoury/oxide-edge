use tokio::io::AsyncWriteExt;
use tracing::error;

pub async fn log(file_name: impl Into<String>, data: &[u8]) {
    let file_name = file_name.into();
    let file = tokio::fs::File::create(format!("dump/{file_name}")).await;
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
