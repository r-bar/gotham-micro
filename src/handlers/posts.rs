use gotham::state::State;
use gotham::handler::HandlerFuture;
use gotham::http::response::*;
use models::posts::Post;
use gotham_middleware_diesel::state_data::Diesel;
use diesel::prelude::*;
use schema::posts::dsl::*;
use db;
use hyper::{StatusCode, Request, Response};
use mime;
use serde_json;
use future;

pub fn index(mut state: State) -> Box<HandlerFuture> {
    //let db = state.take::<Diesel<PgConnection>>();
    let connection = db::connect(&mut state).expect("Failed to connect to database");
    let post_list = posts.load::<Post>(&*connection).expect(
        "Error loading posts",
    );
    let json = serde_json::to_string(&post_list).unwrap().into_bytes();
    let res = create_response(&state, StatusCode::Ok, Some((json, mime::APPLICATION_JSON)));
    //let f = future::ok((state, res));
    //Box::new(f)
    //unimplemented!();
    Box::new(future::ok((state, res)))
}

pub fn index2(mut state: State) -> (State, Response) {
    //let db = state.take::<Diesel<PgConnection>>();
    let connection = db::connect(&mut state).expect("Failed to connect to database");
    let post_list = posts.load::<Post>(&*connection).expect(
        "Error loading posts",
    );
    let json = serde_json::to_string(&post_list).unwrap().into_bytes();
    let res = create_response(&state, StatusCode::Ok, Some((json, mime::APPLICATION_JSON)));
    (state, res)
}

pub fn create(state: State) -> Box<HandlerFuture> {
    unimplemented!()

}

pub fn update(state: State) -> Box<HandlerFuture> {
    unimplemented!()

}

pub fn show(state: State) -> Box<HandlerFuture> {
    unimplemented!()

}

pub fn delete(state: State) -> Box<HandlerFuture> {
    unimplemented!()

}
