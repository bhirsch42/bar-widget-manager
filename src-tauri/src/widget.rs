use anyhow::{anyhow, Result};

use mlua::Lua;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Widget {
    filename: String,
    body: String,
    info: Option<WidgetInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WidgetInfo {
    name: Option<String>,
    description: Option<String>,
    author: Option<String>,
    date: Option<String>,
    version: Option<String>,
}

impl Widget {
    pub fn new(filename: &str, body: &str) -> Self {
        Widget {
            filename: filename.to_string(),
            body: body.to_string(),
            info: get_widget_properties_from_lua_code(body),
        }
    }
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

fn get_widget_info_attribute(lua: &Lua, attribute_name: &str) -> Option<String> {
    let code = format!(r#"widget:GetInfo()["{}"]"#, attribute_name);
    lua.load(&code).eval::<String>().ok()
}

fn get_widget_properties_from_lua_code(lua_code: &str) -> Option<WidgetInfo> {
    let widget_info_fn_code = extract_widget_info_lua_fn(lua_code).ok()?;
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

    Some(widget_info)
}
