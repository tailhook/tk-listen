use std::time::Duration;

use futures::{Stream, IntoFuture};
use tokio_core::reactor::Handle;

use sleep_on_error;
use listen;

pub trait ListenExt: Stream {
    fn sleep_on_error(self, delay: Duration, handle: &Handle)
        -> sleep_on_error::SleepOnError<Self>
        where Self: Sized,
    {
        sleep_on_error::new(self, delay, handle)
    }
    fn listen(self, max_connections: usize) -> listen::Listen<Self>
        where Self: Sized,
              Self::Item: IntoFuture<Item=(), Error=()>,
    {
        listen::new(self, max_connections)
    }
}

impl<T: Stream> ListenExt for T {}
