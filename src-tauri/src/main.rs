// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    vec,
};

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use futures::future::join_all;
use github_widget_source::GithubWidgetSource;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{api::path::cache_dir, InvokeError, PackageInfo};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

use crate::github_widget_source::Widget;

mod github_widget_source;

fn create_github_widget_source(app_cache_folder_name: &str) -> Result<GithubWidgetSource> {
    let cache_dir = cache_dir().ok_or(anyhow::Error::msg("Error getting cache dir"))?;
    let app_cache_dir = cache_dir.join(app_cache_folder_name);
    Ok(GithubWidgetSource::new(app_cache_dir))
}

#[tokio::main]
async fn main() -> Result<()> {
    let context = tauri::generate_context!();
    let mut github_widget_source = create_github_widget_source(&context.package_info().name)?;
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
    println!("command:get_all_widgets");
    let mut github_widget_source = github_widget_source.lock().await;

    let response: Vec<Widget> = github_widget_source
        .get_all_widgets()
        .await
        .map_err(InvokeError::from_anyhow)?;

    Ok(response)
}
