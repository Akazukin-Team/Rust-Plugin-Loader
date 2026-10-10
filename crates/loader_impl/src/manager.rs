use crate::plugin::{PluginContext, PluginMeta};
use lib::{file, matcher::Matcher};
use libloading::Error;
use semver::{Version, VersionReq};
use std::{
    collections::HashMap,
    error::Error as IoError,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

pub struct PluginManager {
    plugins: Arc<Mutex<HashMap<String, Arc<PluginContext>>>>,
}

struct PathMatcher {
    ext: String,
}

impl PathMatcher {
    pub fn new(ext: String) -> Self {
        Self { ext }
    }
}

impl Matcher for PathMatcher {
    type Item = PathBuf;

    fn matches(&self, item: &Self::Item) -> bool {
        item.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(&self.ext))
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        // register loader as a host entry under key "host:loader"
        let loader_key = "host:loader".to_string();
        let loader_ver = crate::loader_version().to_string();

        let loader_meta = PluginMeta::new(loader_key.clone(), loader_ver, Vec::new());
        map.insert(loader_key, Arc::new(PluginContext::new(loader_meta, None)));

        Self {
            plugins: Arc::new(Mutex::new(map)),
        }
    }

    pub fn load_plugins(&self, path: &PathBuf) -> Result<Vec<Arc<PluginContext>>, String> {
        let ext: Option<&str> = if cfg!(target_os = "windows") {
            Some("dll")
        } else if cfg!(target_os = "macos") {
            Some("dylib")
        } else if cfg!(target_os = "linux") {
            Some("so")
        } else {
            None
        };

        if ext.is_none() {
            return Err("Not supported".to_string());
        }

        let matcher = &PathMatcher::new(ext.unwrap().to_string());
        let res = file::find_files(path, matcher);
        if res.is_err() {
            return Err("Failed to resolve plugins".to_string());
        }

        res.ok().map(|lib| self.load_plugin(path))
    }

    pub fn load_plugin<'a>(&'a self, path: &str) -> Result<Arc<PluginContext>, String> {
        let abs = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let key = abs.to_string_lossy().into_owned();

        let ctx = if let Ok(plugins) = self.plugins.lock() {
            if plugins.contains_key(&key) {
                return Err(format!("The plugin is already registered; {}", key));
            } else {
                PluginContext::load(&abs)?
            }
        } else {
            return Err("Failed to lock plugin map".to_string());
        };

        let deps = ctx.get_meta().get_dependencies();
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
                        let ver_str = existing.get_meta().get_version();
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
                                existing.get_meta().get_name(),
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

        if let Ok(mut map) = self.plugins.lock() {
            let plugin_name = &ctx.get_meta().get_name().to_string();
            let ctx_arc = Arc::new(ctx);
            map.insert(plugin_name.clone(), ctx_arc.clone());
            return Ok(ctx_arc);
        } else {
            for k in newly_loaded.iter().rev() {
                let _ = self.unload_plugin(k);
            }
            return Err("failed to lock plugin map".to_string());
        }
    }

    pub fn unload_plugin(&self, path: &str) -> Result<(), String> {
        let abs = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let key = abs.to_string_lossy().into_owned();
        if let Ok(mut set) = self.plugins.lock() {
            if let Some(ctx) = set.remove(&key) {
                if let Some(vt) = ctx.get_vtable() {
                    vt.plugin_on_unload();
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
                let meta = plugin.get_meta();
                let entry = format!("{} -> {}@{}", path, meta.get_name(), meta.get_version());
                out.push(entry);
            }
            Ok(out)
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
