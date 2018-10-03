// Copyright 2018 Cássio Kirch.

use super::{Id, MessageGuard};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

type InnerTree<T> = Arc<RwLock<BTreeMap<Id, T>>>;
type OutTree<T> = Arc<RwLock<BTreeMap<Id, InnerTree<T>>>>;

pub struct Messages<T> {
    tree: OutTree<T>,
    last_inner_tree_id: Arc<RwLock<Id>>,
    divisor: Arc<Id>,
}

impl<T> Messages<T> {
    fn new(divisor: u32) -> Self {
        Messages {
            tree: OutTree::default(),
            last_inner_tree_id: Arc::new(RwLock::new(0)),
            divisor: Arc::new(divisor as Id),
        }
    }
}

impl<T> Default for Messages<T> {
    fn default() -> Self {
        Messages::new(1024)
    }
}

impl<T: Clone> MessageGuard for Messages<T> {
    type Message = T;

    fn insert_message<M>(&self, message: M) -> Result<Id, String>
    where
        Self::Message: From<M>,
    {
        let last = self.last_inner_tree_id.read().unwrap().clone();
        if last % *self.divisor == 0 {
            let mut out_tree = self.tree.write().unwrap();
            out_tree.insert(last, Arc::default());
        }

        let last_group = (last / *self.divisor) * *self.divisor;

        let outer_tree = self.tree.read().unwrap();
        let mut inner_tree = outer_tree.get(&last_group).unwrap().write().unwrap();
        inner_tree.insert(last, message.into());
        let mut last = self.last_inner_tree_id.write().unwrap();
        *last += 1;

        Ok(*last - 1)
    }

    fn get_message_by_id(&self, id: Id) -> Result<Self::Message, String> {
        let group_id = (id / *self.divisor) * *self.divisor;
        let outer_tree = &*self.tree.read().map_err(|e| e.to_string())?;
        let inner_tree = outer_tree.get(&group_id);
        if inner_tree.is_none() {
            Err("Inner tree not found".into())
        } else {
            let inner_tree = &*inner_tree.unwrap().read().map_err(|e| e.to_string())?;
            let message = inner_tree.get(&id);
            if message.is_some() {
                Ok(message.unwrap().clone())
            } else {
                Err("Message not found".into())
            }
        }
    }

    fn get_latest(&self, n: Id) -> Vec<Self::Message> {
        let latest_id = self.last_inner_tree_id.read().unwrap().clone();
        let group_id = (latest_id / *self.divisor) * *self.divisor;
        let outer_tree = self.tree.read().unwrap();
        let mut v = Vec::with_capacity(2 * *self.divisor as usize);

        // scan! expects two IDs belonging to the same inner tree.
        macro_rules! scan {
            ($begin:expr, $end:expr) => {
                let begin_group_id = ($begin / *self.divisor) * *self.divisor;
                match outer_tree.get(&begin_group_id) {
                    Some(inner_tree) => {
                        let inner_tree = inner_tree.read().unwrap();
                        for id in $begin..$end {
                            v.push(inner_tree.get(&id).unwrap().clone());
                        }
                    }
                    None => {}
                }
            };
        }

        if n == 0 {
            if group_id > 0 {
                scan!(group_id - *self.divisor, group_id);
            }
            scan!(group_id, latest_id);
        } else if latest_id >= n {
            let first_message_id = latest_id - n;
            let first_group_id = (first_message_id / *self.divisor) * *self.divisor;
            scan!(first_message_id, first_group_id + *self.divisor);

            for group_count in (first_group_id + 1) / *self.divisor..group_id / *self.divisor {
                scan!(
                    group_count * *self.divisor,
                    group_count * *self.divisor + *self.divisor
                );
            }
        }
        v
    }
}
