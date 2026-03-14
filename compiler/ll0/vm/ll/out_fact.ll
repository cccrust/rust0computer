define i32 @fact(i32 %n) {
entry:
  %t1 = icmp sle i32 %n, 1
  br i1 %t1, label %then, label %else

then:
  ret i32 1

else:
  %t2 = sub i32 %n, 1
  %t3 = call i32 @fact(i32 %t2)
  %t4 = mul i32 %n, %t3
  ret i32 %t4

}

define i32 @main() {
entry:
  %t1 = call i32 @fact(i32 5)
  ret i32 %t1

}

