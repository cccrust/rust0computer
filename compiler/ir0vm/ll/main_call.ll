; main_call.ll
; main() calls add(10, 32) => 42

define i32 @add(i32 %x, i32 %y) {
entry:
  %sum = add i32 %x, %y
  ret i32 %sum
}

define i32 @main() {
entry:
  %v = call i32 @add(i32 10, i32 32)
  ret i32 %v
}
