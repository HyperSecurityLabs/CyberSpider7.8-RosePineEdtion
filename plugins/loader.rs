use anyhow::Result;
use libloading::{Library, Symbol};
use std::path::Path;
use crate::plugins::{Plugin, PluginInfo};

pub struct PluginLoader {
    libraries: Vec<Library>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            libraries: Vec::new(),
        }
    }

    pub async fn load_dynamic_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<Box<dyn Plugin>> {
        let path = path.as_ref();
        
        unsafe {
            let lib = Library::new(path)?;
            
            let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = 
                lib.get(b"create_plugin")?;
            
            let plugin_ptr = create_plugin();
            let plugin = Box::from_raw(plugin_ptr);
            
            self.libraries.push(lib);
            
            Ok(plugin)
        }
    }

    pub fn load_plugin_from_code<P: AsRef<Path>>(&self, plugin_file: P) -> Result<Box<dyn Plugin>> {
        let plugin_file = plugin_file.as_ref();
        
        if plugin_file.extension().and_then(|s| s.to_str()) == Some("rs") {
            self.compile_and_load_rust_plugin(plugin_file)
        } else {
            Err(anyhow::anyhow!("Unsupported plugin file format"))
        }
    }

    fn compile_and_load_rust_plugin(&self, plugin_file: &Path) -> Result<Box<dyn Plugin>> {
        let plugin_name = plugin_file
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid plugin file name"))?;
        
        let temp_dir = std::env::temp_dir();
        let temp_project_dir = temp_dir.join(format!("cyberspider_plugin_{}", plugin_name));
        
        std::fs::create_dir_all(&temp_project_dir)?;
        
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
cyberspider = {{ path = "{}" }}
async-trait = "0.1"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
"#,
            plugin_name,
            std::env::current_dir()?.to_string_lossy()
        );
        
        std::fs::write(temp_project_dir.join("Cargo.toml"), cargo_toml)?;
        std::fs::create_dir(temp_project_dir.join("src"))?;
        
        let plugin_content = std::fs::read_to_string(plugin_file)?;
        std::fs::write(temp_project_dir.join("src/lib.rs"), plugin_content)?;
        
        let output = std::process::Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&temp_project_dir)
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to compile plugin: {}", error));
        }
        
        let so_path = temp_project_dir
            .join("target/release")
            .join(format!("lib{}.so", plugin_name));
        
        let mut loader = PluginLoader::new();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(loader.load_dynamic_plugin(so_path))
        })
    }

    pub fn validate_plugin<P: AsRef<Path>>(&self, path: P) -> Result<PluginInfo> {
        let path = path.as_ref();
        
        unsafe {
            let lib = Library::new(path)?;
            
            let get_plugin_info: Symbol<unsafe extern "C" fn() -> PluginInfo> = 
                lib.get(b"get_plugin_info")?;
            
            let info = get_plugin_info();
            Ok(info)
        }
    }
}

impl Drop for PluginLoader {
    fn drop(&mut self) {
        self.libraries.clear();
    }
}
