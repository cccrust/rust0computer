fn main() -> i32 {
    let x: i32 = 10;
    let y: i32 = if x > 5 { x + 1 } else { x - 1 };
    y * 2
}
