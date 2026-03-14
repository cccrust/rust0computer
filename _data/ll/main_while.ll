define i32 @main() {
entry:
  %i.addr = alloca i32
  store i32 0, i32* %i.addr
  br label %cond1

cond1:
  br i1 false, label %body2, label %exit3

body2:
  br label %cond1

exit3:
  %t1 = load i32, i32* %i.addr
  %t2 = add i32 %t1, 42
  ret i32 %t2

}

