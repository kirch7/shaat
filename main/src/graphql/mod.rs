use std::sync::Arc;
use actix::{Actor, Addr, Handler, Message, Syn, SyncContext};
use actix_web::{AsyncResponder, Error, FutureResponse, Json, HttpRequest, HttpResponse};
use futures::Future;
use juniper::http::{GraphQLRequest, graphiql::graphiql_source};
use serde_json;
use ws::WsChatSessionState;

mod schema;
pub use self::schema::create_schema;

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct AppState {
    pub executor: Addr<Syn, GraphQLExecutor>,
}

pub struct GraphQLExecutor {
    schema: Arc<schema::Schema>,
}

impl GraphQLExecutor {
    pub fn new(schema: Arc<schema::Schema>) -> GraphQLExecutor {
        GraphQLExecutor { schema: schema }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

pub fn graphiql(req: HttpRequest<(WsChatSessionState, ::db::State, AppState)>) -> Result<HttpResponse, Error> {
    use actix_web::HttpMessage;

    let host = req
        .headers()
        .get("host")
        .map(|e| e.to_str().unwrap_or(""))
        .unwrap_or("");
    let html = graphiql_source(&format!("//{}/graphql", host));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

pub fn graphql(
    (req, data): (HttpRequest<(WsChatSessionState, ::db::State, ::graphql::AppState)>, Json<GraphQLData>)
) -> FutureResponse<HttpResponse> {
    req.state().2.executor
        .send(data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let res = msg.0.execute(&self.schema, &());
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}
