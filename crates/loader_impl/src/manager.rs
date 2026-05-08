use crate::plugin::Plugin;
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub enum PluginEntry {
    Host {
        name: String,
        version: String,
    },
    Dynamic {
        context: Arc<crate::plugin::PluginContext>,
    },
}

impl PluginEntry {
    pub fn name(&self) -> String {
        match self {
            PluginEntry::Host { name, .. } => name.clone(),
            PluginEntry::Dynamic { context } => {
                if let Ok(lock) = context.meta.lock() {
                    lock.name.clone()
                } else {
                    String::new()
                }
            }
        }
    }

    pub fn version(&self) -> String {
        match self {
            PluginEntry::Host { version, .. } => version.clone(),
            PluginEntry::Dynamic { context } => {
                if let Ok(lock) = context.meta.lock() {
                    lock.version.clone()
                } else {
                    String::new()
                }
            }
        }
    }
}

pub struct PluginManager {
    plugins: Mutex<HashMap<String, PluginEntry>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        // register loader as a host entry under key "host:loader"
        let loader_key = "host:loader".to_string();
        let loader_ver = crate::loader_version().to_string();
        m.insert(
            loader_key,
            PluginEntry::Host {
                name: "loader".to_string(),
                version: loader_ver,
            },
        );
        Self {
            plugins: Mutex::new(m),
        }
    }

    pub fn load_plugin(&self, path: &str) -> Result<String, String> {
        let abs = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let key = abs.to_string_lossy().into_owned();

        // If already loaded, return its name
        if let Ok(map) = self.plugins.lock() {
            if let Some(existing) = map.get(&key) {
                return Ok(existing.name());
            }
        }

        // Load the plugin library first to inspect declared dependencies.
        let context = Plugin::load(&abs)?;
        let deps = if let Ok(lock) = context.meta.lock() {
            lock.deps.clone()
        } else {
            Vec::new()
        };
        let mut newly_loaded: Vec<String> = Vec::new();

        // helper closure: verify that an entry exists in the map and satisfies an optional version requirement
        let verify_loaded = |key: &str,
                             req_opt: &Option<String>,
                             newly_loaded: &mut Vec<String>|
         -> Result<(), String> {
            if let Ok(map) = self.plugins.lock() {
                if let Some(existing) = map.get(key) {
                    if let Some(req_str) = req_opt {
                        let req = VersionReq::parse(req_str).map_err(|e| {
                            format!("invalid version requirement '{}' : {}", req_str, e)
                        })?;
                        let ver_str = existing.version();
                        if ver_str.is_empty() {
                            for k in newly_loaded.iter().rev() {
                                let _ = self.unload_plugin(k);
                            }
                            return Err(format!("dependency '{}' does not provide version", key));
                        }
                        let ver = Version::parse(&ver_str).map_err(|e| {
                            for k in newly_loaded.iter().rev() {
                                let _ = self.unload_plugin(k);
                            }
                            format!(
                                "invalid version '{}' from plugin {} : {}",
                                ver_str,
                                existing.name(),
                                e
                            )
                        })?;
                        if !req.matches(&ver) {
                            for k in newly_loaded.iter().rev() {
                                let _ = self.unload_plugin(k);
                            }
                            return Err(format!(
                                "version mismatch for {}: {} does not satisfy {}",
                                key, ver, req
                            ));
                        }
                    }
                    Ok(())
                } else {
                    for k in newly_loaded.iter().rev() {
                        let _ = self.unload_plugin(k);
                    }
                    Err(format!("dependency '{}' failed to load", key))
                }
            } else {
                for k in newly_loaded.iter().rev() {
                    let _ = self.unload_plugin(k);
                }
                Err("failed to lock plugin map".to_string())
            }
        };

        for dep in deps {
            let (path_part, ver_req_opt) = if let Some(idx) = dep.rfind('@') {
                let (p, v) = dep.split_at(idx);
                (p.to_string(), Some(v[1..].to_string()))
            } else {
                (dep.clone(), None)
            };

            // normalize host short names like "loader" to host:<name>
            if path_part == "loader" || path_part.starts_with("host:") {
                let host_key = if path_part.starts_with("host:") {
                    path_part.clone()
                } else {
                    format!("host:{}", path_part)
                };
                // verify host entry exists and satisfies version
                verify_loaded(&host_key, &ver_req_opt, &mut newly_loaded)?;
                continue;
            }

            // dynamic dependency: canonicalize, then either verify existing or load then verify
            let dep_abs = std::fs::canonicalize(&path_part).map_err(|e| {
                for k in newly_loaded.iter().rev() {
                    let _ = self.unload_plugin(k);
                }
                format!("failed to canonicalize dependency '{}' : {}", path_part, e)
            })?;
            let dep_key = dep_abs.to_string_lossy().into_owned();

            // if not loaded, attempt to load
            if let Ok(map) = self.plugins.lock() {
                if !map.contains_key(&dep_key) {
                    match self.load_plugin(dep_abs.to_string_lossy().as_ref()) {
                        Ok(_) => newly_loaded.push(dep_key.clone()),
                        Err(e) => {
                            for k in newly_loaded.iter().rev() {
                                let _ = self.unload_plugin(k);
                            }
                            return Err(e);
                        }
                    }
                }
            } else {
                for k in newly_loaded.iter().rev() {
                    let _ = self.unload_plugin(k);
                }
                return Err("failed to lock plugin map".to_string());
            }

            // verify the (now-loaded) dependency satisfies version
            verify_loaded(&dep_key, &ver_req_opt, &mut newly_loaded)?;
        }

        let name = if let Ok(lock) = context.meta.lock() {
            lock.name.clone()
        } else {
            String::new()
        };
        if let Ok(mut map) = self.plugins.lock() {
            map.insert(
                key.clone(),
                PluginEntry::Dynamic {
                    context: context.clone(),
                },
            );
        } else {
            for k in newly_loaded.iter().rev() {
                let _ = self.unload_plugin(k);
            }
            return Err("failed to lock plugin map".to_string());
        }
        Ok(name)
    }

    pub fn unload_plugin(&self, path: &str) -> Result<(), String> {
        let abs = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let key = abs.to_string_lossy().into_owned();
        if let Ok(mut map) = self.plugins.lock() {
            if let Some(entry) = map.remove(&key) {
                if let PluginEntry::Dynamic { context } = entry {
                    context.on_unload();
                    context.free();
                }
            }
            Ok(())
        } else {
            Err("failed to lock plugin map".to_string())
        }
    }

    pub fn list_plugins(&self) -> Result<Vec<String>, String> {
        if let Ok(map) = self.plugins.lock() {
            let mut out = Vec::new();
            for (path, plugin) in map.iter() {
                let entry = format!("{} -> {}@{}", path, plugin.name(), plugin.version());
                out.push(entry);
            }
            Ok(out)
        } else {
            Err("failed to lock plugin map".to_string())
        }
    }

    pub fn call_plugin_by_path(&self, path: &str, _input: &[u8]) -> Result<String, String> {
        let abs = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let key = abs.to_string_lossy().into_owned();
        if let Ok(map) = self.plugins.lock() {
            if let Some(entry) = map.get(&key) {
                match entry {
                    PluginEntry::Dynamic { .. } => Err("plugin process API removed".to_string()),
                    PluginEntry::Host { .. } => Err("cannot call host entry".to_string()),
                }
            } else {
                Err("plugin not loaded".to_string())
            }
        } else {
            Err("failed to lock plugin map".to_string())
        }
    }
}

static GLOBAL_PLUGIN_MANAGER: OnceLock<Arc<PluginManager>> = OnceLock::new();

pub fn set_global_plugin_manager(m: Arc<PluginManager>) -> Result<(), String> {
    GLOBAL_PLUGIN_MANAGER
        .set(m)
        .map_err(|_| "global plugin manager already set".to_string())
}

pub fn get_global_plugin_manager() -> Option<&'static Arc<PluginManager>> {
    GLOBAL_PLUGIN_MANAGER.get()
}
