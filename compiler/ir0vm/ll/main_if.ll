; main_if.ll
; main() returns 7 if 3 > 2 else 0

define i32 @main() {
entry:
  %cond = icmp sgt i32 3, 2
  br i1 %cond, label %then, label %else

then:
  ret i32 7

else:
  ret i32 0
}
