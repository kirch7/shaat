use actix::{Actor, Addr, Handler, Message, Syn, SyncContext};

use actix_web;

use diesel::{self, SqliteConnection, RunQueryDsl};
use diesel::r2d2::{ConnectionManager, Pool};

pub mod schema;
pub mod models;

pub struct DbExecutor(pub Pool<ConnectionManager<SqliteConnection>>);

/// An state for actix
pub struct State {
    pub db: Addr<Syn, DbExecutor>,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

/// Structure to help new users insertion on database.
pub struct CreateUser(pub models::User);

/// Structure to help query users by their names on database.  pub
pub struct QueryUser(pub String);

/// Structure to help query users by their names on database.  pub
pub struct QueryAllUsers;

/// Structure to help new users insertion on database.
pub struct InsertMessage(pub models::Message);

/// Structure to help query users by their names on database.
pub struct QueryMessage(pub ::messages::Id);

impl Message for CreateUser {
    type Result = Result<models::User, actix_web::Error>;
}

impl Message for QueryUser {
    type Result = Result<models::User, actix_web::Error>;
}

impl Message for QueryAllUsers {
    type Result = Result<Vec<models::User>, actix_web::Error>;
}

impl Message for InsertMessage {
    type Result = Result<models::Message, actix_web::Error>;
}

impl Message for QueryMessage {
    type Result = Result<models::Message, actix_web::Error>;
}

impl Handler<QueryUser> for DbExecutor {
    type Result = Result<models::User, actix_web::Error>;

    fn handle(&mut self, message: QueryUser, _ctx: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;
        use diesel::QueryDsl;
        
        let connection: &SqliteConnection = &self.0.get().unwrap();

        let all_users = users
            .select((username, color, password))
            .load::<(String, String, String)>(connection)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        all_users.iter()
            .find(|ref t| t.0 == message.0)
            .map(|user| Ok(models::User {
                username: user.0.clone(),
                color:    user.1.clone(),
                password: user.2.clone(),
            }))
            .unwrap_or(Err(actix_web::error::ErrorInternalServerError("Empty query result")))
    }
}

impl Handler<QueryAllUsers> for DbExecutor {
    type Result = Result<Vec<models::User>, actix_web::Error>;

    fn handle(&mut self, _message: QueryAllUsers, _ctx: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;
        use diesel::QueryDsl;
        
        let connection: &SqliteConnection = &self.0.get().unwrap();

        Ok(users
           .select((username, color, password))
           .load::<(String, String, String)>(connection)
           .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
           .iter()
           .map(|ref user| models::User {
               username: user.0.clone(),
               color:    user.1.clone(),
               password: user.2.clone(),
           })
           .collect())
    }
}

impl Handler<CreateUser> for DbExecutor {
    type Result = Result<models::User, actix_web::Error>;

    fn handle(&mut self, message: CreateUser, _ctx: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::users;

        let user = message.0.clone();
        let new_user = models::NewUser {
            username: &user.username.clone(),
            color:    &user.color.clone(),
            password: &user.password.clone(),
        };

        let connection: &SqliteConnection = &self.0.get().unwrap();

        diesel::insert_into(users)
            .values(&new_user)
            .execute(connection)
            .map_err(|_| actix_web::error::ErrorInternalServerError("Error inserting person"))?;

        Ok(user)
    }
}

impl Handler<QueryMessage> for DbExecutor {
    type Result = Result<models::Message, actix_web::Error>;

    fn handle(&mut self, m: QueryMessage, _ctx: &mut Self::Context) -> Self::Result {
        use self::schema::messages::dsl::*;
        use diesel::QueryDsl;
        
        let connection: &SqliteConnection = &self.0.get().unwrap();

        let all_messages = messages
            .select((id, username, message))
            .load::<(::messages::Id, String, String)>(connection)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        all_messages.iter()
            .find(|ref tup| tup.0 == m.0)
            .map(|tuple| Ok(models::Message {
                id:       tuple.0.clone(),
                username: tuple.1.clone(),
                message:  tuple.2.clone(),
            }))
            .unwrap_or(Err(actix_web::error::ErrorInternalServerError("Empty query result")))
    }
}

impl Handler<InsertMessage> for DbExecutor {
    type Result = Result<models::Message, actix_web::Error>;

    fn handle(&mut self, message: InsertMessage, _ctx: &mut Self::Context) -> Self::Result {
        use self::schema::messages::dsl::messages;

        let message = message.0.clone();
        let new_message = models::NewMessage {
            id:       message.id.clone(),
            username: &message.username.clone(),
            message:  &message.message.clone(),
        };

        let connection: &SqliteConnection = &self.0.get().unwrap();

        diesel::insert_into(messages)
            .values(&new_message)
            .execute(connection)
            .map_err(|_| actix_web::error::ErrorInternalServerError("Error inserting message"))?;

        Ok(message)
    }
}
