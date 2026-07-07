use mlua::prelude::*;

use crate::latex_render::block::Block;

pub fn render_with_lua(script: &str, block: &Block, config: &str, title: &str) -> Result<String, String> {
    let lua = Lua::new();

    lua.load(script)
        .exec()
        .map_err(|e| format!("Failed to load lua script: {}", e))?;

    let render_latex: LuaFunction = lua
        .globals()
        .get("render_latex")
        .map_err(|e| format!("Lua script must define 'render_latex' function: {}", e))?;

    let block_table = block
        .to_lua_table(&lua)
        .map_err(|e| format!("Failed to convert block to lua table: {}", e))?;

    let config_value = json_to_lua_value(&lua, config)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    // Merge title into config table so lua can access config.title
    let config_value = if !title.is_empty() {
        let config_table = match config_value {
            LuaValue::Table(t) => t,
            LuaValue::Nil => lua.create_table().map_err(|e| e.to_string())?,
            _ => return Err("config must be a JSON object or null".to_string()),
        };
        config_table
            .set("title", lua.create_string(title).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        LuaValue::Table(config_table)
    } else {
        config_value
    };

    let result: String = render_latex
        .call((block_table, config_value))
        .map_err(|e| format!("Failed to call render_latex: {}", e))?;

    Ok(result)
}

fn json_to_lua_value<'lua>(lua: &'lua Lua, json_str: &str) -> Result<LuaValue, String> {
    if json_str.trim().is_empty() {
        return Ok(LuaValue::Nil);
    }
    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    serde_json_to_lua(lua, &value)
}

fn serde_json_to_lua<'lua>(lua: &'lua Lua, value: &serde_json::Value) -> Result<LuaValue, String> {
    match value {
        serde_json::Value::Null => Ok(LuaValue::Nil),
        serde_json::Value::Bool(b) => Ok(LuaValue::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Err(format!("Unsupported number: {}", n))
            }
        }
        serde_json::Value::String(s) => Ok(LuaValue::String(
            lua.create_string(s).map_err(|e| e.to_string())?,
        )),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table().map_err(|e| e.to_string())?;
            for (i, v) in arr.iter().enumerate() {
                table
                    .set(i + 1, serde_json_to_lua(lua, v).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            }
            Ok(LuaValue::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table().map_err(|e| e.to_string())?;
            for (k, v) in obj {
                table
                    .set(
                        k.as_str(),
                        serde_json_to_lua(lua, v).map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| e.to_string())?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}
