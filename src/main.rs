//#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate chrono;
#[macro_use]
extern crate log;
extern crate fern;
extern crate mime;

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
use gotham::handler::{NewHandler, HandlerFuture, IntoHandlerError};
use gotham::middleware::pipeline::new_pipeline;
use gotham::state::{State, FromState};
use gotham::http::response::create_response;

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

fn router() -> Router {
    let editable_pipeline_set = new_pipeline_set();
    let (editable_pipeline_set, global) = editable_pipeline_set.add(new_pipeline().build());
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

    tree_builder.add_child(echo);
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

fn set_logging() {
    fern::Dispatch::new()
        .level(LogLevelFilter::Error)
        .level_for("gotham", log::LogLevelFilter::Error)
        .level_for("gotham::state", log::LogLevelFilter::Error)
        .level_for("gotham::start", log::LogLevelFilter::Info)
        .level_for("kitchen_sink", log::LogLevelFilter::Error)
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

fn main() {
    set_logging();

    gotham::start("127.0.0.1:7878", router());
}
