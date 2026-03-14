// Covers: block expression and nested let

fn main() -> i32 {
    let v: i32 = {
        let a: i32 = 10;
        let b: i32 = 20;
        a + b
    };
    v
}
