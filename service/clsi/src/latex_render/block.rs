use std::{collections, sync};

use mlua::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Marks {
    #[serde(rename = "type")]
    pub marks_type: sync::Arc<str>,
    #[serde(
        rename = "attrs",
        skip_serializing_if = "collections::BTreeMap::is_empty"
    )]
    pub attrs: collections::BTreeMap<String, String>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Block {
    #[serde(rename = "type")]
    pub content_type: sync::Arc<str>,
    #[serde(
        rename = "attrs",
        skip_serializing_if = "collections::BTreeMap::is_empty"
    )]
    pub attrs: collections::BTreeMap<sync::Arc<str>, sync::Arc<str>>,
    #[serde(rename = "content", skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Self>,
    #[serde(rename = "marks", skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<Marks>,
    #[serde(rename = "text")]
    pub text: sync::Arc<str>,
}

impl Block {
    pub fn to_lua_table(&self, lua: &Lua) -> LuaResult<LuaTable> {
        let table = lua.create_table()?;
        table.set("type", self.content_type.as_ref())?;
        table.set("text", self.text.as_ref())?;

        let attrs_table = lua.create_table()?;
        for (k, v) in &self.attrs {
            attrs_table.set(k.as_ref(), v.as_ref())?;
        }
        table.set("attrs", attrs_table)?;

        let children = lua.create_table()?;
        for (i, child) in self.children.iter().enumerate() {
            children.set(i + 1, child.to_lua_table(lua)?)?;
        }
        table.set("children", children)?;

        let marks_table = lua.create_table()?;
        for (i, mark) in self.marks.iter().enumerate() {
            let mark_tbl = lua.create_table()?;
            mark_tbl.set("type", mark.marks_type.as_ref())?;
            let mark_attrs = lua.create_table()?;
            for (k, v) in &mark.attrs {
                mark_attrs.set(k.as_str(), v.as_str())?;
            }
            mark_tbl.set("attrs", mark_attrs)?;
            marks_table.set(i + 1, mark_tbl)?;
        }
        table.set("marks", marks_table)?;

        Ok(table)
    }
}
