use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use anyhow::{Result, Context};
use wasmtime::Module;
use sha2::{Sha256, Digest};

use super::engine::WasmEngine;

/// Module info with metadata
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub path: PathBuf,
    pub hash: String,
    pub size: usize,
    pub compile_time_ms: u64,
}

/// Module loader with caching support
pub struct ModuleLoader {
    engine: Arc<WasmEngine>,
    module_cache: Arc<RwLock<HashMap<String, CachedModule>>>,
    module_info: Arc<RwLock<HashMap<String, ModuleInfo>>>,
    cache_dir: Option<PathBuf>,
}

/// Cached module data
struct CachedModule {
    module: Module,
    hash: String,
    loaded_at: std::time::Instant,
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new(engine: Arc<WasmEngine>, cache_dir: Option<PathBuf>) -> Result<Self> {
        // Create cache directory if specified
        if let Some(ref dir) = cache_dir {
            fs::create_dir_all(dir)
                .context("Failed to create cache directory")?;
        }
        
        Ok(ModuleLoader {
            engine,
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            module_info: Arc::new(RwLock::new(HashMap::new())),
            cache_dir,
        })
    }
    
    /// Load a module from file
    pub fn load_from_file(&self, module_name: &str, path: &Path) -> Result<Module> {
        // Read file
        let wasm_bytes = fs::read(path)
            .context(format!("Failed to read WASM file: {:?}", path))?;
        
        // Calculate hash
        let hash = Self::calculate_hash(&wasm_bytes);
        
        // Check cache
        {
            let cache = self.module_cache.read().unwrap();
            if let Some(cached) = cache.get(module_name) {
                if cached.hash == hash {
                    log::debug!("Module '{}' loaded from cache", module_name);
                    return Ok(cached.module.clone());
                }
            }
        }
        
        // Load and compile module
        let start = std::time::Instant::now();
        let module = self.engine.load_module(module_name, &wasm_bytes)?;
        let compile_time = start.elapsed().as_millis() as u64;
        
        // Cache compiled module
        self.cache_module(module_name, module.clone(), hash.clone())?;
        
        // Store module info
        let info = ModuleInfo {
            name: module_name.to_string(),
            path: path.to_path_buf(),
            hash: hash.clone(),
            size: wasm_bytes.len(),
            compile_time_ms: compile_time,
        };
        
        {
            let mut infos = self.module_info.write().unwrap();
            infos.insert(module_name.to_string(), info);
        }
        
        // Save to disk cache if enabled
        if let Some(ref cache_dir) = self.cache_dir {
            self.save_to_disk_cache(module_name, &module, &hash)?;
        }
        
        log::info!(
            "Loaded module '{}' from {:?} ({} bytes, compiled in {}ms)",
            module_name,
            path,
            wasm_bytes.len(),
            compile_time
        );
        
        Ok(module)
    }
    
    /// Load a module from bytes
    pub fn load_from_bytes(&self, module_name: &str, wasm_bytes: &[u8]) -> Result<Module> {
        // Calculate hash
        let hash = Self::calculate_hash(wasm_bytes);
        
        // Check cache
        {
            let cache = self.module_cache.read().unwrap();
            if let Some(cached) = cache.get(module_name) {
                if cached.hash == hash {
                    return Ok(cached.module.clone());
                }
            }
        }
        
        // Load and compile
        let module = self.engine.load_module(module_name, wasm_bytes)?;
        
        // Cache module
        self.cache_module(module_name, module.clone(), hash)?;
        
        Ok(module)
    }
    
    /// Load from disk cache if available
    pub fn load_from_disk_cache(&self, module_name: &str) -> Result<Option<Module>> {
        let cache_dir = match &self.cache_dir {
            Some(dir) => dir,
            None => return Ok(None),
        };
        
        let cache_path = cache_dir.join(format!("{}.cached", module_name));
        if !cache_path.exists() {
            return Ok(None);
        }
        
        // Read cached module
        let cached_bytes = fs::read(&cache_path)
            .context("Failed to read cached module")?;
        
        // Deserialize module
        unsafe {
            let module = Module::deserialize(&self.engine.engine, &cached_bytes)
                .context("Failed to deserialize cached module")?;
            
            log::debug!("Loaded module '{}' from disk cache", module_name);
            Ok(Some(module))
        }
    }
    
    /// Save module to disk cache
    fn save_to_disk_cache(
        &self,
        module_name: &str,
        module: &Module,
        hash: &str,
    ) -> Result<()> {
        let cache_dir = match &self.cache_dir {
            Some(dir) => dir,
            None => return Ok(()),
        };
        
        // Serialize module
        let serialized = module.serialize()
            .context("Failed to serialize module")?;
        
        // Save to cache file
        let cache_path = cache_dir.join(format!("{}.cached", module_name));
        fs::write(&cache_path, &serialized)
            .context("Failed to write cached module")?;
        
        // Save hash for validation
        let hash_path = cache_dir.join(format!("{}.hash", module_name));
        fs::write(&hash_path, hash)
            .context("Failed to write module hash")?;
        
        log::debug!("Saved module '{}' to disk cache", module_name);
        Ok(())
    }
    
    /// Cache a module in memory
    fn cache_module(&self, name: &str, module: Module, hash: String) -> Result<()> {
        let cached = CachedModule {
            module,
            hash,
            loaded_at: std::time::Instant::now(),
        };
        
        let mut cache = self.module_cache.write().unwrap();
        cache.insert(name.to_string(), cached);
        
        Ok(())
    }
    
    /// Calculate SHA256 hash of bytes
    fn calculate_hash(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
    
    /// Get module info
    pub fn get_module_info(&self, module_name: &str) -> Option<ModuleInfo> {
        let infos = self.module_info.read().unwrap();
        infos.get(module_name).cloned()
    }
    
    /// List all loaded modules
    pub fn list_modules(&self) -> Vec<String> {
        let cache = self.module_cache.read().unwrap();
        cache.keys().cloned().collect()
    }
    
    /// Clear module cache
    pub fn clear_cache(&self) {
        let mut cache = self.module_cache.write().unwrap();
        cache.clear();
        
        let mut infos = self.module_info.write().unwrap();
        infos.clear();
        
        log::info!("Cleared module cache");
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.module_cache.read().unwrap();
        let infos = self.module_info.read().unwrap();
        
        let total_size: usize = infos.values().map(|i| i.size).sum();
        let total_compile_time: u64 = infos.values().map(|i| i.compile_time_ms).sum();
        
        CacheStats {
            modules_cached: cache.len(),
            total_size_bytes: total_size,
            total_compile_time_ms: total_compile_time,
            cache_dir: self.cache_dir.clone(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub modules_cached: usize,
    pub total_size_bytes: usize,
    pub total_compile_time_ms: u64,
    pub cache_dir: Option<PathBuf>,
}

/// Module validator
pub struct ModuleValidator;

impl ModuleValidator {
    /// Validate a WASM module
    pub fn validate(wasm_bytes: &[u8]) -> Result<()> {
        wasmparser::validate(wasm_bytes)
            .context("Invalid WASM module")?;
        Ok(())
    }
    
    /// Check if module exports required functions
    pub fn check_exports(wasm_bytes: &[u8], required_exports: &[&str]) -> Result<()> {
        let mut exports = Vec::new();
        
        for payload in wasmparser::Parser::new(0).parse_all(wasm_bytes) {
            if let wasmparser::Payload::ExportSection(export_section) = payload? {
                for export in export_section {
                    let export = export?;
                    exports.push(export.name.to_string());
                }
            }
        }
        
        for required in required_exports {
            if !exports.contains(&required.to_string()) {
                anyhow::bail!("Module missing required export: {}", required);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_calculation() {
        let data1 = b"test data";
        let data2 = b"test data";
        let data3 = b"different data";
        
        let hash1 = ModuleLoader::calculate_hash(data1);
        let hash2 = ModuleLoader::calculate_hash(data2);
        let hash3 = ModuleLoader::calculate_hash(data3);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_module_validator() {
        // Valid empty module
        let valid_wasm = wat::parse_str(r#"(module)"#).unwrap();
        assert!(ModuleValidator::validate(&valid_wasm).is_ok());
        
        // Invalid bytes
        let invalid_wasm = b"not wasm";
        assert!(ModuleValidator::validate(invalid_wasm).is_err());
    }
    
    #[test]
    fn test_export_checking() {
        let wasm = wat::parse_str(r#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add)
                (func (export "mul") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.mul))
        "#).unwrap();
        
        assert!(ModuleValidator::check_exports(&wasm, &["add", "mul"]).is_ok());
        assert!(ModuleValidator::check_exports(&wasm, &["add", "sub"]).is_err());
    }
}