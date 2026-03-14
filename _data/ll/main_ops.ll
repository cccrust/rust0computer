define i32 @main() {
entry:
  %logic.addr = alloca i1
  %ne.addr = alloca i1
  %eq.addr = alloca i1
  %ge.addr = alloca i1
  %gt.addr = alloca i1
  %le.addr = alloca i1
  %lt.addr = alloca i1
  %rem.addr = alloca i32
  %div.addr = alloca i32
  %mul.addr = alloca i32
  %sub.addr = alloca i32
  %add.addr = alloca i32
  %b.addr = alloca i32
  %a.addr = alloca i32
  store i32 10, i32* %a.addr
  store i32 3, i32* %b.addr
  %t1 = load i32, i32* %a.addr
  %t2 = load i32, i32* %b.addr
  %t3 = add i32 %t1, %t2
  store i32 %t3, i32* %add.addr
  %t4 = load i32, i32* %a.addr
  %t5 = load i32, i32* %b.addr
  %t6 = sub i32 %t4, %t5
  store i32 %t6, i32* %sub.addr
  %t7 = load i32, i32* %a.addr
  %t8 = load i32, i32* %b.addr
  %t9 = mul i32 %t7, %t8
  store i32 %t9, i32* %mul.addr
  %t10 = load i32, i32* %a.addr
  %t11 = load i32, i32* %b.addr
  %t12 = sdiv i32 %t10, %t11
  store i32 %t12, i32* %div.addr
  %t13 = load i32, i32* %a.addr
  %t14 = load i32, i32* %b.addr
  %t15 = srem i32 %t13, %t14
  store i32 %t15, i32* %rem.addr
  %t16 = load i32, i32* %a.addr
  %t17 = load i32, i32* %b.addr
  %t18 = icmp slt i32 %t16, %t17
  store i1 %t18, i1* %lt.addr
  %t19 = load i32, i32* %a.addr
  %t20 = load i32, i32* %b.addr
  %t21 = icmp sle i32 %t19, %t20
  store i1 %t21, i1* %le.addr
  %t22 = load i32, i32* %a.addr
  %t23 = load i32, i32* %b.addr
  %t24 = icmp sgt i32 %t22, %t23
  store i1 %t24, i1* %gt.addr
  %t25 = load i32, i32* %a.addr
  %t26 = load i32, i32* %b.addr
  %t27 = icmp sge i32 %t25, %t26
  store i1 %t27, i1* %ge.addr
  %t28 = load i32, i32* %a.addr
  %t29 = load i32, i32* %b.addr
  %t30 = icmp eq i32 %t28, %t29
  store i1 %t30, i1* %eq.addr
  %t31 = load i32, i32* %a.addr
  %t32 = load i32, i32* %b.addr
  %t33 = icmp ne i32 %t31, %t32
  store i1 %t33, i1* %ne.addr
  %t34 = load i1, i1* %gt.addr
  %t35 = load i1, i1* %ge.addr
  %t36 = and i1 %t34, %t35
  %t37 = load i1, i1* %lt.addr
  %t38 = xor i1 %t37, true
  %t39 = load i1, i1* %ne.addr
  %t40 = and i1 %t38, %t39
  %t41 = or i1 %t36, %t40
  store i1 %t41, i1* %logic.addr
  %t42 = load i1, i1* %logic.addr
  br i1 %t42, label %then1, label %else2

then1:
  %t43 = load i32, i32* %add.addr
  %t44 = load i32, i32* %sub.addr
  %t45 = add i32 %t43, %t44
  %t46 = load i32, i32* %mul.addr
  %t47 = add i32 %t45, %t46
  %t48 = load i32, i32* %div.addr
  %t49 = add i32 %t47, %t48
  %t50 = load i32, i32* %rem.addr
  %t51 = add i32 %t49, %t50
  br label %merge3

else2:
  br label %merge3

merge3:
  %t52 = phi i32 [ %t51, %then1 ], [ 0, %else2 ]
  ret i32 %t52

}

