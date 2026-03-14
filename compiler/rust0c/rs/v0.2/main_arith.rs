// Covers: fn, let, i32, arithmetic ops, call

fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn main() -> i32 {
    let a: i32 = 20;
    let b: i32 = 22;
    add(a, b) * 2 - 2
}
