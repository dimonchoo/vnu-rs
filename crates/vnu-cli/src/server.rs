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
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
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
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();

    if let Err(e) = reader.read_line(&mut line) {
        eprintln!("Failed to read request line: {}", e);
        send_response(&mut stream, "400 Bad Request", "text/plain", "Bad Request");
        return;
    }
    if line.trim().is_empty() {
        return;
    }

    let method = line
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_uppercase();

    let mut content_length = 0usize;

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
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
                    Some(stream) => handle_connection(stream, &mut engine),
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
