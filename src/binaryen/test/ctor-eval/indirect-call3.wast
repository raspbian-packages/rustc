(module
  (type $v (func))
  (memory 256 256)
  (data (i32.const 10) "waka waka waka waka waka")
  (import "env" "tableBase" (global $tableBase i32))
  (import "env" "_abort" (func $_abort))
  (table 2 2 anyfunc)
  (elem (get_global $tableBase) $_abort $call-indirect)
  (export "test1" $test1)
  (func $test1
    (call_indirect (type $v) (i32.const 1)) ;; safe to call
    (i32.store8 (i32.const 20) (i32.const 120))
  )
  (func $call-indirect
    (i32.store8 (i32.const 40) (i32.const 67))
  )
)
