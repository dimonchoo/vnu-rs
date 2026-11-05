pub mod assets;
pub mod dispatch;
pub mod parse;
pub mod workers;

use std::net::TcpListener;
use std::thread;
use workers::spawn_worker_pool;

pub fn run_server(port: u16) {
    let bind_addr = std::env::var("VNU_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
    let listener = TcpListener::bind((bind_addr.as_str(), port))
        .unwrap_or_else(|e| panic!("failed to bind to {}:{}: {}", bind_addr, port, e));
    const MAX_WORKERS: usize = 32;
    let worker_count = thread::available_parallelism()
        .map(|n| n.get().clamp(4, MAX_WORKERS))
        .unwrap_or(8);
    println!(
        "vnu-rs server listening on http://{}:{} ({} workers)",
        bind_addr, port, worker_count
    );

    let senders = spawn_worker_pool(worker_count);

    let mut next_worker = 0usize;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let target = next_worker % worker_count;
                next_worker = next_worker.wrapping_add(1);
                if let Err(e) = senders[target].send(stream) {
                    eprintln!("Failed to enqueue connection: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_request_head_browser_html() {
        let req = "GET / HTTP/1.1\r\nHost: localhost:8888\r\nAccept: text/html,application/xhtml+xml\r\n\r\n";
        let mut cursor = Cursor::new(req);
        let mut line_buf = String::new();
        let head = parse_request_head(&mut cursor, &mut line_buf).unwrap();
        assert!(matches!(head.method, HttpMethod::Get));
        assert_eq!(head.path, "/");
        assert!(head.accept_html);
    }

    #[test]
    fn test_parse_request_head_curl_plain() {
        let req = "GET / HTTP/1.1\r\nHost: localhost:8888\r\nUser-Agent: curl/8.7.1\r\nAccept: */*\r\n\r\n";
        let mut cursor = Cursor::new(req);
        let mut line_buf = String::new();
        let head = parse_request_head(&mut cursor, &mut line_buf).unwrap();
        assert!(matches!(head.method, HttpMethod::Get));
        assert_eq!(head.path, "/");
        assert!(!head.accept_html);
    }
}
