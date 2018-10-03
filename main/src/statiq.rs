// Copyright 2018 Cássio Kirch.

use super::bad_request;
use actix_web::{HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::path::PathBuf;

type Cache = super::static_cache::HashCache;

lazy_static! {
    static ref CACHE: Cache = Cache::default();
    static ref STATIC_PATH: String = super::get_envvar("SHAAT_STATIC").unwrap();
}

#[inline]
fn get_html(subname: &str) -> Result<Vec<u8>, String> {
    let filename = (*STATIC_PATH).clone() + "/" + subname;
    CACHE.get(&PathBuf::from(filename))
}

pub fn handle_html(subname: &str) -> HttpResponse {
    let content = get_html(subname);
    if content.is_ok() {
        let mime = "text/html";
        HttpResponse::Ok().content_type(mime).body(content.unwrap())
    } else {
        bad_request()
    }
}

pub fn handle_insert_on_html(
    filename: &str,
    insertions: &HashMap<String, String>,
) -> HttpResponse {
    let content0 = get_html(&filename);

    if content0.is_ok() {
        let mime = "text/html";
        let content = String::from_utf8(content0.unwrap()).unwrap();
        let mut out_content = Vec::default();
        for line in content.lines() {
            let line0 = line.trim();
            if line0.starts_with("<!--") && line0.ends_with("-->") {
                let key = line0
                    .replace("<!--", "")
                    .replace("-->", "")
                    .trim()
                    .to_string();
                let value = insertions
                    .get(&key)
                    .map(|value| value.as_str())
                    .unwrap_or("");
                out_content.extend(value.as_bytes());
            } else {
                out_content.extend(line.as_bytes());
            }
            out_content.extend(&[b'\n']);
        }
        HttpResponse::Ok().content_type(mime).body(out_content)
    } else {
        bad_request()
    }
}

pub fn handle_static(req: HttpRequest) -> HttpResponse {
    match req.match_info().get("filename") {
        Some(filename) => {
            let ext = filename.split('.').last();
            if let Some(ext) = ext {
                let mime = match ext {
                    "js" => "application/javascript",
                    "css" => "text/css",
                    _other => {
                        eprintln!("{}", ext);
                        return bad_request();
                    }
                };
                let filename = (*STATIC_PATH).clone() + "/" + filename;
                let content = CACHE.get(&PathBuf::from(filename));
                if content.is_ok() {
                    HttpResponse::Ok().content_type(mime).body(content.unwrap())
                } else {
                    eprintln!("cacaca");
                    bad_request()
                }
            } else {
                bad_request()
            }
        }
        None => {
            bad_request()
        }
    }
}
