use super::schema::users;
use super::schema::messages;

#[derive(Clone, Serialize, Queryable)]
pub struct User {
    pub username: String,
    pub color:    String,
    pub password: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub color:    &'a str,
    pub password: &'a str,
}

#[derive(Clone, Serialize, Queryable)]
pub struct Message {
    pub id:       ::messages::Id,
    pub username: String,
    pub message:  String,
}

#[derive(Insertable)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub id:       ::messages::Id,
    pub username: &'a str,
    pub message:  &'a str,
}

