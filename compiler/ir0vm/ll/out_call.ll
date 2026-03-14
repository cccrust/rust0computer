define i32 @add(i32 %x, i32 %y) {
entry:
  %t1 = add i32 %x, %y
  ret i32 %t1

}

define i32 @main() {
entry:
  %t1 = call i32 @add(i32 10, i32 32)
  ret i32 %t1

}

