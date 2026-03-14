define i32 @fact(i32 %n) {
entry:
  %t1 = icmp sle i32 %n, 1
  br i1 %t1, label %then1, label %else2

then1:
  br label %merge3

else2:
  %t2 = sub i32 %n, 1
  %t3 = call i32 @fact(i32 %t2)
  %t4 = mul i32 %n, %t3
  br label %merge3

merge3:
  %t5 = phi i32 [ 1, %then1 ], [ %t4, %else2 ]
  ret i32 %t5

}

define i32 @main() {
entry:
  %t1 = call i32 @fact(i32 5)
  ret i32 %t1

}

