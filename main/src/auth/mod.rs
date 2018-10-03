// Copyright 2018 Cássio Kirch.

use actix_web::{AsyncResponder, Error, Form, HttpRequest, HttpResponse};
use crypto::{digest::Digest, sha3::Sha3};
use futures::{Future, Stream};
use std::collections::HashMap;

use super::ws::WsChatSessionState;

mod cookies;
pub use self::cookies::{CookieIdentityPolicy, IdentityService, RequestIdentity};

#[derive(Debug, Deserialize)]
pub struct Login {
    username: Option<String>,
    password0: Option<String>,
    repassword: Option<String>,
    usercolor: Option<String>,
}

fn hash(input: &str) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(input);
    hasher.result_str()
}

pub fn handle_index(
    req: HttpRequest<(WsChatSessionState, ::db::State, ::graphql::AppState)>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let in_users = {
        let id = &req.identity();
        if id.is_none() {
            false
        } else {
            (*::users::USERS.read().unwrap()).contains_key(id.unwrap())
        }
    };
    if in_users {
        let fut = req
            .concat2()
            .from_err()
            .and_then(|_| Ok(super::statiq::handle_html("index.html")));
        Box::new(fut)
    } else {
        let fut = req
            .concat2()
            .from_err()
            .and_then(|_| Ok(HttpResponse::Found().header("location", "/login").finish()));
        Box::new(fut)
    }
}

pub fn handle_login_post(
    (req, form): (
        HttpRequest<(WsChatSessionState, ::db::State, ::graphql::AppState)>,
        Form<Login>,
    ),
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let already_logged = {
        let id = &req.identity();
        id.is_some()
    };

    if already_logged {
        send_to(req, "/")
    } else {
        let mut req3 = req.clone();

        req.concat2()
            .from_err()
            .and_then(move |_| {
                let mut s = String::default();

                let username = form.username.clone();
                match &username {
                    None => {
                        s += "undefined username. ";
                    }
                    Some(username) => {
                        // Validating username.
                        if username.len() < 2 {
                            s += "username too short. ";
                        } else if username.len() > 12 {
                            s += "username too large. ";
                        }
                    }
                }

                let password0 = form.password0.clone();
                let repassword = form.repassword.clone();

                let mut exists = false;
                if s.is_empty() {
                    exists = {
                        if password0.is_none() {
                            s += "No password. ";
                        }
                        ::users::USERS
                            .read()
                            .unwrap()
                            .contains_key(&username.clone().unwrap())
                    };
                    if exists {
                        if repassword.is_some() && repassword.unwrap() != "" {
                            s += "Re-register? ";
                        }
                        let password0 = password0.clone();
                        if password0.is_some() {
                            let password0 = hash(&password0.unwrap());
                            let user = ::users::USERS.read().unwrap();
                            let password = user
                                .get(&username.clone().unwrap())
                                .unwrap()
                                .password
                                .clone();
                            if password != password0 {
                                s += "Wrong password. ";
                            }
                        }
                    } else {
                        if repassword.is_none() {
                            s += "Enter password twice. ";
                        }

                        match &form.usercolor {
                            None => {
                                s += "undefined usercolor. ";
                            }
                            Some(usercolor) => {
                                // Validating color.
                                if usercolor.len() != 7 {
                                    s += "could not understand color. ";
                                } else {
                                    for c in usercolor.chars() {
                                        if c == '#'
                                            || (c as u32 >= '0' as u32 && c as u32 <= '9' as u32)
                                            || (c as u32 >= 'a' as u32 && c as u32 <= 'f' as u32)
                                            || (c as u32 >= 'A' as u32 && c as u32 <= 'F' as u32)
                                        {
                                            continue;
                                        } else {
                                            s += "could not understand color. ";
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if s.is_empty() {
                    if !exists {
                        let color = form.usercolor.clone().unwrap_or_else(|| "#000000".to_owned());
                        let user_inner = ::db::models::User {
                            username: username.clone().unwrap(),
                            color: color.clone(),
                            password: hash(&password0.clone().unwrap()),
                        };
                        match req3.state().1.db.send(::db::CreateUser(user_inner)).wait() {
                            Err(e) => {
                                s += &format!("{:?}", e);
                            }
                            Ok(_) => {
                                req3.remember(username.clone().unwrap());
                                ::users::USERS.write().unwrap().insert(
                                    username.clone().unwrap(),
                                    ::users::User {
                                        username: username.unwrap().clone(),
                                        color,
                                        password: hash(&password0.unwrap().clone()),
                                    },
                                );
                            }
                        };
                    } else {
                        req3.remember(username.clone().unwrap());
                    }
                }

                if s.is_empty() {
                    Ok(HttpResponse::Found().header("location", "/").finish())
                } else {
                    let mut hash = HashMap::default();
                    hash.insert("ERRORMESSAGE".into(), s);
                    Ok(::statiq::handle_insert_on_html("login.html", &hash))
                }
            })
            .responder()
    }
}

pub fn handle_login_get(
    req: HttpRequest<(WsChatSessionState, ::db::State, ::graphql::AppState)>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let already_logged = {
        let id = &req.identity();
        if id.is_none() {
            false
        } else {
            (*::users::USERS.read().unwrap()).contains_key(id.unwrap())
        }
    };
    if already_logged {
        send_to(req, "/")
    } else {
        req.concat2()
            .from_err()
            .and_then(move |_| Ok(::statiq::handle_html("login.html")))
            .responder()
    }
}

pub fn handle_logout(
    mut req: HttpRequest<(WsChatSessionState, ::db::State, ::graphql::AppState)>,
) -> HttpResponse {
    req.forget();
    HttpResponse::Found().header("location", "/").finish()
}

fn send_to(
    req: HttpRequest<(WsChatSessionState, ::db::State, ::graphql::AppState)>,
    to: &'static str,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let fut = req
        .concat2()
        .from_err()
        .and_then(move |_| Ok(HttpResponse::Found().header("location", to).finish()));
    Box::new(fut)
}
