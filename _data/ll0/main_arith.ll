define i32 @add(i32 %x, i32 %y) {
entry:
  %t1 = add i32 %x, %y
  ret i32 %t1

}

define i32 @main() {
entry:
  %b.addr = alloca i32
  %a.addr = alloca i32
  store i32 20, i32* %a.addr
  store i32 22, i32* %b.addr
  %t1 = load i32, i32* %a.addr
  %t2 = load i32, i32* %b.addr
  %t3 = call i32 @add(i32 %t1, i32 %t2)
  %t4 = mul i32 %t3, 2
  %t5 = sub i32 %t4, 2
  ret i32 %t5

}

