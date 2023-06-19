use std::path::PathBuf;

use anyhow::Result;

use tokio::{fs::File, io::AsyncWriteExt};

pub fn remove_whitespace(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}

pub async fn get_or_create_file(path: &PathBuf) -> Result<File> {
    if (File::open(path).await).is_err() {
        let mut file = File::create(path).await?;
        file.write_all(b"{}").await?;
        file.flush().await?;
    }

    Ok(File::open(path).await?)
}
