(module
  ;; Simple multiplication module
  (func $multiply (export "compute") (param $a f32) (param $b f32) (result f32)
    local.get $a
    local.get $b
    f32.mul
  )
  
  ;; Memory for shared data
  (memory (export "memory") 1)
)