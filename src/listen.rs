use futures::{Future, Async};


pub struct Listen<S> {
    stream: S,
}

pub fn new<S>(stream: S, limit: usize) -> Listen<S> {
    Listen {
        stream: stream,
    }
}

impl<S> Future for Listen<S> {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<()>, ()> {
        unimplemented!();
    }
}
