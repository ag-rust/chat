pub enum FilterMessage {
    Yes,
    No,
}

pub trait Filter {
    fn filter(&self, msg: &str) -> FilterMessage;
}

pub struct LengthFilter {
    length: usize,
}

impl LengthFilter {
    pub fn new(length: usize) -> LengthFilter {
        LengthFilter { length }
    }
}

impl Filter for LengthFilter {
    fn filter(&self, msg: &str) -> FilterMessage {
        if msg.len() > self.length {
            FilterMessage::Yes
        } else {
            FilterMessage::No
        }
    }
}
