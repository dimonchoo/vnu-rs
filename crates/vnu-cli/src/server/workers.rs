use super::dispatch::handle_connection;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread;
use vnu_parser::ValidatorEngine;

pub const PER_WORKER_QUEUE_CAPACITY: usize = 128;

pub fn spawn_worker_pool(worker_count: usize) -> Vec<SyncSender<TcpStream>> {
    let mut senders: Vec<SyncSender<TcpStream>> = Vec::with_capacity(worker_count);

    for worker_id in 0..worker_count {
        let (tx, rx): (SyncSender<TcpStream>, Receiver<TcpStream>) =
            sync_channel(PER_WORKER_QUEUE_CAPACITY);
        senders.push(tx);

        thread::spawn(move || {
            let mut engine = ValidatorEngine::default();
            let mut line_buf = String::with_capacity(1024);
            let mut body_buf = Vec::with_capacity(16 * 1024);
            let mut resp_buf = Vec::with_capacity(16 * 1024);

            while let Ok(stream) = rx.recv() {
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handle_connection(
                        stream,
                        &mut engine,
                        &mut line_buf,
                        &mut body_buf,
                        &mut resp_buf,
                    );
                }));
                if res.is_err() {
                    eprintln!(
                        "Worker {} recovered from panic, resetting engine",
                        worker_id
                    );
                    engine = ValidatorEngine::default();
                }
            }
            eprintln!("Worker {} terminating", worker_id);
        });
    }

    senders
}
