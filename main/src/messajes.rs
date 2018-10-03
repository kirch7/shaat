// Copyright 2018 Cássio Kirch.

use actix::{Addr, Syn};
use db::DbExecutor;
use diesel::{Connection, ExpressionMethods, RunQueryDsl, SqliteConnection, QueryDsl};
use messages::{MessageGuard, Messages};
use futures::Future;

lazy_static! {
    pub static ref MESSAGES: Messages<Message> = Messages::default();
}

#[derive(Clone, Default, Message)]
pub struct Message {
    pub message: Vec<u8>,
    pub username: String,
}

pub fn register_message(username: &String, message: Vec<u8>, db: &Addr<Syn, DbExecutor>) -> Result<(), ()> {
    match (*MESSAGES)
        .insert_message(Message {
            message:  message.clone(),
            username: username.clone(),
        }) {
            Ok(id) => {
                let message = message.clone();
                let insertion = db.send(::db::InsertMessage(::db::models::Message {
                    id,
                    message: String::from_utf8(message).unwrap(),
                    username: username.clone(),
                })).wait();
                
                
                Ok(())
            },
            Err(_) => Err(()),
        }

}

pub fn load(db: &String) -> Result<(), ()> {
    use ::db::schema::messages::dsl::{messages, id};

    let db = SqliteConnection::establish(db).unwrap();
    
    messages
        .order(id.asc())
        .load::<::db::models::Message>(&db)
        .map_err(|_| ())?
        .iter()
        .map(|ref m| Message {
            username: m.username.clone(),
            message:  m.message.clone().into_bytes(),
        })
        .for_each(|m| {
            let _ = MESSAGES.insert_message(m);
        });
    
    Ok(())
}
