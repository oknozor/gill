type UserId = i32;

pub struct User {
    id: i32,
    name: String,
    public_key: String,
}

pub struct Repository {
    owner: UserId,
    name: String
}