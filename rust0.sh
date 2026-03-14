cargo run -p rust0c -- _data/rs0/$1.rs -o _data/ll0/$1.ll
cargo run -p ll0vm -- _data/ll0/$1.ll
