#[derive(Clone, PartialEq)]
pub struct User {
    pub id: UserId,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub String);
