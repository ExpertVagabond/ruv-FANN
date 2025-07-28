;; SIMD operations test module
(module
  (memory 1)
  
  ;; Vector add using SIMD
  (func $vector_add (result i32)
    ;; Load two v128 vectors from memory
    (v128.const i32x4 1 2 3 4)
    (v128.const i32x4 5 6 7 8)
    
    ;; Add them
    i32x4.add
    
    ;; Extract and sum all lanes
    (i32x4.extract_lane 0)
    (i32x4.extract_lane 1)
    (i32.add)
    (i32x4.extract_lane 2)
    (i32.add)
    (i32x4.extract_lane 3)
    (i32.add))
  
  (export "vector_add" (func $vector_add))
)