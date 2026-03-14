// Covers: while statement, block rule

fn main() -> i32 {
    let i: i32 = 0;
    while false { 0 };
    i + 42
}
