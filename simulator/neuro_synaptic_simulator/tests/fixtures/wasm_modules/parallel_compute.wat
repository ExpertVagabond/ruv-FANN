(module
  ;; Parallel computation test module
  (func $vector_dot_product (export "compute") 
    (param $a_ptr i32) (param $b_ptr i32) (param $len i32) 
    (result f32)
    (local $i i32)
    (local $sum f32)
    
    ;; Initialize sum to 0
    f32.const 0.0
    local.set $sum
    
    ;; Initialize counter
    i32.const 0
    local.set $i
    
    ;; Compute dot product
    loop $dot_loop
      ;; Load values from both arrays
      local.get $a_ptr
      local.get $i
      i32.const 4
      i32.mul
      i32.add
      f32.load
      
      local.get $b_ptr
      local.get $i
      i32.const 4
      i32.mul
      i32.add
      f32.load
      
      ;; Multiply and add to sum
      f32.mul
      local.get $sum
      f32.add
      local.set $sum
      
      ;; Increment counter
      local.get $i
      i32.const 1
      i32.add
      local.set $i
      
      ;; Continue if not done
      local.get $i
      local.get $len
      i32.lt_u
      br_if $dot_loop
    end
    
    local.get $sum
  )
  
  ;; Matrix multiplication kernel
  (func $matrix_multiply_kernel (export "process_inputs")
    (param $a_ptr i32) (param $b_ptr i32) (param $c_ptr i32)
    (param $m i32) (param $n i32) (param $k i32)
    (local $i i32)
    (local $j i32)
    (local $l i32)
    (local $sum f32)
    
    ;; For each row in A
    i32.const 0
    local.set $i
    
    loop $row_loop
      ;; For each column in B
      i32.const 0
      local.set $j
      
      loop $col_loop
        ;; Initialize sum for C[i][j]
        f32.const 0.0
        local.set $sum
        
        ;; Compute dot product of row i and column j
        i32.const 0
        local.set $l
        
        loop $dot_loop
          ;; A[i][l]
          local.get $a_ptr
          local.get $i
          local.get $n
          i32.mul
          local.get $l
          i32.add
          i32.const 4
          i32.mul
          i32.add
          f32.load
          
          ;; B[l][j]
          local.get $b_ptr
          local.get $l
          local.get $k
          i32.mul
          local.get $j
          i32.add
          i32.const 4
          i32.mul
          i32.add
          f32.load
          
          ;; Multiply and add
          f32.mul
          local.get $sum
          f32.add
          local.set $sum
          
          ;; Next element
          local.get $l
          i32.const 1
          i32.add
          local.set $l
          
          local.get $l
          local.get $n
          i32.lt_u
          br_if $dot_loop
        end
        
        ;; Store result in C[i][j]
        local.get $c_ptr
        local.get $i
        local.get $k
        i32.mul
        local.get $j
        i32.add
        i32.const 4
        i32.mul
        i32.add
        local.get $sum
        f32.store
        
        ;; Next column
        local.get $j
        i32.const 1
        i32.add
        local.set $j
        
        local.get $j
        local.get $k
        i32.lt_u
        br_if $col_loop
      end
      
      ;; Next row
      local.get $i
      i32.const 1
      i32.add
      local.set $i
      
      local.get $i
      local.get $m
      i32.lt_u
      br_if $row_loop
    end
  )
  
  ;; Memory for matrix operations
  (memory (export "memory") 100)
)