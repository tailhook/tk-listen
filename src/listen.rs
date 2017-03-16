use std::marker::PhantomData;

use futures::{Stream, Future, IntoFuture, Async};
use futures::stream::BufferUnordered;


/// This is basically `.then(|_| Ok(()))`, but we can't name a closure
struct SwallowErrors<F: Future<Item=(), Error=()>,  E>(F, PhantomData<E>);

/// This is basically `map(|f| f.then(|_| Ok(())))`,
/// but we can't name a closure
struct MapSwallowErrors<S: Stream>(S)
    where S::Item: IntoFuture<Item=(), Error=()>;



/// A structure returned by `ListenExt::listen`
///
/// This is a future that returns when incoming stream has been closed and
/// all connections (futures) have been processed. It uses `BufferUnordered`
/// inside to do heavy lifting.
pub struct Listen<S: Stream>
    where S::Item: IntoFuture<Item=(), Error=()>,
{
    buffer: BufferUnordered<MapSwallowErrors<S>>,
}

pub fn new<S: Stream>(stream: S, limit: usize) -> Listen<S>
    where S::Item: IntoFuture<Item=(), Error=()>,
{
    Listen {
        buffer: MapSwallowErrors(stream).buffer_unordered(limit),
    }
}

impl<S: Stream> Future for Listen<S>
    where S::Item: IntoFuture<Item=(), Error=()>,
{
    type Item = ();
    type Error = S::Error;
    fn poll(&mut self) -> Result<Async<()>, S::Error> {
        loop {
            match self.buffer.poll()? {
                // Some future just finished, let's check for next one
                Async::Ready(Some(())) => continue,
                // No future ready
                Async::NotReady => return Ok(Async::NotReady),
                // Stream is done
                Async::Ready(None) => return Ok(Async::Ready(())),
            }
        }
    }
}

impl<F: Future<Item=(), Error=()>, E> Future for SwallowErrors<F, E> {
    type Item = ();
    type Error = E;  // actually void

    #[inline(always)]
    fn poll(&mut self) -> Result<Async<F::Item>, E> {
        match self.0.poll() {
            Ok(x) => Ok(x),
            Err(()) => Ok(Async::Ready(())),
        }
    }
}

impl<S: Stream> Stream for MapSwallowErrors<S>
    where S::Item: IntoFuture<Item=(), Error=()>,
{
    type Item = SwallowErrors<<S::Item as IntoFuture>::Future, S::Error>;
    type Error = S::Error;

    #[inline(always)]
    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, S::Error> {
        match self.0.poll()? {
            Async::Ready(None) => Ok(Async::Ready(None)),
            Async::Ready(Some(f)) => {
                Ok(Async::Ready(Some(
                    SwallowErrors(f.into_future(), PhantomData)
                )))
            }
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}
