use std::io;
use std::time::Duration;

use futures::{Future, Stream, Async};
use tokio_core::reactor::{Handle, Timeout};


/// This function defines errors that are per-connection. Which basically
/// means that if we get this error from `accept()` system call it means
/// next connection might be ready to be accepted.
///
/// All other errors will incur a timeout before next `accept()` is performed.
/// The timeout is useful to handle resource exhaustion errors like ENFILE
/// and EMFILE. Otherwise, could enter into tight loop.
fn connection_error(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::ConnectionRefused ||
    e.kind() == io::ErrorKind::ConnectionAborted ||
    e.kind() == io::ErrorKind::ConnectionReset
}


pub struct SleepOnError<S> {
    stream: S,
    delay: Duration,
    handle: Handle,
    timeout: Option<Timeout>,
}

pub fn new<S>(stream: S, delay: Duration, handle: &Handle)
    -> SleepOnError<S>
{
    SleepOnError {
        stream: stream,
        delay: delay,
        handle: handle.clone(),
        timeout: None,
    }
}

impl<I, S: Stream<Item=I, Error=io::Error>> Stream for SleepOnError<S> {
    type Item = I;
    type Error = ();
    fn poll(&mut self) -> Result<Async<Option<I>>, ()> {
        if let Some(ref mut to) = self.timeout {
            match to.poll().expect("timeout never fails") {
                Async::Ready(_) => {}
                Async::NotReady => return Ok(Async::NotReady),
            }
        }
        self.timeout = None;
        loop {
            match self.stream.poll() {
                Ok(x) => return Ok(x),
                Err(ref e) if connection_error(e) => continue,
                Err(e) => {
                    debug!("Accept error: {}. Sleeping {:?}...",
                        e, self.delay);
                    let mut timeout = Timeout::new(self.delay, &self.handle)
                        .expect("can always set a timeout");
                    let result = timeout.poll()
                        .expect("timeout never fails");
                    match result {
                        Async::Ready(()) => continue,
                        Async::NotReady => {
                            self.timeout = Some(timeout);
                            return Ok(Async::NotReady);
                        }
                    }
                }
            }
        }
    }
}
