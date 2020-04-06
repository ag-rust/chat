pub mod filters;

use flume::{Receiver, Sender};
use std::collections::HashMap;

use filters::{Filter, FilterMessage, LengthFilter};

use crate::user::{User, UserId};

pub type ChannelMessage = String;

pub type BackChannel = Sender<ChannelMessage>;

pub enum BrokerEvent {
    Sending { id: UserId, text: String },
    Connect(User, BackChannel),
    Disconnect(User),
    Stop,
}

pub struct Broker {
    connections: HashMap<UserId, BackChannel>,
    filters: Vec<Box<dyn Filter + Send + Sync + 'static>>,
}

impl Broker {
    pub fn new() -> Broker {
        Broker {
            connections: HashMap::new(),
            filters: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn Filter + Send + Sync + 'static>) {
        self.filters.push(filter);
    }

    pub fn handle_message(&mut self, message: BrokerEvent) {
        match message {
            BrokerEvent::Sending {
                id: user_id,
                text: text,
            } => {
                println!("Sending message: {}", text);

                for filter in &self.filters {
                    match filter.filter(&text) {
                        FilterMessage::Yes => return,
                        FilterMessage::No => {}
                    }
                }
                for (u, back_channel) in &self.connections {
                    if u == &user_id {
                        continue;
                    }

                    back_channel.send(text.clone()).unwrap();
                }
            }
            BrokerEvent::Connect(user, back_channel) => {
                println!("Client connected: {}", user.display_name);
                self.connections.insert(user.id, back_channel);
            }
            BrokerEvent::Disconnect(user) => {
                self.connections.remove(&user.id);
            }
            BrokerEvent::Stop => unreachable!(),
        }
    }
}

#[test]
fn test_connect_disconnect() {
    let mut broker = Broker::new();
    let (client_sender, client_receiver) = flume::unbounded();

    let user = User {
        id: UserId("skade".into()),
        display_name: "Skade".into(),
    };
    broker.handle_message(BrokerEvent::Connect(user.clone(), client_sender));

    assert_eq!(broker.connections.len(), 1);

    broker.handle_message(BrokerEvent::Disconnect(user));

    assert_eq!(broker.connections.len(), 0);
}

#[test]
fn test_sending() {
    let mut broker = Broker::new();
    let (client1_sender, _) = flume::unbounded();
    let (client2_sender, client2_receiver) = flume::unbounded();

    let user = User {
        id: UserId("skade".into()),
        display_name: "Skade".into(),
    };
    let user2 = User {
        id: UserId("other".into()),
        display_name: "Other".into(),
    };

    broker.handle_message(BrokerEvent::Connect(user.clone(), client1_sender));
    broker.handle_message(BrokerEvent::Connect(user2.clone(), client2_sender));

    assert_eq!(broker.connections.len(), 2);

    broker.handle_message(BrokerEvent::Sending {
        id: user.id,
        text: "Hello!".into(),
    });

    assert_eq!(String::from("Hello!"), client2_receiver.recv().unwrap());
}

#[test]
fn test_filter() {
    let mut broker = Broker::new();
    let filter = LengthFilter::new(10);
    broker.add_filter(Box::new(filter));

    let (client1_sender, _) = flume::unbounded();
    let (client2_sender, client2_receiver) = flume::unbounded();

    let user = User {
        id: UserId("skade".into()),
        display_name: "Skade".into(),
    };
    let user2 = User {
        id: UserId("other".into()),
        display_name: "Other".into(),
    };

    broker.handle_message(BrokerEvent::Connect(user.clone(), client1_sender));
    broker.handle_message(BrokerEvent::Connect(user2.clone(), client2_sender));

    assert_eq!(broker.connections.len(), 2);

    broker.handle_message(BrokerEvent::Sending {
        id: user.id,
        text: "Hello! HelllOOOO!".into(),
    });

    assert!(client2_receiver.try_recv().is_err());
}
