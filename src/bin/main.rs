// #![allow(dead_code)]
extern crate rust_webserver_example;
use rust_webserver_example::ThreadPool;

// use std::io;
use std::thread;
use std::time::Duration;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(2);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        pool.execute(|| {
            handle(stream);
        });
    }
}

fn handle(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();
    // println!("{}", String::from_utf8_lossy(&buf));

    let get   = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // parse request types
    let (status, filename) = if buf.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", PathBuf::from("hello.html"))
    } else if buf.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5)); // wait for a bit
        ("HTTP/1.1 200 OK\r\n\r\n", PathBuf::from("hello.html"))
    } else { // look for file
        // ("HTTP/1.1 404 NOT FOUND\r\n\r\n", PathBuf::from("404.html"))
        let p = find_file(&String::from_utf8_lossy(&buf));
        if let Some(p) = p {
            if p.is_file() {
                ("HTTP/1.1 200 OK\r\n\r\n", p)
            } else {
                ("HTTP/1.1 404 NOT FOUND\r\n\r\n", PathBuf::from("404.html"))
            }
        } else {
            ("HTTP/1.1 404 NOT FOUND\r\n\r\n", PathBuf::from("404.html"))
        }
    };

    let response = format!("{}{}\r\n", status, file_to_string(filename));
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn file_to_string<P: AsRef<Path>>(p: P) -> String {
    let mut contents = String::new();
    let mut file = File::open(p).unwrap();
    file.read_to_string(&mut contents).unwrap();
    contents
}

fn find_file(request: &str) -> Option<PathBuf> {
    let server_root = PathBuf::from(".");
    let file_path = request.split_whitespace()
                            .nth(1).unwrap();
    // println!("{}", file_path);
    let path = server_root.join(&file_path[1..]);
    println!("Request for file {:?}", path);

    let path = path.canonicalize();
    if path.is_err() {
        return None
    } else {
        return Some(path.unwrap())
    }
}
