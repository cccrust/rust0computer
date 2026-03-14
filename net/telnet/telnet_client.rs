use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:2323").expect("無法連線到伺服器");
    println!("成功連線至伺服器！輸入 'exit' 離開。");

    // 複製 stream 以便在另一個執行緒讀取
    let mut stream_clone = stream.try_clone().expect("無法複製 stream");

    // 執行緒 1: 負責接收伺服器訊息並印出
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match stream_clone.read(&mut buffer) {
                Ok(0) => {
                    println!("\n伺服器已關閉連線。");
                    std::process::exit(0);
                }
                Ok(n) => {
                    // 使用 from_utf8_lossy 避免不合法的字節導致崩潰
                    let response = String::from_utf8_lossy(&buffer[..n]);
                    print!("{}", response);
                    io::stdout().flush().unwrap(); // 確保 prompt 立即印出
                }
                Err(_) => break,
            }
        }
    });

    // 主執行緒: 負責讀取鍵盤輸入並發送給伺服器
    let stdin = io::stdin();
    let mut input = String::new();
    
    loop {
        input.clear();
        if stdin.read_line(&mut input).unwrap_or(0) == 0 {
            break;
        }
        
        if stream.write_all(input.as_bytes()).is_err() {
            println!("發送訊息失敗，斷開連線。");
            break;
        }
    }
}