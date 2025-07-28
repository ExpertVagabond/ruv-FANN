(module
  ;; Simple neural network layer simulation
  (func $sigmoid (param $x f32) (result f32)
    ;; 1 / (1 + exp(-x))
    local.get $x
    f32.neg
    call $exp
    f32.const 1.0
    f32.add
    f32.const 1.0
    f32.div
  )
  
  (func $exp (param $x f32) (result f32)
    ;; Approximation of exp(x) using Taylor series
    local.get $x
    local.get $x
    f32.mul
    f32.const 0.5
    f32.mul
    local.get $x
    f32.add
    f32.const 1.0
    f32.add
  )
  
  (func $neural_layer (export "compute") 
    (param $in1 f32) (param $in2 f32) (param $in3 f32) 
    (result f32)
    ;; Simple 3-input neural layer with fixed weights
    local.get $in1
    f32.const 0.5
    f32.mul
    
    local.get $in2
    f32.const 0.3
    f32.mul
    f32.add
    
    local.get $in3
    f32.const 0.2
    f32.mul
    f32.add
    
    ;; Apply sigmoid activation
    call $sigmoid
  )
  
  ;; Process array of inputs
  (func $process_batch (export "process_inputs") 
    (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $result f32)
    
    ;; Process each triple of inputs
    local.get $ptr
    local.set $i
    
    loop $process_loop
      ;; Load three inputs
      local.get $i
      f32.load
      
      local.get $i
      i32.const 4
      i32.add
      f32.load
      
      local.get $i
      i32.const 8
      i32.add
      f32.load
      
      ;; Process through neural layer
      call $neural_layer
      
      ;; Store result
      local.get $i
      local.get $result
      f32.store
      
      ;; Increment pointer by 12 bytes (3 floats)
      local.get $i
      i32.const 12
      i32.add
      local.set $i
      
      ;; Check if done
      local.get $i
      local.get $ptr
      local.get $len
      i32.add
      i32.lt_u
      br_if $process_loop
    end
    
    ;; Return number of results
    local.get $len
    i32.const 3
    i32.div_u
  )
  
  ;; Memory for data processing
  (memory (export "memory") 10)
)