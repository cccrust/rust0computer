// Covers: true/false, logical ops, equality

fn main() -> i32 {
    let a: bool = true;
    let b: bool = false;
    let c: bool = (a && !b) || (a == b);
    if c { 1 } else { 0 }
}
