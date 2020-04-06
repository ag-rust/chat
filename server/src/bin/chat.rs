use chat::broker::{Broker, BrokerEvent};
use chat::user::{User, UserId};
use flume::{Receiver, Sender};
use std::net::{TcpListener, TcpStream};
use std::io::BufReader;
use std::io::prelude::*;
use protocol::{Identify, Send, Message};
use serde_json;
use std::sync::Arc;

fn handle_client(stream: TcpStream, sender: Sender<BrokerEvent>) {    
    let mut reader = BufReader::new(&stream);

    let mut identify_line = String::new();
    reader.read_line(&mut identify_line).unwrap();
    let (client_sender, client_receiver) = flume::unbounded();

    
    let user = match serde_json::from_str::<Identify>(&identify_line) {
        Ok(identify) => {
            let user = User { display_name: identify.display_name.clone(), id: UserId(identify.display_name) };
            sender.send(BrokerEvent::Connect(user.clone(), client_sender));
            user
        },
        Err(e) => {
            return;
        },
    };

    let shared_stream = Arc::new(stream);
    let ingoing_stream = shared_stream.clone();
    let _ingoing = std::thread::spawn(move || {
        loop {
            let mut reader = BufReader::new(&*ingoing_stream);
            let mut message_line = String::new();
            reader.read_line(&mut message_line).unwrap();

            if let Ok(send) = serde_json::from_str::<Send>(&message_line) {
                sender.send(BrokerEvent::Sending { id: user.id.clone(), text: send.message.clone() });
            }
        }
    });

    let outgoing_stream = shared_stream.clone();
    let _outgoing = std::thread::spawn(move || {
        for message in client_receiver {
            let message = Message { payload: message};
            serde_json::to_writer(&*outgoing_stream, &message).unwrap();
            write!(&*outgoing_stream, "\n").unwrap();
        }

    });

}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    let broker = Broker::new();
    let (broker_sender, broker_receiver) = flume::unbounded();

    std::thread::spawn(move || broker_loop(broker, broker_receiver));

    // accept connections and process them serially
    for stream in listener.incoming() {
        let cloned_sender = broker_sender.clone();
        std::thread::spawn(move || {
            handle_client(stream.unwrap(), cloned_sender);
        });
    }
    Ok(())
}

fn broker_loop(mut broker: Broker, messages: Receiver<BrokerEvent>) -> Broker {
    while let Ok(message) = messages.recv() {
        match message {
            BrokerEvent::Stop => break,
            _ => {}
        }

        broker.handle_message(message)
    }
    broker
}

#[test]
fn test_broker_loop() {
    let broker = Broker::new();
    let (client1_sender, _) = flume::unbounded();
    let (client2_sender, client2_receiver) = flume::unbounded();
    let (broker_sender, broker_receiver) = flume::unbounded();

    std::thread::spawn(move || broker_loop(broker, broker_receiver));

    let user = User {
        id: UserId("skade".into()),
        display_name: "Skade".into(),
    };
    let user2 = User {
        id: UserId("other".into()),
        display_name: "Other".into(),
    };

    broker_sender.send(BrokerEvent::Connect(user.clone(), client1_sender));
    broker_sender.send(BrokerEvent::Connect(user2.clone(), client2_sender));

    broker_sender.send(BrokerEvent::Sending {
        id: user.id,
        text: "Hello!".into(),
    });

    assert_eq!(String::from("Hello!"), client2_receiver.recv().unwrap());
}
