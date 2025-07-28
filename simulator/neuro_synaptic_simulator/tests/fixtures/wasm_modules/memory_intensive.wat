(module
  ;; Memory-intensive computation module for testing limits
  (func $allocate_and_compute (export "compute") 
    (param $size i32) (result f32)
    (local $i i32)
    (local $sum f32)
    
    ;; Try to allocate memory
    memory.size
    local.get $size
    i32.const 65536  ;; Pages are 64KB
    i32.div_u
    i32.const 1
    i32.add
    memory.grow
    
    ;; Check if allocation succeeded
    i32.const -1
    i32.eq
    if
      ;; Return -1.0 to indicate failure
      f32.const -1.0
      return
    end
    
    ;; Fill memory with pattern and compute sum
    f32.const 0.0
    local.set $sum
    
    i32.const 0
    local.set $i
    
    loop $fill_loop
      ;; Store value
      local.get $i
      local.get $i
      f32.convert_i32_u
      f32.const 0.001
      f32.mul
      f32.store
      
      ;; Add to sum
      local.get $i
      f32.load
      local.get $sum
      f32.add
      local.set $sum
      
      ;; Next position
      local.get $i
      i32.const 4
      i32.add
      local.set $i
      
      ;; Continue if not done
      local.get $i
      local.get $size
      i32.lt_u
      br_if $fill_loop
    end
    
    local.get $sum
  )
  
  ;; Initial small memory
  (memory (export "memory") 1 1000)  ;; 1 initial page, max 1000 pages
)