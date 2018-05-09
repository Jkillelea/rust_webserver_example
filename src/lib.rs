#![allow(dead_code)]
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

/// A threadsafe, heap allocated closure
type Job = Box<FnBox + Send + 'static>;

/// Possible message types; Execute a job or die
enum Message {
    NewJob(Job),
    Terminate,
}

/// A thread worker
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

/// The ThreadPool, handles starting and stopping workers, as well as
/// sending jobs
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

/// Makes a heap allocated closure callable
trait FnBox {
    fn call_box(self: Box<Self>);
}
impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let mut workers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F) // execute a task. 
        where F: FnOnce() + Send + 'static // The closure must be statically allocated
    {
        let job = Box::new(f);
        self.sender.send(
            Message::NewJob(job)
        ).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) { // Tell each worker to shut down
        println!("Sending terminate message to all workers.");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");
        
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap()
            }
        }
    }
}


impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job", id);
                        job.call_box();
                    },
                    Message::Terminate => {
                        println!("Worker {} is terminating", id);
                        break
                    },
                }
            }
        });
        Worker { id: id, thread: Some(thread) }   
    }
}