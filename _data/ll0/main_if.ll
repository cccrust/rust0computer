define i32 @main() {
entry:
  %ok.addr = alloca i1
  %y.addr = alloca i32
  %x.addr = alloca i32
  store i32 3, i32* %x.addr
  store i32 2, i32* %y.addr
  %t1 = load i32, i32* %x.addr
  %t2 = load i32, i32* %y.addr
  %t3 = icmp sgt i32 %t1, %t2
  store i1 %t3, i1* %ok.addr
  %t4 = load i1, i1* %ok.addr
  br i1 %t4, label %then1, label %else2

then1:
  br label %merge3

else2:
  br label %merge3

merge3:
  %t5 = phi i32 [ 7, %then1 ], [ 0, %else2 ]
  ret i32 %t5

}

