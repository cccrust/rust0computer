; main_fact.ll
; main() returns fact(5) = 120

define i32 @fact(i32 %n) {
entry:
  %cond = icmp sle i32 %n, 1
  br i1 %cond, label %base, label %recurse

base:
  ret i32 1

recurse:
  %n1 = sub i32 %n, 1
  %f = call i32 @fact(i32 %n1)
  %res = mul i32 %n, %f
  ret i32 %res
}

define i32 @main() {
entry:
  %v = call i32 @fact(i32 5)
  ret i32 %v
}
