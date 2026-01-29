use anyhow::{Context, Result};
use std::fs;
use std::io::{copy, Cursor};
use std::path::{Path, PathBuf};

pub struct ModelDownloader;

impl ModelDownloader {
    pub fn get_model_dir() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".orunla");
        path.push("models");
        path.push("gliner_small-v2.1");
        path
    }

    pub fn ensure_model_files() -> Result<PathBuf> {
        println!("DEBUG: env::home_dir check...");
        let model_dir = Self::get_model_dir();
        println!("DEBUG: Model dir: {:?}", model_dir);
        let onnx_dir = model_dir.join("onnx");
        if !onnx_dir.exists() {
            println!("DEBUG: Creating dir: {:?}", onnx_dir);
            fs::create_dir_all(&onnx_dir).context("Failed to create onnx directory")?;
        }

        let model_path = onnx_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !model_path.exists() {
            println!("Downloading GliNER model (model.onnx)... This may take a minute.");
            let url = "https://huggingface.co/onnx-community/gliner_small-v2.1/resolve/main/onnx/model.onnx?download=true";
            Self::download_file(url, &model_path).context("Failed to download model.onnx")?;
        }

        if !tokenizer_path.exists() {
            println!("Downloading GliNER tokenizer (tokenizer.json)...");
            let url = "https://huggingface.co/onnx-community/gliner_small-v2.1/resolve/main/tokenizer.json?download=true";
            Self::download_file(url, &tokenizer_path)
                .context("Failed to download tokenizer.json")?;
        }

        Ok(model_dir)
    }

    fn download_file(url: &str, path: &Path) -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .build()?;
        let response = client.get(url).send()?;
        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download file from {}: {}",
                url,
                response.status()
            );
        }
        let mut content = Cursor::new(response.bytes()?);
        let mut file = fs::File::create(path)?;
        copy(&mut content, &mut file)?;
        Ok(())
    }
}
