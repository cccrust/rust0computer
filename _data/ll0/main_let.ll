define i32 @main() {
entry:
  %y.addr = alloca i32
  %x.addr = alloca i32
  store i32 10, i32* %x.addr
  %t1 = load i32, i32* %x.addr
  %t2 = icmp sgt i32 %t1, 5
  br i1 %t2, label %then1, label %else2

then1:
  %t3 = load i32, i32* %x.addr
  %t4 = add i32 %t3, 1
  br label %merge3

else2:
  %t5 = load i32, i32* %x.addr
  %t6 = sub i32 %t5, 1
  br label %merge3

merge3:
  %t7 = phi i32 [ %t4, %then1 ], [ %t6, %else2 ]
  store i32 %t7, i32* %y.addr
  %t8 = load i32, i32* %y.addr
  %t9 = mul i32 %t8, 2
  ret i32 %t9

}

