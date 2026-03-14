define i32 @main() {
entry:
  %c.addr = alloca i1
  %b.addr = alloca i1
  %a.addr = alloca i1
  store i1 true, i1* %a.addr
  store i1 false, i1* %b.addr
  %t1 = load i1, i1* %a.addr
  %t2 = load i1, i1* %b.addr
  %t3 = xor i1 %t2, true
  %t4 = and i1 %t1, %t3
  %t5 = load i1, i1* %a.addr
  %t6 = load i1, i1* %b.addr
  %t7 = icmp eq i1 %t5, %t6
  %t8 = or i1 %t4, %t7
  store i1 %t8, i1* %c.addr
  %t9 = load i1, i1* %c.addr
  br i1 %t9, label %then1, label %else2

then1:
  br label %merge3

else2:
  br label %merge3

merge3:
  %t10 = phi i32 [ 1, %then1 ], [ 0, %else2 ]
  ret i32 %t10

}

