use std::time::Duration;

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
