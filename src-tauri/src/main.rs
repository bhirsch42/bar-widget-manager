// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::vec;

use anyhow::Result;
use github_widget_source::GithubWidgetSource;

use tauri::{api::path::cache_dir, InvokeError};
use tokio::{
    fs::{create_dir, metadata},
    sync::Mutex,
};

use crate::github_widget_source::Widget;

mod github_widget_source;
mod utils;

async fn create_github_widget_source(app_cache_folder_name: &str) -> Result<GithubWidgetSource> {
    let cache_dir = cache_dir().ok_or(anyhow::Error::msg("Error getting cache dir"))?;
    let app_cache_dir = cache_dir.join(app_cache_folder_name);

    println!("create_github_widget_source {:?}", &app_cache_dir);
    if (metadata(&app_cache_dir).await).is_err() {
        create_dir(&app_cache_dir).await?;
    }

    Ok(GithubWidgetSource::new(app_cache_dir))
}

#[tokio::main]
async fn main() -> Result<()> {
    let context = tauri::generate_context!();
    let mut github_widget_source =
        create_github_widget_source(&context.package_info().name).await?;
    github_widget_source.load_cache().await?;

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
) -> Result<Vec<Widget>, InvokeError> {
    let mut github_widget_source = github_widget_source.lock().await;

    let response: Vec<Widget> = github_widget_source
        .get_all_widgets()
        .await
        .map_err(InvokeError::from_anyhow)?;

    Ok(response)
}
