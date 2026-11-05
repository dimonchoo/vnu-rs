use super::assets::INDEX_HTML;
use super::parse::{
    HttpMethod, ParseHeadError, RequestHead, parse_request_head, read_post_body, reset_body_buffer,
};
use crate::service::validate_content;
use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;
use vnu_parser::ValidatorEngine;

pub fn send_response(
    stream: &mut TcpStream,
    resp_buf: &mut Vec<u8>,
    status: &str,
    content_type: &str,
    body: &str,
    keep_alive: bool,
) -> bool {
    let conn_hdr = if keep_alive { "keep-alive" } else { "close" };
    resp_buf.clear();
    let _ = write!(
        resp_buf,
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, User-Agent\r\nConnection: {}\r\n\r\n",
        status,
        content_type,
        body.len(),
        conn_hdr
    );
    resp_buf.extend_from_slice(body.as_bytes());
    if let Err(e) = stream.write_all(resp_buf) {
        eprintln!("Failed to write HTTP response: {}", e);
        return false;
    }
    true
}

pub fn dispatch_request(
    head: RequestHead,
    reader: &mut BufReader<TcpStream>,
    write_stream: &mut TcpStream,
    engine: &mut ValidatorEngine,
    body_buf: &mut Vec<u8>,
    resp_buf: &mut Vec<u8>,
) -> bool {
    let keep_alive = head.keep_alive;
    match head.method {
        HttpMethod::Options => {
            send_response(
                write_stream,
                resp_buf,
                "204 No Content",
                "text/plain",
                "",
                keep_alive,
            ) && keep_alive
        }
        HttpMethod::Get => {
            let serves_html = (head.accept_html || head.path == "/index.html")
                && !head.path.contains("out=text")
                && head.path != "/healthz";

            if serves_html {
                send_response(
                    write_stream,
                    resp_buf,
                    "200 OK",
                    "text/html; charset=utf-8",
                    &INDEX_HTML,
                    keep_alive,
                ) && keep_alive
            } else {
                send_response(
                    write_stream,
                    resp_buf,
                    "200 OK",
                    "text/plain",
                    "vnu-rs is running",
                    keep_alive,
                ) && keep_alive
            }
        }
        HttpMethod::Post => {
            let res = if let Err(status) = read_post_body(reader, head.content_length, body_buf) {
                let msg = if status.starts_with("413") {
                    "Payload Too Large"
                } else {
                    "Failed to read body"
                };
                send_response(write_stream, resp_buf, status, "text/plain", msg, false);
                false
            } else {
                let body_str = String::from_utf8_lossy(body_buf);
                match validate_content(engine, &body_str, false) {
                    Ok((_report, json)) => {
                        send_response(
                            write_stream,
                            resp_buf,
                            "200 OK",
                            "application/json; charset=utf-8",
                            &json,
                            keep_alive,
                        ) && keep_alive
                    }
                    Err(e) => {
                        eprintln!("Internal validation error: {}", e);
                        send_response(
                            write_stream,
                            resp_buf,
                            "500 Internal Server Error",
                            "text/plain",
                            "Internal Server Error",
                            false,
                        );
                        false
                    }
                }
            };
            reset_body_buffer(body_buf);
            res
        }
        HttpMethod::Other => {
            send_response(
                write_stream,
                resp_buf,
                "405 Method Not Allowed",
                "text/plain",
                "Method Not Allowed",
                keep_alive,
            ) && keep_alive
        }
    }
}

pub fn handle_connection(
    stream: TcpStream,
    engine: &mut ValidatorEngine,
    line_buf: &mut String,
    body_buf: &mut Vec<u8>,
    resp_buf: &mut Vec<u8>,
) {
    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("Failed to set TCP_NODELAY: {}", e);
    }
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(5))) {
        eprintln!("Failed to set read timeout: {}", e);
    }
    if let Err(e) = stream.set_write_timeout(Some(Duration::from_secs(5))) {
        eprintln!("Failed to set write timeout: {}", e);
    }

    let mut write_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to clone socket stream: {}", e);
            return;
        }
    };
    let mut reader = BufReader::with_capacity(16 * 1024, stream);

    loop {
        match parse_request_head(&mut reader, line_buf) {
            Ok(head) => {
                if !dispatch_request(
                    head,
                    &mut reader,
                    &mut write_stream,
                    engine,
                    body_buf,
                    resp_buf,
                ) {
                    break;
                }
            }
            Err(ParseHeadError::Eof) => break,
            Err(ParseHeadError::HeadersTooLarge) => {
                send_response(
                    &mut write_stream,
                    resp_buf,
                    "431 Request Header Fields Too Large",
                    "text/plain",
                    "Headers Too Large",
                    false,
                );
                break;
            }
            Err(ParseHeadError::BadRequest) => {
                send_response(
                    &mut write_stream,
                    resp_buf,
                    "400 Bad Request",
                    "text/plain",
                    "Bad Request",
                    false,
                );
                break;
            }
        }
    }
}
