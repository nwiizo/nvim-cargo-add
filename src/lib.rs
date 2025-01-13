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

    async fn add_dependency(
        &self,
        crate_name: &str,
        version: Option<&str>,
        auto_save: bool,
    ) -> LuaResult<()> {
        if !std::path::Path::new("Cargo.toml").exists() {
            return Err(LuaError::RuntimeError(
                "Not in a Rust project directory".to_string(),
            ));
        }

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

        if auto_save {
            // 自動フォーマット
            let _ = Command::new("cargo")
                .arg("fmt")
                .output()
                .map_err(|e| LuaError::RuntimeError(format!("Failed to format: {}", e)))?;
        }

        Ok(())
    }

    async fn remove_dependency(&self, crate_name: &str, auto_save: bool) -> LuaResult<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("remove").arg(crate_name);

        let output = cmd.output().map_err(|e| {
            LuaError::RuntimeError(format!("Failed to execute cargo remove: {}", e))
        })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(LuaError::RuntimeError(format!(
                "cargo remove failed: {}",
                error
            )));
        }

        if auto_save {
            // 自動フォーマット
            let _ = Command::new("cargo")
                .arg("fmt")
                .output()
                .map_err(|e| LuaError::RuntimeError(format!("Failed to format: {}", e)))?;
        }

        Ok(())
    }

    async fn add_dev_dependency(
        &self,
        crate_name: &str,
        version: Option<&str>,
        auto_save: bool,
    ) -> LuaResult<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("add").arg("--dev").arg(crate_name);

        if let Some(ver) = version {
            cmd.arg("--version").arg(ver);
        }

        let output = cmd
            .output()
            .map_err(|e| LuaError::RuntimeError(format!("Failed to execute cargo add: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(LuaError::RuntimeError(format!(
                "cargo add dev-dependency failed: {}",
                error
            )));
        }

        if auto_save {
            // 自動フォーマット
            let _ = Command::new("cargo")
                .arg("fmt")
                .output()
                .map_err(|e| LuaError::RuntimeError(format!("Failed to format: {}", e)))?;
        }

        Ok(())
    }

    async fn search_crates(&self, query: &str) -> LuaResult<Vec<String>> {
        if query.len() < 2 {
            return Ok(Vec::new());
        }

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
            description: Option<String>,
            downloads: i64,
        }

        let crates: CratesResponse = response.json().await.map_err(|e| {
            LuaError::RuntimeError(format!("Failed to parse crates.io response: {}", e))
        })?;

        Ok(crates
            .crates
            .into_iter()
            .map(|c| {
                format!(
                    "{} ({})\t{}\t{} downloads",
                    c.name,
                    c.max_version,
                    c.description.unwrap_or_default(),
                    c.downloads
                )
            })
            .collect())
    }

    async fn list_dependencies(&self) -> LuaResult<Vec<String>> {
        let mut cmd = Command::new("cargo");
        cmd.arg("metadata")
            .arg("--format-version=1")
            .arg("--no-deps");

        let output = cmd.output().map_err(|e| {
            LuaError::RuntimeError(format!("Failed to execute cargo metadata: {}", e))
        })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(LuaError::RuntimeError(format!(
                "cargo metadata failed: {}",
                error
            )));
        }

        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| LuaError::RuntimeError(format!("Failed to parse metadata: {}", e)))?;

        let dependencies = metadata["packages"][0]["dependencies"]
            .as_array()
            .map(|deps| {
                deps.iter()
                    .map(|dep| {
                        format!(
                            "{} {}\t{}",
                            dep["name"].as_str().unwrap_or("unknown"),
                            dep["req"].as_str().unwrap_or("*"),
                            if dep["kind"].as_str() == Some("dev") {
                                "(dev)"
                            } else {
                                ""
                            }
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(dependencies)
    }
}

#[mlua::lua_module]
fn nvim_cargo_add(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    let cargo_add = CargoAdd::new()?;

    let add_cargo = {
        let cargo_add = cargo_add.clone();
        lua.create_function(
            move |_, (name, version, auto_save): (String, Option<String>, Option<bool>)| {
                let runtime = cargo_add.runtime.clone();
                runtime.block_on(async {
                    cargo_add
                        .add_dependency(&name, version.as_deref(), auto_save.unwrap_or(true))
                        .await
                })
            },
        )?
    };

    let search_crates = {
        let cargo_add = cargo_add.clone();
        lua.create_function(move |_, query: String| {
            let runtime = cargo_add.runtime.clone();
            runtime.block_on(async { cargo_add.search_crates(&query).await })
        })?
    };

    let remove_cargo = {
        let cargo_add = cargo_add.clone();
        lua.create_function(move |_, (name, auto_save): (String, Option<bool>)| {
            let runtime = cargo_add.runtime.clone();
            runtime.block_on(async {
                cargo_add
                    .remove_dependency(&name, auto_save.unwrap_or(true))
                    .await
            })
        })?
    };

    let add_dev_cargo = {
        let cargo_add = cargo_add.clone();
        lua.create_function(
            move |_, (name, version, auto_save): (String, Option<String>, Option<bool>)| {
                let runtime = cargo_add.runtime.clone();
                runtime.block_on(async {
                    cargo_add
                        .add_dev_dependency(&name, version.as_deref(), auto_save.unwrap_or(true))
                        .await
                })
            },
        )?
    };

    let list_deps = {
        let cargo_add = cargo_add.clone();
        lua.create_function(move |_, _: ()| {
            let runtime = cargo_add.runtime.clone();
            runtime.block_on(async { cargo_add.list_dependencies().await })
        })?
    };

    exports.set("add", add_cargo)?;
    exports.set("search", search_crates)?;
    exports.set("remove", remove_cargo)?;
    exports.set("add_dev", add_dev_cargo)?;
    exports.set("list", list_deps)?;

    Ok(exports)
}
