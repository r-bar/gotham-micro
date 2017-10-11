use gotham::router::route::dispatch::Dispatcher;
use std::error::Error;

/// Default implementation of the `Dispatcher` trait.
pub struct DispatcherImpl<H, C, P>
where
    H: NewHandler,
    C: PipelineHandleChain<P>,
{
    new_handler: H,
    pipeline_chain: C,
    pipelines: PipelineSet<P>,
}


impl<H, C, P> ResultDispatcherImpl<H, C, P>
where
    H: NewHandler,
    H::Instance: 'static,
    C: PipelineHandleChain<P>,
{
    /// Creates a new `ResultDispatcherImpl`.
    ///
    /// * `new_handler` - The `Handler` that will be called once the `pipeline_chain` is complete.
    /// * `pipeline_chain` - A chain of `Pipeline` instance handles that indicate which `Pipelines` will be invoked.
    /// * `pipelines` - All `Pipeline` instances, accessible by the handles provided in `pipeline_chain`.
    ///
    pub fn new(new_handler: H, pipeline_chain: C, pipelines: PipelineSet<P>) -> Self {
        ResultDispatcherImpl {
            new_handler,
            pipeline_chain,
            pipelines,
        }
    }

}

impl<H, C, P> Dispatcher for ResultDispatcherImpl<H, C, P>
where
    H: NewHandler,
    H::Instance: 'static,
    C: PipelineHandleChain<P>,
{
    fn dispatch(&self, state: State) -> Box<HandlerFuture> {
        match self.new_handler.new_handler() {
            Ok(h) => {
                trace!("[{}] cloning handler", request_id(&state));
                self.pipeline_chain.call(
                    &self.pipelines,
                    state,
                    move |state| h.handle(state),
                )
            }
            Err(e) => {
                trace!("[{}] error cloning handler", request_id(&state));
                Box::new(future::err((state, e.into_handler_error())))
            }
        }
    }
}

pub fn default_error_handler<E: Error>(error: ) {

}
