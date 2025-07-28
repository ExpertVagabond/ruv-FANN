;; Memory operations test module
(module
  (memory (export "memory") 1)
  
  ;; Sum bytes from memory starting at offset
  (func $sum_bytes (param $offset i32) (param $count i32) (result i32)
    (local $sum i32)
    (local $i i32)
    (local.set $sum (i32.const 0))
    (local.set $i (i32.const 0))
    
    (loop $loop
      (local.get $i)
      (local.get $count)
      (i32.lt_u)
      (if
        (then
          ;; Add byte at offset+i to sum
          (local.set $sum
            (i32.add
              (local.get $sum)
              (i32.load8_u
                (i32.add
                  (local.get $offset)
                  (local.get $i)))))
          ;; Increment i
          (local.set $i
            (i32.add
              (local.get $i)
              (i32.const 1)))
          (br $loop))))
    
    (local.get $sum))
  
  (export "sum_bytes" (func $sum_bytes))
)