//use diesel::prelude::*;
//use diesel::types::*;
//use gotham_derive;
//use serde;


#[derive(Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}
