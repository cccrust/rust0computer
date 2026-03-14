define i32 @main() {
entry:
  %b.addr = alloca i32
  %a.addr = alloca i32
  %v.addr = alloca i32
  store i32 10, i32* %a.addr
  store i32 20, i32* %b.addr
  %t1 = load i32, i32* %a.addr
  %t2 = load i32, i32* %b.addr
  %t3 = add i32 %t1, %t2
  store i32 %t3, i32* %v.addr
  %t4 = load i32, i32* %v.addr
  ret i32 %t4

}

