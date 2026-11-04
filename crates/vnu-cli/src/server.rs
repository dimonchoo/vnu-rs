use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use vnu_parser::ValidatorEngine;
use crate::service::validate_content;

const MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10 MB limit
const WORKER_COUNT: usize = 4;
const QUEUE_CAPACITY: usize = 32; // Bounded channel for backpressure

fn send_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, User-Agent\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write HTTP response: {}", e);
    }
}

fn handle_connection(mut stream: TcpStream, engine: &mut ValidatorEngine) {
    if let Err(e) = stream.set_read_timeout(Some(std::time::Duration::from_secs(5))) {
        eprintln!("Failed to set socket read timeout: {}", e);
        return;
    }
    if let Err(e) = stream.set_write_timeout(Some(std::time::Duration::from_secs(5))) {
        eprintln!("Failed to set socket write timeout: {}", e);
        return;
    }

    const MAX_HEADER_BYTES: usize = 64 * 1024;
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();

    // Cap the request line reading to MAX_HEADER_BYTES + 1
    match reader.by_ref().take((MAX_HEADER_BYTES + 1) as u64).read_line(&mut line) {
        Ok(n) if n > MAX_HEADER_BYTES => {
            send_response(&mut stream, "431 Request Header Fields Too Large", "text/plain", "Headers Too Large");
            return;
        }
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to read request line: {}", e);
            send_response(&mut stream, "400 Bad Request", "text/plain", "Bad Request");
            return;
        }
    }
    if line.trim().is_empty() {
        return;
    }

    let method = line
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_uppercase();

    if method == "OPTIONS" {
        send_response(&mut stream, "204 No Content", "text/plain", "");
        return;
    }

    let mut content_length = 0usize;
    let mut total_header_bytes = line.len();

    loop {
        line.clear();
        let remaining_cap = (MAX_HEADER_BYTES.saturating_sub(total_header_bytes) + 1) as u64;
        match reader.by_ref().take(remaining_cap).read_line(&mut line) {
            Ok(0) => break,
            Ok(n) => {
                total_header_bytes += n;
                if total_header_bytes > MAX_HEADER_BYTES {
                    send_response(&mut stream, "431 Request Header Fields Too Large", "text/plain", "Headers Too Large");
                    return;
                }
                if line.trim().is_empty() {
                    break;
                }
                if let Some((k, v)) = line.split_once(':') {
                    if k.trim().eq_ignore_ascii_case("content-length") {
                        content_length = v.trim().parse().unwrap_or(0);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed reading headers: {}", e);
                send_response(&mut stream, "400 Bad Request", "text/plain", "Bad Request");
                return;
            }
        }
    }

    if method == "GET" {
        send_response(&mut stream, "200 OK", "text/plain", "vnu-rs is running");
        return;
    }

    if method == "POST" {
        if content_length > MAX_BODY_SIZE {
            eprintln!("Payload too large: {} bytes (max {})", content_length, MAX_BODY_SIZE);
            send_response(&mut stream, "413 Payload Too Large", "text/plain", "Payload Too Large");
            return;
        }

        let mut body = vec![0u8; content_length];
        if let Err(e) = reader.read_exact(&mut body) {
            eprintln!("Failed to read request body: {}", e);
            send_response(&mut stream, "400 Bad Request", "text/plain", "Failed to read body");
            return;
        }

        let body_str = String::from_utf8_lossy(&body);
        match validate_content(engine, &body_str, false) {
            Ok((_report, json)) => {
                send_response(&mut stream, "200 OK", "application/json; charset=utf-8", &json);
            }
            Err(e) => {
                eprintln!("Internal validation error: {}", e);
                send_response(&mut stream, "500 Internal Server Error", "text/plain", "Internal Server Error");
            }
        }
        return;
    }

    send_response(&mut stream, "405 Method Not Allowed", "text/plain", "Method Not Allowed");
}

pub fn run_server(port: u16) {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .unwrap_or_else(|e| panic!("failed to bind to 127.0.0.1:{}: {}", port, e));
    println!("vnu-rs server listening on http://127.0.0.1:{}", port);

    let (tx, rx): (SyncSender<TcpStream>, Receiver<TcpStream>) = sync_channel(QUEUE_CAPACITY);
    let rx_arc = Arc::new(Mutex::new(rx));

    for worker_id in 0..WORKER_COUNT {
        let rx_clone = Arc::clone(&rx_arc);
        thread::spawn(move || {
            let mut engine = ValidatorEngine::default();
            loop {
                let stream_opt = {
                    let lock = rx_clone.lock();
                    match lock {
                        Ok(receiver) => receiver.recv().ok(),
                        Err(_) => break,
                    }
                };

                match stream_opt {
                    Some(stream) => {
                        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            handle_connection(stream, &mut engine);
                        }));
                        if res.is_err() {
                            eprintln!("Worker {} recovered from panic, resetting engine", worker_id);
                            engine = ValidatorEngine::default();
                        }
                    }
                    None => break,
                }
            }
            eprintln!("Worker {} terminating", worker_id);
        });
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = tx.send(stream) {
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
