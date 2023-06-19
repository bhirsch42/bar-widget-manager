use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use futures::future::join_all;
use mlua::Lua;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

use crate::utils::remove_whitespace;

const SOURCE: &str = "https://api.github.com/repos/zxbc/BAR_widgets/git/trees/main?recursive=1";

pub struct GithubWidgetSource {
    cache_path: PathBuf,
    response_cache: HashMap<String, String>,
    is_response_cache_loaded: bool,
    client: Client,
}

async fn get_or_create_file(path: &PathBuf) -> Result<File> {
    if (File::open(path).await).is_err() {
        let mut file = File::create(path).await?;
        file.write_all(b"{}").await?;
        file.flush().await?;
    }

    Ok(File::open(path).await?)
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

#[derive(Debug, Serialize, Deserialize)]

struct GithubBlob {
    sha: String,
    node_id: String,
    size: i32,
    content: String,
    encoding: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Widget {
    filename: String,
    body: String,
    info: Option<WidgetInfo>,
}

impl GithubWidgetSource {
    pub fn new(cache_path: PathBuf) -> Self {
        let client = reqwest::ClientBuilder::new()
            .user_agent("BAR Widget Manager")
            .build()
            .expect("Error creating request client");

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

    pub async fn load_cache(&mut self) -> Result<()> {
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
            println!("fetch (cached): {:?}", url);
            cache_result.clone()
        } else {
            println!("fetch: {:?}", url);
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

    pub async fn get_all_widgets(&mut self) -> Result<Vec<Widget>> {
        let directory = self.get_directory().await?;

        let source_mutex = Mutex::new(self);

        let widgets = join_all(
            directory
                .tree
                .iter()
                .filter(|github_file| is_lua_filename(&github_file.path))
                .map(|github_file| async {
                    let response = source_mutex.lock().await.fetch(&github_file.url).await?;
                    let response: GithubBlob = serde_json::from_str(&response)?;

                    let lua_code =
                        decode_base64(&response.content).context("Error decoding base64")?;

                    let info = get_widget_properties_from_lua_code(&lua_code);

                    Ok::<Widget, anyhow::Error>(Widget {
                        filename: github_file.path.clone(),
                        body: lua_code,
                        info,
                    })
                }),
        )
        .await;

        let widgets: Result<Vec<Widget>> = widgets.into_iter().collect();
        let widgets: Vec<Widget> = widgets?;

        Ok(widgets)
    }
}

fn is_lua_filename(filename: &str) -> bool {
    let path = Path::new(filename);
    let extension = path.extension();
    match extension {
        Some(extension) => extension.to_string_lossy() == "lua",
        None => false,
    }
}

fn decode_base64(encoded_str: &str) -> Result<String> {
    let decoded_bytes =
        base64::engine::general_purpose::STANDARD.decode(remove_whitespace(encoded_str))?;
    Ok(String::from_utf8(decoded_bytes).expect("Invalid UTF-8"))
}

fn extract_widget_info_lua_fn(lua_code: &str) -> Result<String> {
    let lines: Vec<&str> = lua_code.split('\n').collect();

    let start_index = lines
        .iter()
        .position(|line| *line == "function widget:GetInfo()")
        .ok_or(anyhow!("Couldn't find widget:GetInfo()"))?;

    let end_index = lines[start_index..]
        .iter()
        .position(|line| *line == "end")
        .ok_or(anyhow!("Couldn't find end of widget:GetInfo()"))?;

    let widget_info_def = lines[start_index..(end_index + 1)].join("\n");

    Ok(widget_info_def)
}

#[derive(Debug, Serialize, Deserialize)]
struct WidgetInfo {
    name: Option<String>,
    description: Option<String>,
    author: Option<String>,
    date: Option<String>,
    version: Option<String>,
}

fn get_widget_info_attribute(lua: &Lua, attribute_name: &str) -> Option<String> {
    let code = format!(r#"widget:GetInfo()["{}"]"#, attribute_name);
    lua.load(&code).eval::<String>().ok()
}

fn get_widget_properties_from_lua_code(lua_code: &str) -> Option<WidgetInfo> {
    let widget_info_fn_code = extract_widget_info_lua_fn(lua_code).ok()?;

    println!("{}", widget_info_fn_code);

    let lua = Lua::new();
    let globals = lua.globals();

    let empty_table = lua.create_table().ok()?;
    globals.set("widget", empty_table).ok()?;
    lua.load(&widget_info_fn_code).exec().ok()?;

    let widget_info = WidgetInfo {
        name: get_widget_info_attribute(&lua, "name"),
        description: get_widget_info_attribute(&lua, "desc"),
        author: get_widget_info_attribute(&lua, "author"),
        date: get_widget_info_attribute(&lua, "date"),
        version: get_widget_info_attribute(&lua, "version"),
    };

    println!("{:#?}", widget_info);

    Some(widget_info)
}
