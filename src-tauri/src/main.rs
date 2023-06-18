// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{api::path::cache_dir, InvokeError, PackageInfo};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

const SOURCE: &str = "https://api.github.com/repos/zxbc/BAR_widgets/git/trees/main?recursive=1";

struct GithubWidgetSource {
    cache_path: PathBuf,
    response_cache: HashMap<String, String>,
    is_response_cache_loaded: bool,
    client: Client,
}

async fn get_or_create_file(path: &PathBuf) -> Result<File> {
    match File::open(path).await {
        Ok(cache_file) => Ok(cache_file),
        Err(_) => {
            let mut file = File::create(path).await?;
            file.write_all(b"{}").await?;
            Ok(file)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubDirectory {
    sha: String,
    url: String,
    tree: Vec<GithubFile>,
    truncated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubFile {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    item_type: String,
    sha: String,
    size: usize,
    url: String,
}

impl GithubWidgetSource {
    fn new(cache_path: PathBuf) -> Self {
        let client = reqwest::ClientBuilder::new()
            .user_agent("BAR Widget Manager")
            .build()
            .expect("Error creating request client");

        println!("{cache_path:?}");

        GithubWidgetSource {
            cache_path,
            response_cache: HashMap::new(),
            is_response_cache_loaded: false,
            client,
        }
    }

    fn cache_filename(&self) -> PathBuf {
        self.cache_path.join("response_cache.json")
    }

    async fn load_cache(&mut self) -> Result<()> {
        if self.is_response_cache_loaded {
            return Err(anyhow!("Already loaded cache"));
        }

        let mut cache_file = get_or_create_file(&self.cache_filename())
            .await
            .expect("Error initializing Github response cache");

        let mut file_contents = String::new();

        cache_file
            .read_to_string(&mut file_contents)
            .await
            .expect("Error reading Github response cache");

        let response_cache: HashMap<String, String> =
            serde_json::from_str(&file_contents).expect("Error parsing GitHub response cache");

        self.response_cache = response_cache;
        self.is_response_cache_loaded = true;

        Ok(())
    }

    async fn save_cache(&mut self) -> Result<()> {
        let mut cache_file = File::create(&self.cache_filename())
            .await
            .expect("Error retrieving Github response cache");

        let json_data = serde_json::to_string(&self.response_cache)?;

        cache_file.write_all(json_data.as_bytes()).await?;

        Ok(())
    }

    async fn fetch(&mut self, url: &str) -> Result<String> {
        let response = if let Some(cache_result) = self.response_cache.get(url) {
            cache_result.clone()
        } else {
            let response: reqwest::Response = self.client.get(url).send().await?;
            let response_content = response.text().await?;
            self.response_cache
                .insert(url.to_string(), response_content.clone());

            self.save_cache().await?;
            response_content
        };

        Ok(response)
    }

    async fn get_directory(&mut self) -> Result<GithubDirectory> {
        let response = self.fetch(SOURCE).await?;
        let response: GithubDirectory = serde_json::from_str(&response)?;
        Ok(response)
    }
}

fn create_github_widget_source(app_cache_folder_name: &str) -> Result<GithubWidgetSource> {
    let cache_dir = cache_dir().ok_or(anyhow::Error::msg("Error getting cache dir"))?;
    let app_cache_dir = cache_dir.join(app_cache_folder_name);
    Ok(GithubWidgetSource::new(app_cache_dir))
}

fn main() -> Result<()> {
    let context = tauri::generate_context!();
    let github_widget_source = create_github_widget_source(&context.package_info().name)?;

    tauri::Builder::default()
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .manage(context.package_info().clone())
        .manage(Mutex::new(github_widget_source))
        .invoke_handler(tauri::generate_handler![get_all_widgets])
        .run(context)
        .expect("error while running tauri application");

    Ok(())
}

#[tauri::command]
async fn get_all_widgets(
    github_widget_source: tauri::State<'_, Mutex<GithubWidgetSource>>,
) -> Result<GithubDirectory, InvokeError> {
    let mut github_widget_source = github_widget_source.lock().await;

    let response: GithubDirectory = github_widget_source
        .get_directory()
        .await
        .map_err(InvokeError::from_anyhow)?;

    Ok(response)
}
