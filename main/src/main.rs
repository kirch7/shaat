// Copyright 2018 Cássio Kirch.

#[macro_use]
extern crate actix;
extern crate actix_web;
extern crate cookie;
extern crate crypto;
#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate lazy_static;
extern crate listenfd;
extern crate messages;
extern crate r2d2;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate static_cache;
extern crate time;

use actix::{Addr, Arbiter, Syn, SyncArbiter};

use actix_web::middleware::Logger;
use actix_web::{http::Method, App, HttpResponse};

use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;

use std::sync::Arc;

mod auth;
mod db;
mod graphql;
mod messajes;
mod statiq;
mod users;
mod ws;

use auth::{CookieIdentityPolicy, IdentityService};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ::std::env::set_var("RUST_LOG", "actix_web=info");

    let addr = get_envvar("SHAAT_ADDR").ok_or("SHAAT_ADDR must be set")?;
    let db_addr = get_envvar("SHAAT_DB").ok_or("SHAAT_DB must be set")?;
    let _ = get_envvar("SHAAT_STATIC").ok_or("SHAAT_STATIC must be set")?;

    env_logger::init();

    println!("loading users");
    users::load(&db_addr).unwrap();
    println!("loading messages");
    messajes::load(&db_addr).unwrap();
    println!("Okay");

    println!("http://{}", addr);

    let sys = actix::System::new("shaat");
    let a_server: Addr<Syn, _> = Arbiter::start(|_| ws::ChatServer::default());

    // Sqlite
    let db_manager = ConnectionManager::<SqliteConnection>::new(db_addr);
    let db_pool = r2d2::Pool::builder().build(db_manager)?;
    let db_server = SyncArbiter::start(1, move || db::DbExecutor(db_pool.clone()));

    // Graphql
    let schema = Arc::new(graphql::create_schema());
    let graphql_addr = SyncArbiter::start(1, move || graphql::GraphQLExecutor::new(schema.clone()));

    let http_server = actix_web::server::new(move || {
        let ws_state = ws::WsChatSessionState {
            addr: a_server.clone(),
            username: String::default(),
        };
        let db_state = db::State {
            db: db_server.clone(),
        };
        let graphql_state = graphql::AppState {
            executor: graphql_addr.clone(),
        };

        vec![
            App::new()
                .middleware(Logger::default())
                .prefix("/static")
                .resource("/{filename}", |r| r.f(statiq::handle_static))
                .boxed(),
            App::with_state((ws_state, db_state, graphql_state))
                .middleware(Logger::default())
                .middleware(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("shaat-auth")
                        .secure(false),
                ))
                .resource("/ws/", |r| r.get().f(ws::handle_ws))
                .resource("/graphql", |r| {
                    r.method(Method::POST).with(graphql::graphql)
                })
                .resource("/graphiql", |r| r.method(Method::GET).f(graphql::graphiql))
                .resource("/login", |r| {
                    r.method(Method::POST).with(auth::handle_login_post);
                    r.method(Method::GET).f(auth::handle_login_get);
                })
                .resource("/logout", |r| r.f(auth::handle_logout))
                .resource("/", |r| r.f(auth::handle_index))
                .boxed(),
        ]
    });

    let mut manager = listenfd::ListenFd::from_env();
    let http_server = if let Some(li) = manager.take_tcp_listener(0).map_err(|e| e.to_string())? {
        http_server.listen(li)
    } else {
        http_server.bind(addr).map_err(|e| e.to_string())?
    };

    http_server.start();
    let _ = sys.run();

    Ok(())
}

#[inline]
fn get_envvar(key: &str) -> Option<String> {
    std::env::vars()
        .find(|(key_, _value)| key_ == key)
        .map(|(_key, value)| value)
}

#[inline]
fn bad_request() -> HttpResponse {
    HttpResponse::BadRequest().finish()
}
