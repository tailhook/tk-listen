use std::time::Duration;

use futures::{Stream, IntoFuture};
use tokio_core::reactor::Handle;

use sleep_on_error;
use listen;


/// An extension trait that provides necessary combinators for turning
/// a stream of `accept()` events into a full-featured connection listener
///
/// Usually both `sleep_on_error` and `listen` commbinators are used in pair
/// with `.map()` in-between. See [examples][1] for full-featured example.
///
/// [1]: https://github.com/tailhook/tk-listen/tree/master/examples
pub trait ListenExt: Stream {
    /// Turns a listening stream that you can get from `TcpListener::incoming`
    /// into a stream that supresses errors and sleeps on resource shortage,
    /// effectively allowing listening stream to resume on error.
    fn sleep_on_error(self, delay: Duration, handle: &Handle)
        -> sleep_on_error::SleepOnError<Self>
        where Self: Sized,
    {
        sleep_on_error::new(self, delay, handle)
    }
    /// Turns a stream of protocol handlers usually produced by mapping
    /// a stream of accepted cnnec
    fn listen(self, max_connections: usize) -> listen::Listen<Self>
        where Self: Sized,
              Self::Item: IntoFuture<Item=(), Error=()>,
    {
        listen::new(self, max_connections)
    }
}

impl<T: Stream> ListenExt for T {}
