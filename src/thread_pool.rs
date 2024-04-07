use std::sync::{Arc, mpsc, Mutex};
use std::thread;

use crate::station_data::StationData;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Vec<u8>>>,
}

impl ThreadPool {
    pub fn new(size: usize, result_sender: mpsc::Sender<StationData>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            workers.push(Worker::new(receiver.clone(), result_sender.clone()));
        }

        Self { workers, sender: Some(sender) }
    }

    pub fn execute(&self, f: Vec<u8>) {
        self.sender.as_ref().unwrap().send(f).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>, result_sender: mpsc::Sender<StationData>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(chunk) => {
                    let data = handle_chunk(chunk);
                    result_sender.send(data).unwrap();
                }
                Err(_) => {
                    break;
                }
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
}

fn handle_chunk(chunk: Vec<u8>) -> StationData {
    let mut data = StationData::new();

    let mut start = 0;
    for i in 0..chunk.len() {
        if chunk[i] == b'\n' {
            data.consume_line(&chunk[start..i]);
            start = i + 1;
        }
    }

    if start < chunk.len() - 1 {
        data.consume_line(&chunk[start..]);
    }

    data
}

