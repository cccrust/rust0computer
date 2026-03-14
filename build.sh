set -x
cargo build -p rust0c
cargo build -p ll0vm
cargo build -p nn0

rustc --crate-type=staticlib compiler/qd0/qd0lib.rs -o bin/qd0lib.a
rustc compiler/qd0/qd0c.rs -o bin/qd0c
rustc compiler/qd0/qd0vm.rs -o bin/qd0vm
rustc compiler/js0/js0c.rs -o bin/js0c

rustc net/telnet/telnet_client.rs -o bin/telnet
rustc net/telnet/telnet_server.rs -o bin/telnet_server
