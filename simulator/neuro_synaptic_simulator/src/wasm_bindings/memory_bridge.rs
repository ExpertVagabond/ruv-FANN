// Memory bridge for WASM - JavaScript shared memory management
// Provides efficient memory allocation and sharing between WASM and JS

use wasm_bindgen::prelude::*;
use js_sys::{SharedArrayBuffer, Uint8Array, Float32Array};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// Shared buffer for zero-copy data exchange
#[wasm_bindgen]
pub struct SharedBuffer {
    buffer: SharedArrayBuffer,
    offset: usize,
    size: usize,
}

#[wasm_bindgen]
impl SharedBuffer {
    /// Create a new shared buffer
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize) -> Result<SharedBuffer, JsValue> {
        let buffer = SharedArrayBuffer::new(size as u32);
        
        Ok(SharedBuffer {
            buffer,
            offset: 0,
            size,
        })
    }

    /// Get the underlying SharedArrayBuffer
    #[wasm_bindgen(getter)]
    pub fn buffer(&self) -> SharedArrayBuffer {
        self.buffer.clone()
    }

    /// Get buffer size
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Create a Float32Array view of the buffer
    pub fn as_f32_array(&self) -> Float32Array {
        Float32Array::new_with_byte_offset_and_length(
            &self.buffer,
            self.offset as u32,
            (self.size / 4) as u32,
        )
    }

    /// Create a Uint8Array view of the buffer
    pub fn as_u8_array(&self) -> Uint8Array {
        Uint8Array::new_with_byte_offset_and_length(
            &self.buffer,
            self.offset as u32,
            self.size as u32,
        )
    }
}

/// Memory pool for efficient allocation
pub struct WasmMemoryPool {
    total_size: usize,
    allocated: Arc<Mutex<usize>>,
    free_list: Arc<Mutex<Vec<MemoryRegion>>>,
}

/// Memory region descriptor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub offset: usize,
    pub size: usize,
    pub in_use: bool,
}

/// Memory statistics
#[derive(Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_bytes: usize,
    pub allocated_bytes: usize,
    pub free_bytes: usize,
    pub fragmentation_ratio: f32,
    pub allocation_count: usize,
}

impl WasmMemoryPool {
    /// Create a new memory pool
    pub fn new(size: usize) -> Result<Self, String> {
        let initial_region = MemoryRegion {
            offset: 0,
            size,
            in_use: false,
        };

        Ok(WasmMemoryPool {
            total_size: size,
            allocated: Arc::new(Mutex::new(0)),
            free_list: Arc::new(Mutex::new(vec![initial_region])),
        })
    }

    /// Allocate memory from the pool
    pub fn allocate(&self, size: usize) -> Result<MemoryRegion, String> {
        let mut free_list = self.free_list.lock().unwrap();
        
        // Find first-fit free region
        for (i, region) in free_list.iter_mut().enumerate() {
            if !region.in_use && region.size >= size {
                // Split region if necessary
                if region.size > size {
                    let new_region = MemoryRegion {
                        offset: region.offset + size,
                        size: region.size - size,
                        in_use: false,
                    };
                    region.size = size;
                    free_list.insert(i + 1, new_region);
                }
                
                region.in_use = true;
                
                let mut allocated = self.allocated.lock().unwrap();
                *allocated += size;
                
                return Ok(region.clone());
            }
        }
        
        Err("Out of memory".to_string())
    }

    /// Free a memory region
    pub fn free(&self, region: MemoryRegion) -> Result<(), String> {
        let mut free_list = self.free_list.lock().unwrap();
        
        // Find the region
        for r in free_list.iter_mut() {
            if r.offset == region.offset && r.size == region.size {
                if !r.in_use {
                    return Err("Region already free".to_string());
                }
                
                r.in_use = false;
                
                let mut allocated = self.allocated.lock().unwrap();
                *allocated -= region.size;
                
                // Coalesce adjacent free regions
                self.coalesce_free_regions();
                
                return Ok(());
            }
        }
        
        Err("Invalid memory region".to_string())
    }

    /// Coalesce adjacent free regions to reduce fragmentation
    fn coalesce_free_regions(&self) {
        let mut free_list = self.free_list.lock().unwrap();
        
        let mut i = 0;
        while i < free_list.len() - 1 {
            if !free_list[i].in_use && !free_list[i + 1].in_use {
                // Check if regions are adjacent
                if free_list[i].offset + free_list[i].size == free_list[i + 1].offset {
                    // Merge regions
                    free_list[i].size += free_list[i + 1].size;
                    free_list.remove(i + 1);
                    continue;
                }
            }
            i += 1;
        }
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let allocated = *self.allocated.lock().unwrap();
        let free_list = self.free_list.lock().unwrap();
        
        let free_bytes = free_list.iter()
            .filter(|r| !r.in_use)
            .map(|r| r.size)
            .sum();
        
        let allocation_count = free_list.iter()
            .filter(|r| r.in_use)
            .count();
        
        let fragmentation_ratio = if self.total_size > 0 {
            (free_list.len() as f32 - 1.0) / (self.total_size as f32 / 1024.0)
        } else {
            0.0
        };
        
        MemoryStats {
            total_bytes: self.total_size,
            allocated_bytes: allocated,
            free_bytes,
            fragmentation_ratio,
            allocation_count,
        }
    }

    /// Reset the memory pool
    pub fn reset(&self) {
        let mut free_list = self.free_list.lock().unwrap();
        free_list.clear();
        free_list.push(MemoryRegion {
            offset: 0,
            size: self.total_size,
            in_use: false,
        });
        
        let mut allocated = self.allocated.lock().unwrap();
        *allocated = 0;
    }
}

/// JavaScript-friendly memory allocation functions
#[wasm_bindgen]
pub fn allocate_shared_buffer(size: usize) -> Result<SharedBuffer, JsValue> {
    SharedBuffer::new(size)
}

#[wasm_bindgen]
pub fn create_f32_buffer(size: usize) -> Result<Float32Array, JsValue> {
    if size % 4 != 0 {
        return Err(JsValue::from_str("Size must be multiple of 4 for f32 array"));
    }
    
    let buffer = SharedArrayBuffer::new(size as u32);
    Ok(Float32Array::new(&buffer))
}

#[wasm_bindgen]
pub fn copy_to_shared_buffer(
    src: &Float32Array,
    dst: &SharedBuffer,
    offset: usize,
) -> Result<(), JsValue> {
    let dst_array = dst.as_f32_array();
    
    if offset + src.length() as usize > dst_array.length() as usize {
        return Err(JsValue::from_str("Copy would exceed buffer bounds"));
    }
    
    for i in 0..src.length() {
        dst_array.set_index(offset as u32 + i, src.get_index(i));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool() {
        let pool = WasmMemoryPool::new(1024).unwrap();
        
        // Test allocation
        let region1 = pool.allocate(256).unwrap();
        assert_eq!(region1.offset, 0);
        assert_eq!(region1.size, 256);
        
        let region2 = pool.allocate(256).unwrap();
        assert_eq!(region2.offset, 256);
        assert_eq!(region2.size, 256);
        
        // Test stats
        let stats = pool.get_stats();
        assert_eq!(stats.allocated_bytes, 512);
        assert_eq!(stats.free_bytes, 512);
        
        // Test free
        pool.free(region1).unwrap();
        let stats = pool.get_stats();
        assert_eq!(stats.allocated_bytes, 256);
    }
}