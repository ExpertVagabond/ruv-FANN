;; Memory growth test module
(module
  (memory 1)  ;; Start with 1 page (64KB)
  
  ;; Try to grow memory by specified pages
  (func $grow_memory (param $pages i32) (result i32)
    (local.get $pages)
    (memory.grow))
  
  ;; Get current memory size in pages
  (func $memory_size (result i32)
    (memory.size))
  
  (export "grow_memory" (func $grow_memory))
  (export "memory_size" (func $memory_size))
  (export "memory" (memory 0))
)