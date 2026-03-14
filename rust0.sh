cargo run -p rust0c -- _data/rs/$1.rs -o _data/ll/$1.ll
cargo run -p ir0vm -- _data/ll/$1.ll