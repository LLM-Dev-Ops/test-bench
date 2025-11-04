// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! WASM runtime using wasmtime.

use anyhow::{Result, Context, bail};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, ResourceTable, WasiView};
use std::sync::Arc;
use parking_lot::Mutex;

use crate::plugins::types::ResourceLimits;

/// WASM runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Resource limits
    pub limits: ResourceLimits,

    /// Enable WASI support
    pub enable_wasi: bool,

    /// Enable multi-memory support
    pub enable_multi_memory: bool,

    /// Enable bulk memory operations
    pub enable_bulk_memory: bool,

    /// Enable reference types
    pub enable_reference_types: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            limits: ResourceLimits::default(),
            enable_wasi: true,
            enable_multi_memory: false,
            enable_bulk_memory: true,
            enable_reference_types: true,
        }
    }
}

/// Runtime limits wrapper
#[derive(Debug, Clone)]
pub struct RuntimeLimits {
    limits: ResourceLimits,
}

impl RuntimeLimits {
    pub fn new(limits: ResourceLimits) -> Self {
        Self { limits }
    }

    pub fn max_memory_bytes(&self) -> usize {
        self.limits.max_memory_bytes
    }

    pub fn max_execution_time_ms(&self) -> u64 {
        self.limits.max_execution_time_ms
    }
}

/// WASM runtime instance
pub struct WasmRuntime {
    engine: Engine,
    config: RuntimeConfig,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let mut engine_config = Config::new();

        // Configure engine
        engine_config.wasm_bulk_memory(config.enable_bulk_memory);
        engine_config.wasm_reference_types(config.enable_reference_types);
        engine_config.wasm_multi_memory(config.enable_multi_memory);

        // Set resource limits
        engine_config.max_wasm_stack(2 * 1024 * 1024); // 2 MB stack

        // Enable async support
        engine_config.async_support(true);

        // Create engine
        let engine = Engine::new(&engine_config)
            .context("Failed to create WASM engine")?;

        Ok(Self { engine, config })
    }

    /// Load a WASM module from bytes
    pub fn load_module(&self, wasm_bytes: &[u8]) -> Result<Module> {
        Module::new(&self.engine, wasm_bytes)
            .context("Failed to load WASM module")
    }

    /// Create a new instance of a module
    pub async fn instantiate(&self, module: &Module) -> Result<WasmInstance> {
        let mut linker = Linker::new(&self.engine);

        // Add WASI if enabled
        let wasi_ctx = if self.config.enable_wasi {
            Some(self.create_wasi_ctx()?)
        } else {
            None
        };

        // Create store with limits
        let mut store = self.create_store(wasi_ctx)?;

        // Add WASI to linker if enabled
        // TODO: Update to wasmtime 19.0 WASI API
        // The WASI API has changed significantly in wasmtime 19.0
        // For now, plugins will run without WASI support
        if self.config.enable_wasi {
            tracing::warn!("WASI support is temporarily disabled pending API migration to wasmtime 19.0");
            // wasmtime_wasi::command::add_to_linker::<StoreData>(&mut linker)
            //     .context("Failed to add WASI to linker")?;
        }

        // Instantiate the module
        let instance = linker
            .instantiate_async(&mut store, module)
            .await
            .context("Failed to instantiate WASM module")?;

        Ok(WasmInstance {
            store: Arc::new(Mutex::new(store)),
            instance,
            config: self.config.clone(),
        })
    }

    /// Create WASI context
    fn create_wasi_ctx(&self) -> Result<WasiCtx> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
        Ok(wasi)
    }

    /// Create store with resource limits
    fn create_store(&self, wasi_ctx: Option<WasiCtx>) -> Result<Store<StoreData>> {
        let wasi = wasi_ctx.unwrap_or_else(|| {
            WasiCtxBuilder::new().build()
        });

        let data = StoreData {
            wasi_ctx: wasi,
            table: ResourceTable::new(),
            limits: StoreLimits::default(),
        };

        let mut store = Store::new(&self.engine, data);

        // Set memory limits
        let max_memory = self.config.limits.max_memory_bytes;
        store.limiter(|data| &mut data.limits);
        store.data_mut().limits.memory_size = max_memory;

        Ok(store)
    }

    /// Get the engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

/// Store data with WASI context and limits
struct StoreData {
    wasi_ctx: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
}

impl WasiView for StoreData {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

/// Store limits
#[derive(Default)]
struct StoreLimits {
    memory_size: usize,
}

impl ResourceLimiter for StoreLimits {
    fn memory_growing(&mut self, current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool, Error> {
        if desired > self.memory_size {
            return Ok(false);
        }
        Ok(true)
    }

    fn table_growing(&mut self, _current: u32, desired: u32, _maximum: Option<u32>) -> Result<bool, Error> {
        const MAX_TABLE_ELEMENTS: u32 = 10000;
        Ok(desired < MAX_TABLE_ELEMENTS)
    }
}

/// WASM instance wrapper
pub struct WasmInstance {
    store: Arc<Mutex<Store<StoreData>>>,
    instance: Instance,
    config: RuntimeConfig,
}

impl WasmInstance {
    /// Call a function exported by the WASM module
    pub async fn call_function(
        &self,
        name: &str,
        args: &[Val],
    ) -> Result<Vec<Val>> {
        let func = self.instance
            .get_func(&mut *self.store.lock(), name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", name))?;

        let mut results = vec![Val::I32(0); func.ty(&*self.store.lock()).results().len()];

        func.call_async(&mut *self.store.lock(), args, &mut results)
            .await
            .context(format!("Failed to call function '{}'", name))?;

        Ok(results)
    }

    /// Get exported memory
    pub fn get_memory(&self, name: &str) -> Result<Memory> {
        self.instance
            .get_memory(&mut *self.store.lock(), name)
            .ok_or_else(|| anyhow::anyhow!("Memory '{}' not found", name))
    }

    /// Read data from memory
    pub fn read_memory(&self, memory: &Memory, offset: usize, len: usize) -> Result<Vec<u8>> {
        let store = self.store.lock();
        let data = memory.data(&*store);

        if offset + len > data.len() {
            bail!("Memory read out of bounds");
        }

        Ok(data[offset..offset + len].to_vec())
    }

    /// Write data to memory
    pub fn write_memory(&self, memory: &Memory, offset: usize, data: &[u8]) -> Result<()> {
        let mut store = self.store.lock();
        let mem_data = memory.data_mut(&mut *store);

        if offset + data.len() > mem_data.len() {
            bail!("Memory write out of bounds");
        }

        mem_data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Allocate memory in the WASM instance
    pub async fn allocate(&self, size: usize) -> Result<u32> {
        let args = [Val::I32(size as i32)];
        let results = self.call_function("plugin_alloc", &args).await?;

        if let Some(Val::I32(ptr)) = results.first() {
            Ok(*ptr as u32)
        } else {
            bail!("Invalid allocation result")
        }
    }

    /// Free memory in the WASM instance
    pub async fn free(&self, ptr: u32, size: usize) -> Result<()> {
        let args = [Val::I32(ptr as i32), Val::I32(size as i32)];
        self.call_function("plugin_free", &args).await?;
        Ok(())
    }

    /// Get the store (for advanced usage)
    pub fn store(&self) -> Arc<Mutex<Store<StoreData>>> {
        self.store.clone()
    }

    /// Get the instance
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config() {
        let config = RuntimeConfig::default();
        assert!(config.enable_wasi);
        assert!(config.enable_bulk_memory);
    }

    #[tokio::test]
    async fn test_runtime_creation() {
        let config = RuntimeConfig::default();
        let runtime = WasmRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_runtime_limits() {
        let limits = ResourceLimits {
            max_memory_bytes: 128 * 1024 * 1024,
            max_execution_time_ms: 60_000,
            max_instructions: Some(2_000_000_000),
        };

        let runtime_limits = RuntimeLimits::new(limits);
        assert_eq!(runtime_limits.max_memory_bytes(), 128 * 1024 * 1024);
        assert_eq!(runtime_limits.max_execution_time_ms(), 60_000);
    }
}
