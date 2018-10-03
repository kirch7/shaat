use super::ws;
use actix::{fut, ActorFuture, AsyncContext, ContextFutureSpawner, StreamHandler, WrapFuture};
use actix::{Actor, ActorContext, Addr, Handler, Running, Syn};

pub struct WsChatSessionState {
    pub addr: Addr<Syn, super::ChatServer>,
    pub username: String,
}

pub struct WsChatSession {
    id: usize,
    username: String,
}

impl WsChatSession {
    pub fn new(username: String) -> Self {
        WsChatSession {
            id: 0,
            username: username,
        }
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self, (WsChatSessionState, ::db::State, ::graphql::AppState)>;

    fn started(&mut self, context: &mut Self::Context) {
        let address: Addr<Syn, _> = context.address();
        context
            .state()
            .0
            .addr
            .send(super::Connect {
                addr: address.recipient(),
                username: self.username.clone(),
            })
            .into_actor(self)
            .then(|response, actor, context| {
                match response {
                    Ok(response) => actor.id = response,
                    _ => context.stop(),
                };
                fut::ok(())
            })
            .wait(context);
    }

    fn stopping(&mut self, context: &mut Self::Context) -> Running {
        context.state().0.addr.do_send(super::Disconnect {
            session_id: self.id,
        });
        Running::Stop
    }
}

impl Handler<::messajes::Message> for WsChatSession {
    type Result = ();

    fn handle(
        &mut self,
        message: ::messajes::Message,
        context: &mut Self::Context,
    ) -> Self::Result {
        let color = {
            match ::users::USERS.read().unwrap().get(&message.username) {
                Some(user) => user.color.clone(),
                None       => "#ffffff".into(),
            }
        };
        let open = format!("<code style=\"color:{}\">{} </code>", color, message.username);
        let mut m = open.into_bytes();
        let close = "<br/>".as_bytes();
        m.extend_from_slice(&message.message);
        m.extend_from_slice(close);
        
        context.text(m);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WsChatSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            ws::Message::Text(text) => {
                let text = text.trim();
                eprintln!("text: {:?}", text);
                let text = text.as_bytes().to_vec();

                // Registering in server memory:
                let db = &ctx
                    .state()
                    .1
                    .db;
                
                ::messajes::register_message(&self.username, text.clone(), db).unwrap_or(()); // todo: error handling

                // Send to all sessions:
                ctx.state().0.addr.do_send(::messajes::Message{
                    username: self.username.clone(),
                    message:  text,
                })
            }
            ws::Message::Binary(_) => {
                //// todo
            }
            ws::Message::Close(_) => {
                eprintln!("closing {}'s session", self.username);
                ctx.stop();
            }
            _ => {
                eprintln!("other");
                //// todo
            }
        };
    }
}
