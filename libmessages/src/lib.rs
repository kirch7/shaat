// Copyright 2018 Cássio Kirch.

pub type Id = i64;

mod btree;
mod btree2;

pub trait MessageGuard: Default {
    type Message;

    fn insert_message<M>(&self, message: M) -> Result<Id, String>
    where
        Self::Message: From<M>;

    fn get_message_by_id(&self, id: Id) -> Result<Self::Message, String>;
    fn get_latest(&self, n: Id) -> Vec<Self::Message>;
}

pub type Messages2<T> = btree2::Messages<T>;
pub type Messages<T> = btree::Messages<T>;
