use wasmtime::{Config, Engine, Instance, Module, Store, ResourceLimiter, Memory, AsContextMut};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use anyhow::{Result, Context};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

/// WASM Engine wrapper for Neuro-Synaptic chip simulation
pub struct WasmEngine {
    engine: Engine,
    module_cache: Arc<Mutex<HashMap<String, Module>>>,
    instance_count: Arc<Mutex<usize>>,
    max_instances: usize,
}

impl WasmEngine {
    /// Create a new WASM engine with optimized configuration
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        
        // Enable optimizations for performance
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        config.parallel_compilation(true);
        config.cache_config_load_default()?;
        
        // Enable fuel consumption for instruction counting
        config.consume_fuel(true);
        
        // Create engine with configuration
        let engine = Engine::new(&config)?;
        
        Ok(WasmEngine {
            engine,
            module_cache: Arc::new(Mutex::new(HashMap::new())),
            instance_count: Arc::new(Mutex::new(0)),
            max_instances: 256, // Support 256 concurrent instances
        })
    }
    
    /// Load and compile a WASM module with caching
    pub fn load_module(&self, module_name: &str, wasm_bytes: &[u8]) -> Result<Module> {
        // Check cache first
        {
            let cache = self.module_cache.lock().unwrap();
            if let Some(module) = cache.get(module_name) {
                return Ok(module.clone());
            }
        }
        
        // Compile module
        let start = Instant::now();
        let module = Module::new(&self.engine, wasm_bytes)
            .context("Failed to compile WASM module")?;
        let compile_time = start.elapsed();
        
        log::debug!("Compiled module '{}' in {:?}", module_name, compile_time);
        
        // Cache the compiled module
        {
            let mut cache = self.module_cache.lock().unwrap();
            cache.insert(module_name.to_string(), module.clone());
        }
        
        Ok(module)
    }
    
    /// Create a new WASM instance with shared memory
    pub fn create_instance(
        &self,
        module: &Module,
        shared_memory: Arc<Memory>,
        fuel_limit: u64,
    ) -> Result<WasmInstance> {
        // Check instance limit
        {
            let mut count = self.instance_count.lock().unwrap();
            if *count >= self.max_instances {
                anyhow::bail!("Maximum instance limit ({}) reached", self.max_instances);
            }
            *count += 1;
        }
        
        // Create store with limits
        let mut store = Store::new(&self.engine, StoreData {
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
            limiter: StoreLimiter::new(fuel_limit),
        });
        
        // Set fuel limit
        store.add_fuel(fuel_limit)?;
        
        // Create instance
        let instance = Instance::new(&mut store, module, &[])
            .context("Failed to create WASM instance")?;
        
        Ok(WasmInstance {
            instance,
            store,
            shared_memory,
            instance_count: self.instance_count.clone(),
        })
    }
    
    /// Get current instance count
    pub fn instance_count(&self) -> usize {
        *self.instance_count.lock().unwrap()
    }
    
    /// Clear module cache
    pub fn clear_cache(&self) {
        self.module_cache.lock().unwrap().clear();
    }
}

/// WASM instance wrapper
pub struct WasmInstance {
    instance: Instance,
    store: Store<StoreData>,
    shared_memory: Arc<Memory>,
    instance_count: Arc<Mutex<usize>>,
}

impl WasmInstance {
    /// Execute a function in the WASM instance
    pub fn execute_function(
        &mut self,
        func_name: &str,
        args: &[wasmtime::Val],
    ) -> Result<Vec<wasmtime::Val>> {
        let func = self.instance
            .get_func(&mut self.store, func_name)
            .context(format!("Function '{}' not found", func_name))?;
        
        let start = Instant::now();
        let mut results = vec![wasmtime::Val::I32(0); func.ty(&self.store).results().len()];
        
        func.call(&mut self.store, args, &mut results)
            .context(format!("Failed to execute function '{}'", func_name))?;
        
        let execution_time = start.elapsed();
        let fuel_consumed = self.store.fuel_consumed().unwrap_or(0);
        
        log::debug!(
            "Executed '{}' in {:?}, consumed {} fuel units",
            func_name,
            execution_time,
            fuel_consumed
        );
        
        Ok(results)
    }
    
    /// Get fuel consumed
    pub fn fuel_consumed(&self) -> u64 {
        self.store.fuel_consumed().unwrap_or(0)
    }
    
    /// Add more fuel
    pub fn add_fuel(&mut self, fuel: u64) -> Result<()> {
        self.store.add_fuel(fuel)?;
        Ok(())
    }
    
    /// Get shared memory reference
    pub fn shared_memory(&self) -> &Arc<Memory> {
        &self.shared_memory
    }
}

impl Drop for WasmInstance {
    fn drop(&mut self) {
        // Decrement instance count
        let mut count = self.instance_count.lock().unwrap();
        *count = count.saturating_sub(1);
    }
}

/// Store data for WASM instances
struct StoreData {
    wasi: WasiCtx,
    limiter: StoreLimiter,
}

/// Resource limiter for WASM instances
struct StoreLimiter {
    fuel_limit: u64,
}

impl StoreLimiter {
    fn new(fuel_limit: u64) -> Self {
        StoreLimiter { fuel_limit }
    }
}

impl ResourceLimiter for StoreLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        // Limit to 28MB (448 pages of 64KB each)
        const MAX_MEMORY_BYTES: usize = 28 * 1024 * 1024;
        Ok(desired <= MAX_MEMORY_BYTES)
    }
    
    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        Ok(true) // No table limits for now
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let engine = WasmEngine::new().unwrap();
        assert_eq!(engine.instance_count(), 0);
    }
    
    #[test]
    fn test_module_caching() {
        let engine = WasmEngine::new().unwrap();
        let wasm_bytes = wat::parse_str(r#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add))
        "#).unwrap();
        
        // First load should compile
        let module1 = engine.load_module("test", &wasm_bytes).unwrap();
        
        // Second load should use cache
        let module2 = engine.load_module("test", &wasm_bytes).unwrap();
        
        // Should be the same module
        assert_eq!(module1.serialize().unwrap(), module2.serialize().unwrap());
    }
}