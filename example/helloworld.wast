;; WebAssembly WASM AST Hello World! program

(module
  (import "std" "fwrite" (func $fwrite (param i32 i32 i32 i32) (result i32)))
  (import "std" "stdout" (global $stdoutPtr i32))
  (memory 1)
  (export "main" (func $main))

  (data (i32.const 8) "Hello World!\n")

  (func (export "establishStackSpace") (param i32 i32) (nop))

  (func $main (result i32)
    (local $stdout i32)
    (set_local $stdout (i32.load align=4 (get_global $stdoutPtr)))

    (return (call $fwrite
       (i32.const 8)         ;; void *ptr    => Address of our string
       (i32.const 1)         ;; size_t size  => Data size
       (i32.const 13)        ;; size_t nmemb => Length of our string
       (get_local $stdout))  ;; stream
    )
  )
)
