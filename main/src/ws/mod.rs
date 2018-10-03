use actix::{Recipient, Syn};
use actix_web::{self, ws, HttpRequest, HttpResponse};

pub use self::chat_server::ChatServer;
use self::session::WsChatSession;
pub use self::session::WsChatSessionState;
use auth::RequestIdentity;

mod chat_server;
mod session;

#[derive(Message)]
struct Disconnect {
    session_id: usize,
}

#[derive(Message)]
#[rtype(usize)]
struct Connect {
    username: String,
    addr: Recipient<Syn, ::messajes::Message>,
}

pub fn handle_ws(
    req: HttpRequest<(WsChatSessionState, ::db::State, ::graphql::AppState)>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = {
        let req = req.clone();
        match req.identity() {
            Some(username) => username.into(),
            None => {
                return Ok(::bad_request());
            }
        }
    };
    ws::start(req, WsChatSession::new(username))
}
