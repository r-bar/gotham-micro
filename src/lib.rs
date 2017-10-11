//#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate chrono;
//#[macro_use]
extern crate log;
extern crate fern;
extern crate mime;
extern crate gotham_middleware_diesel;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
//extern crate serde;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod models;
pub mod handlers;
pub mod db;
pub mod schema;

//use diesel::infer_schema;
//use diesel::pg::types::sql_types::Jsonb;

use std::env;

use diesel::pg::PgConnection;
//use diesel::pg::types::sql_types::*;
use diesel::types::*;


use futures::{future, Future, Stream};

use hyper::{Body, Response, Method, StatusCode};

use log::LogLevelFilter;

use gotham::router::request::path::NoopPathExtractor;
use gotham::router::request::query_string::NoopQueryStringExtractor;
use gotham::router::response::finalizer::ResponseFinalizerBuilder;
use gotham::router::response::extender::NoopResponseExtender;
use gotham::router::Router;
use gotham::router::route::{Route, RouteImpl, Extractors, Delegation};
use gotham::router::route::dispatch::{new_pipeline_set, finalize_pipeline_set, PipelineSet,
                                      DispatcherImpl, PipelineHandleChain};
use gotham::router::route::matcher::MethodOnlyRouteMatcher;
use gotham::router::tree::TreeBuilder;
use gotham::router::tree::node::{NodeBuilder, SegmentType};
use gotham::handler::{Handler, NewHandler, HandlerFuture, IntoHandlerError, IntoResponse};
use gotham::middleware::pipeline::new_pipeline;
use gotham::state::{State, FromState};
use gotham::http::response::create_response;

use gotham_middleware_diesel::DieselMiddleware;
use gotham_middleware_diesel::state_data::Diesel;

fn static_route<NH, P, C>(
    methods: Vec<Method>,
    new_handler: NH,
    active_pipelines: C,
    pipeline_set: PipelineSet<P>,
) -> Box<Route + Send + Sync>
where
    NH: NewHandler + 'static,
    C: PipelineHandleChain<P> + Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    let matcher = MethodOnlyRouteMatcher::new(methods);
    let dispatcher = DispatcherImpl::new(new_handler, active_pipelines, pipeline_set);
    let extractors: Extractors<NoopPathExtractor, NoopQueryStringExtractor> = Extractors::new();
    let route = RouteImpl::new(
        matcher,
        Box::new(dispatcher),
        extractors,
        Delegation::Internal,
    );
    Box::new(route)
}

pub fn router() -> Router {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
    let diesel_middleware: DieselMiddleware<PgConnection> =
        DieselMiddleware::new(&database_url, None);
    let editable_pipeline_set = new_pipeline_set();
    let (editable_pipeline_set, global) =
        editable_pipeline_set.add(new_pipeline().add(diesel_middleware).build());
    let pipeline_set = finalize_pipeline_set(editable_pipeline_set);

    let mut tree_builder = TreeBuilder::new();
    tree_builder.add_route(static_route(
        vec![Method::Get],
        || Ok(Echo::get),
        (global, ()),
        pipeline_set.clone(),
    ));

    let mut echo = NodeBuilder::new("echo", SegmentType::Static);
    echo.add_route(static_route(
        vec![Method::Get, Method::Head],
        || Ok(Echo::get),
        (global, ()),
        pipeline_set.clone(),
    ));
    echo.add_route(static_route(
        vec![Method::Post],
        || Ok(Echo::post),
        (global, ()),
        pipeline_set.clone(),
    ));

    let mut posts = NodeBuilder::new("posts", SegmentType::Static);
    posts.add_route(static_route(
        vec![Method::Get],
        || Ok(handlers::posts::index),
        (global, ()),
        pipeline_set.clone(),
    ));

    let mut posts2 = NodeBuilder::new("posts2", SegmentType::Static);
    posts.add_route(static_route(
        vec![Method::Get],
        || Ok(handlers::posts::index2),
        (global, ()),
        pipeline_set.clone(),
    ));


    tree_builder.add_child(echo);
    tree_builder.add_child(posts);
    tree_builder.add_child(posts2);
    let tree = tree_builder.finalize();

    let mut response_finalizer_builder = ResponseFinalizerBuilder::new();
    let extender_200 = NoopResponseExtender::new();
    response_finalizer_builder.add(StatusCode::Ok, Box::new(extender_200));
    let extender_500 = NoopResponseExtender::new();
    response_finalizer_builder.add(StatusCode::InternalServerError, Box::new(extender_500));
    let response_finalizer = response_finalizer_builder.finalize();

    Router::new(tree, response_finalizer)
}

struct Echo;

impl Echo {
    pub fn get(state: State) -> (State, Response) {
        let res = create_response(
            &state,
            StatusCode::MethodNotAllowed,
            Some((
                String::from("Use POST /echo instead").into_bytes(),
                mime::TEXT_PLAIN,
            )),
        );
        (state, res)
    }

    pub fn post(mut state: State) -> Box<HandlerFuture> {
        let f = Body::take_from(&mut state).concat2().then(
            move |full_body| {
                match full_body {
                    Ok(valid_body) => {
                        let res = create_response(
                            &state,
                            StatusCode::Ok,
                            Some((valid_body.to_vec(), mime::TEXT_PLAIN)),
                        );
                        future::ok((state, res))
                    }
                    Err(e) => future::err((state, e.into_handler_error())),
                }
            },
        );
        Box::new(f)
    }
}

pub fn set_logging() {
    fern::Dispatch::new()
        .level(LogLevelFilter::Error)
        .level_for("gotham", log::LogLevelFilter::Info)
        .level_for("gotham_middleware_diesel", log::LogLevelFilter::Info)
        .level_for("gotham::state", log::LogLevelFilter::Info)
        .level_for("gotham::start", log::LogLevelFilter::Info)
        .chain(std::io::stdout())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}]{}",
                chrono::UTC::now().format("[%Y-%m-%d %H:%M:%S%.9f]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .apply()
        .unwrap();
}
