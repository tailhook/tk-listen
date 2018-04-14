use std::io;
use std::time::{Duration, Instant};

use futures::{Future, Stream, Async};
use tokio::timer::Delay;


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


/// A structure returned by `ListenExt::sleep_on_error`
///
/// This is a stream that filters original stream for errors, ignores some
/// of them and sleeps on severe ones.
pub struct SleepOnError<S> {
    stream: S,
    delay: Duration,
    timeout: Option<Delay>,
}

pub fn new<S>(stream: S, delay: Duration) -> SleepOnError<S>
{
    SleepOnError {
        stream: stream,
        delay: delay,
        timeout: None,
    }
}

impl<I, S: Stream<Item=I, Error=io::Error>> Stream for SleepOnError<S> {
    type Item = I;
    type Error = ();
    fn poll(&mut self) -> Result<Async<Option<I>>, ()> {
        if let Some(ref mut to) = self.timeout {
            match to.poll().expect("delay never fails") {
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
                    let mut delay = Delay::new(Instant::now() + self.delay);
                    let result = delay.poll()
                        .expect("delay never fails");
                    match result {
                        Async::Ready(()) => continue,
                        Async::NotReady => {
                            self.timeout = Some(delay);
                            return Ok(Async::NotReady);
                        }
                    }
                }
            }
        }
    }
}
