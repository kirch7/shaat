use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use diesel::{Connection, RunQueryDsl, SqliteConnection};

lazy_static! {
    pub static ref USERS: Arc<RwLock<HashMap<String, User>>> = Arc::default();
}

pub struct User {
    pub username: String,
    pub color:    String,
    pub password: String,
}

pub fn load(db: &String) -> Result<(), ()> {
    let db = SqliteConnection::establish(db).unwrap();
    
    let mut users = USERS.write().unwrap();
    ::db::schema::users::dsl::users
        .load::<::db::models::User>(&db)
        .map_err(|_| ())?
        .iter()
        .map(|user| User {
            username: user.username.clone(),
            color:    user.color.clone(),
            password: user.password.clone(),
        })
        .for_each(|user| {
            let _ = users.insert(user.username.clone(), user);
        });
    
    Ok(())
}
