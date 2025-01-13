use mlua::prelude::*;
use std::process::Command;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Clone)]
struct CargoAdd {
    runtime: Arc<Runtime>,
}

impl CargoAdd {
    fn new() -> LuaResult<Self> {
        Ok(Self {
            runtime: Arc::new(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?,
            ),
        })
    }

    async fn add_dependency(&self, crate_name: &str, version: Option<&str>) -> LuaResult<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("add").arg(crate_name);

        if let Some(ver) = version {
            cmd.arg("--version").arg(ver);
        }

        let output = cmd
            .output()
            .map_err(|e| LuaError::RuntimeError(format!("Failed to execute cargo add: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(LuaError::RuntimeError(format!(
                "cargo add failed: {}",
                error
            )));
        }

        Ok(())
    }

    async fn search_crates(&self, query: &str) -> LuaResult<Vec<String>> {
        let client = reqwest::Client::new();
        let url = format!("https://crates.io/api/v1/crates?q={}&per_page=10", query);

        let response =
            client.get(&url).send().await.map_err(|e| {
                LuaError::RuntimeError(format!("Failed to search crates.io: {}", e))
            })?;

        #[derive(serde::Deserialize)]
        struct CratesResponse {
            crates: Vec<Crate>,
        }

        #[derive(serde::Deserialize)]
        struct Crate {
            name: String,
            max_version: String,
        }

        let crates: CratesResponse = response.json().await.map_err(|e| {
            LuaError::RuntimeError(format!("Failed to parse crates.io response: {}", e))
        })?;

        Ok(crates
            .crates
            .into_iter()
            .map(|c| format!("{} ({})", c.name, c.max_version))
            .collect())
    }
}

#[mlua::lua_module]
fn nvim_cargo_add(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    let cargo_add = CargoAdd::new()?;

    // カーゴの追加
    let add_cargo = {
        let cargo_add = cargo_add.clone();
        lua.create_function(move |_, (name, version): (String, Option<String>)| {
            let runtime = cargo_add.runtime.clone();
            runtime.block_on(async { cargo_add.add_dependency(&name, version.as_deref()).await })
        })?
    };

    // クレートの検索
    let search_crates = {
        let cargo_add = cargo_add.clone();
        lua.create_function(move |_, query: String| {
            let runtime = cargo_add.runtime.clone();
            runtime.block_on(async { cargo_add.search_crates(&query).await })
        })?
    };

    exports.set("add", add_cargo)?;
    exports.set("search", search_crates)?;

    Ok(exports)
}
