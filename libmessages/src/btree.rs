// Copyright 2018 Cássio Kirch.

use super::{Id, MessageGuard};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

type Tree<T> = Arc<RwLock<BTreeMap<Id, T>>>;

#[derive(Default)]
pub struct Messages<T> {
    tree: Tree<T>,
    last_id: Arc<RwLock<Id>>,
}

impl<T> MessageGuard for Messages<T>
where
    T: Default + Clone,
{
    type Message = T;

    fn insert_message<M>(&self, message: M) -> Result<super::Id, String>
    where
        Self::Message: From<M>,
    {
        let (mut tree, mut last) = (self.tree.write().unwrap(), self.last_id.write().unwrap());
        tree.insert(*last, message.into());
        *last += 1;

        Ok(*last - 1)
    }

    fn get_message_by_id(&self, id: Id) -> Result<Self::Message, String> {
        let tree = &*self.tree.read().map_err(|e| e.to_string())?;
        let message = tree.get(&id);
        if message.is_some() {
            Ok((*message.unwrap()).clone())
        } else {
            Err("Message not found".into())
        }
    }

    fn get_latest(&self, n: Id) -> Vec<Self::Message> {
        let (tree, last) = (self.tree.read().unwrap(), self.last_id.read().unwrap());
        let n = if n <= 0 { 32 } else { n };

        let n = if n > *last { *last } else { n };
        let n = *last - n;

        let mut v = Vec::with_capacity(n as usize);
        for (key, value) in &*tree {
            if *key >= n {
                v.push(value.clone());
            }
        }
        v
    }
}
