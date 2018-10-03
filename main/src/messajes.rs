// Copyright 2018 Cássio Kirch.

use actix::{Addr, Syn};
use db::DbExecutor;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use futures::Future;
use messages::{MessageGuard, Messages};

lazy_static! {
    pub static ref MESSAGES: Messages<Message> = Messages::default();
}

#[derive(Clone, Default, Message)]
pub struct Message {
    pub message: Vec<u8>,
    pub username: String,
}

pub fn register_message(
    username: &str,
    message: &[u8],
    db: &Addr<Syn, DbExecutor>,
) -> Result<(), ()> {
    match (*MESSAGES).insert_message(Message {
        message: message.to_owned(),
        username: username.to_owned(),
    }) {
        Ok(id) => {
            let message = message.to_owned();
            let _insertion = db
                .send(::db::InsertMessage(::db::models::Message {
                    id,
                    message: String::from_utf8(message).unwrap(),
                    username: username.to_owned(),
                }))
                .wait();

            Ok(())
        }
        Err(_) => Err(()),
    }
}

pub fn load(db: &str) -> Result<(), ()> {
    use db::schema::messages::dsl::{id, messages};

    let db = SqliteConnection::establish(db).unwrap();

    messages
        .order(id.asc())
        .load::<::db::models::Message>(&db)
        .map_err(|_| ())?
        .iter()
        .map(|ref m| Message {
            username: m.username.clone(),
            message: m.message.clone().into_bytes(),
        })
        .for_each(|m| {
            let _ = MESSAGES.insert_message(m);
        });

    Ok(())
}
