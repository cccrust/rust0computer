; main_add.ll
; main() returns 2 + 40 = 42

define i32 @main() {
entry:
  %a = add i32 2, 40
  ret i32 %a
}
