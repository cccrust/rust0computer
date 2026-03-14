define i32 @main() {
entry:
  %t1 = icmp sgt i32 3, 2
  br i1 %t1, label %then, label %else

then:
  ret i32 7

else:
  ret i32 0

}

