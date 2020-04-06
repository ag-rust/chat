use client::Client;
use std::sync::Arc;
use std::io;

fn main() {
    let name = std::env::args().nth(1).unwrap();
    let client = Arc::new(Client::start("127.0.0.1:8080", &name).unwrap());

    println!("started client");

    let reader_client = client.clone();
    let reader = std::thread::spawn(move || {
        loop {
            let message = reader_client.read_line().unwrap();
            println!("received message: {}", message);
        }
    });

    let writer_client = client.clone();
    let writer = std::thread::spawn(move || {
        loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            writer_client.write_msg(&buffer.trim()).unwrap();
        }
    });

    reader.join();
    writer.join();
}
