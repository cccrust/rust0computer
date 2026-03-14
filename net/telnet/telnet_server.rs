use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:2323").expect("無法綁定 Port");
    println!("Telnet 伺服器已啟動於 127.0.0.1:2323...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("連線失敗: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();
    println!("客戶端已連線: {}", peer_addr);

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    
    // 每個連線獨立維護自己的「當前目錄」
    let mut current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let _ = stream.write_all(b"Welcome to Rust Telnet Server!\r\n");

    loop {
        // 顯示 Prompt (例如: /home/user> )
        let prompt = format!("{}> ", current_dir.display());
        if stream.write_all(prompt.as_bytes()).is_err() { break; }

        let mut input = String::new();
        // 讀取客戶端輸入
        if reader.read_line(&mut input).unwrap_or(0) == 0 {
            break; // 客戶端斷線
        }

        let input = input.trim();
        if input.is_empty() { continue; }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        // 處理內建指令
        if command == "exit" {
            let _ = stream.write_all(b"Goodbye!\r\n");
            break;
        } else if command == "cd" {
            let target_dir = if args.is_empty() {
                PathBuf::from("/") // 預設回到根目錄
            } else {
                current_dir.join(args[0])
            };

            if target_dir.exists() && target_dir.is_dir() {
                // 正規化路徑 (解析 .. 與 .)
                current_dir = target_dir.canonicalize().unwrap_or(target_dir);
            } else {
                let _ = stream.write_all(b"cd: no such file or directory\r\n");
            }
            continue;
        }

        // 處理外部指令 (如 ls, cat 等)
        // 注意：Windows 下如果是 dir，需要用 cmd /c dir
        let output = if cfg!(target_os = "windows") && command == "ls" {
            // 自動幫 Windows 轉譯 ls
            Command::new("cmd")
                .args(["/c", "dir"])
                .current_dir(&current_dir)
                .output()
        } else {
            Command::new(command)
                .args(args)
                .current_dir(&current_dir)
                .output()
        };

        match output {
            Ok(output) => {
                let _ = stream.write_all(&output.stdout);
                let _ = stream.write_all(&output.stderr); // 把錯誤訊息也傳回
            }
            Err(e) => {
                let err_msg = format!("Command execution failed: {}\r\n", e);
                let _ = stream.write_all(err_msg.as_bytes());
            }
        }
    }
    println!("客戶端已斷線: {}", peer_addr);
}