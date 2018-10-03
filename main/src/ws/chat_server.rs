use actix::{Actor, Context, Handler, Recipient, Syn};
use std::collections::HashMap;

#[derive(Default, Message)]
pub struct ChatServer {
    sessions: HashMap<usize, (String, Recipient<Syn, ::messajes::Message>)>,
}

impl ChatServer {
    pub fn send_message(&self, message: &::messajes::Message) {
        self.sessions
            .iter()
            .map(|(_id, tuple)| &tuple.1)
            .for_each(|recipient| {
                let _ = recipient.do_send(message.clone()); //// todo: erro handling
            });
    }

    pub fn get_new_id(&mut self) -> usize {
        (0..)
            .filter(|id| !self.sessions.contains_key(&id))
            .nth(0)
            .unwrap()
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<super::Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, message: super::Disconnect, _context: &mut Context<Self>) -> Self::Result {
        let _ = self.sessions.remove(&message.session_id);
    }
}

impl Handler<super::Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, message: super::Connect, _context: &mut Context<Self>) -> Self::Result {
        let id = self.get_new_id();
        let sessions = &mut self.sessions;
        let _ = sessions.insert(id, (message.username, message.addr));
        for (key, tuple) in sessions {
            println!("{}\t{:?}", key, tuple.0);
        }

        id
    }
}

impl Handler<::messajes::Message> for ChatServer {
    type Result = ();

    fn handle(
        &mut self,
        message: ::messajes::Message,
        _context: &mut Context<Self>,
    ) -> Self::Result {
        self.send_message(&message);
    }
}
