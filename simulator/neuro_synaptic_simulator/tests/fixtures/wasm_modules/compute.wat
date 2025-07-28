(module
  ;; General computation module for benchmarking
  (func $compute (export "compute") 
    (param $ptr i32) (param $len i32) (result f32)
    (local $i i32)
    (local $sum f32)
    (local $val f32)
    
    ;; Initialize
    f32.const 0.0
    local.set $sum
    i32.const 0
    local.set $i
    
    ;; Process array
    loop $compute_loop
      ;; Load value
      local.get $ptr
      local.get $i
      i32.const 4
      i32.mul
      i32.add
      f32.load
      local.set $val
      
      ;; Apply some computation
      local.get $val
      local.get $val
      f32.mul     ;; square
      f32.sqrt    ;; square root (back to original)
      local.get $val
      f32.const 2.0
      f32.mul
      f32.add     ;; + 2*val
      local.get $sum
      f32.add
      local.set $sum
      
      ;; Next element
      local.get $i
      i32.const 1
      i32.add
      local.set $i
      
      ;; Continue?
      local.get $i
      local.get $len
      i32.lt_u
      br_if $compute_loop
    end
    
    ;; Return average
    local.get $sum
    local.get $len
    f32.convert_i32_u
    f32.div
  )
  
  ;; Memory
  (memory (export "memory") 10)
)