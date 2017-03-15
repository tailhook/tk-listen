use std::time::Duration;

use futures::{Stream, Async};
use tokio_core::reactor::Handle;


pub struct SleepOnError<S> {
    stream: S,
    delay: Duration,
    handle: Handle,
}

pub fn new<S>(stream: S, delay: Duration, handle: &Handle)
    -> SleepOnError<S>
{
    SleepOnError {
        stream: stream,
        delay: delay,
        handle: handle.clone(),
    }
}

impl<I, E, S: Stream<Item=I, Error=E>> Stream for SleepOnError<S> {
    type Item = I;
    type Error = E;
    fn poll(&mut self) -> Result<Async<Option<I>>, E> {
        unimplemented!();
    }
}
