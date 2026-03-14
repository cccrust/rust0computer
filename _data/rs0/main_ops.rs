// Covers: all arithmetic ops, all comparisons, and logical ops in one file

fn main() -> i32 {
    let a: i32 = 10;
    let b: i32 = 3;

    let add: i32 = a + b;   // 13
    let sub: i32 = a - b;   // 7
    let mul: i32 = a * b;   // 30
    let div: i32 = a / b;   // 3
    let rem: i32 = a % b;   // 1

    let lt: bool = a < b;
    let le: bool = a <= b;
    let gt: bool = a > b;
    let ge: bool = a >= b;
    let eq: bool = a == b;
    let ne: bool = a != b;

    let logic: bool = (gt && ge) || (!lt && ne);

    if logic { add + sub + mul + div + rem } else { 0 }
}
