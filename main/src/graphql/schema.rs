use messages::{Id, MessageGuard};
use juniper::{FieldError, FieldResult};
use juniper::RootNode;

#[derive(GraphQLObject)]
#[graphql(description = "A user for shaat")]
struct User {
    username: String,
    color: String,
}

#[derive(GraphQLObject)]
#[graphql(description = "A shaat message")]
struct Message {
    username: String,
    message: String,
}

pub struct QueryRoot;

graphql_object!(QueryRoot: () |&self| {
    field user(&executor, username: String) -> FieldResult<User> {
        let users = ::users::USERS
            .read()
            .unwrap();
        match users.get(&username) {
            Some(user) => Ok(User {
                username: user.username.clone(),
                color: user.color.clone(),
            }),
            None => Err(FieldError::new(
                "User not found",
                graphql_value!({ "some error": "User not found" })
            )),
        }
    }
    
    field message(&executor, last_n: String) -> FieldResult<Vec<Message>> {
        let last_n = last_n.parse::<Id>();
        if last_n.is_err() || last_n.clone().unwrap() < 0 {
            Err(FieldError::new(
                "Invalid id",
                graphql_value!({ "some error": "Invalid id" })
            ))
        } else {
            let messages: Vec<_> = ::messajes::MESSAGES
                .get_latest(last_n.unwrap())
                .iter()
                .map(|m| Message {
                    username: m.username.clone(),
                    message: String::from_utf8(m.message.clone()).unwrap(),
                })
                .collect();
            Ok(messages)
        } 
    }
});

pub type Schema = RootNode<'static, QueryRoot, ()>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, ())
}
