use std::fmt;

use crate::sinks::{prelude::*, util::partitioner::KeyPartitioner};
use vector_lib::{event::Event, partition::Partitioner};

pub struct AzureBlobSink<Svc, RB, P = KeyPartitioner> {
    service: Svc,
    request_builder: RB,
    partitioner: P,
    batcher_settings: BatcherSettings,
}

impl<Svc, RB, P> AzureBlobSink<Svc, RB, P> {
    pub const fn new(
        service: Svc,
        request_builder: RB,
        partitioner: P,
        batcher_settings: BatcherSettings,
    ) -> Self {
        Self {
            service,
            request_builder,
            partitioner,
            batcher_settings,
        }
    }
}

impl<Svc, RB, P> AzureBlobSink<Svc, RB, P>
where
    Svc: Service<RB::Request> + Send + 'static,
    Svc::Future: Send + 'static,
    Svc::Response: DriverResponse + Send + 'static,
    Svc::Error: fmt::Debug + Into<crate::Error> + Send,
    RB: RequestBuilder<(String, Vec<Event>)> + Send + Sync + 'static,
    RB::Error: fmt::Display + Send,
    RB::Request: Finalizable + MetaDescriptive + Send,
    P: Partitioner<Item = Event, Key = Option<String>> + Unpin + Send,
    P::Key: Eq + std::hash::Hash + Clone,
    P::Item: ByteSizeOf,
{
    async fn run_inner(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        let partitioner = self.partitioner;
        let settings = self.batcher_settings;

        let request_builder = self.request_builder;

        input
            .batched_partitioned(partitioner, || settings.as_byte_size_config())
            .filter_map(|(key, batch)| async move {
                // We don't need to emit an error here if the event is dropped since this will occur if the template
                // couldn't be rendered during the partitioning. A `TemplateRenderingError` is already emitted when
                // that occurs.
                key.map(move |k| (k, batch))
            })
            .request_builder(default_request_builder_concurrency_limit(), request_builder)
            .filter_map(|request| async move {
                match request {
                    Err(error) => {
                        emit!(SinkRequestBuildError { error });
                        None
                    }
                    Ok(req) => Some(req),
                }
            })
            .into_driver(self.service)
            .protocol("https")
            .run()
            .await
    }
}

#[async_trait]
impl<Svc, RB, P> StreamSink<Event> for AzureBlobSink<Svc, RB, P>
where
    Svc: Service<RB::Request> + Send + 'static,
    Svc::Future: Send + 'static,
    Svc::Response: DriverResponse + Send + 'static,
    Svc::Error: fmt::Debug + Into<crate::Error> + Send,
    RB: RequestBuilder<(String, Vec<Event>)> + Send + Sync + 'static,
    RB::Error: fmt::Display + Send,
    RB::Request: Finalizable + MetaDescriptive + Send,
    P: Partitioner<Item = Event, Key = Option<String>> + Unpin + Send,
    P::Key: Eq + std::hash::Hash + Clone,
    P::Item: ByteSizeOf,
{
    async fn run(mut self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        self.run_inner(input).await
    }
}
