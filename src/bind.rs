use std::collections::HashMap;
use std::io;
use std::mem;
use std::net::SocketAddr;
use std::time::Duration;

use futures::{Future, Stream, Async};
use tokio_core::net::{TcpListener, Incoming, TcpStream};
use tokio_core::reactor::{Handle, Timeout};



/// This stream replaces ``tokio_core::net::Incoming`` and listens many sockets
///
/// It receives a stream of lists of addresses as an input.
/// When a new value received on a stream it adapts:
///
/// 1. Removes sockets not in set we're already received (already established
///    connections aren't interfered in any way)
/// 2. Adds sockets to set which wasn't listened before
///
/// Instead of failing on bind error it logs the error and retries in a
/// second (you can change the delay using `BindMany::retry_interval`)
///
/// It's good idea to pass a stream with a **Void** error, because on receiving
/// error `BindMany` will log a message (that doesn't contain an error) and
/// will shutdown. It's better to log specific error and send end-of-stream
/// instead, but that is user's responsibility.
///
/// Note: we track identity of the sockets by `SocketAddr` used to bind it,
/// this means `0.0.0.0` and `127.0.0.1` for example can be bound/unbound
/// independently despite the fact that `0.0.0.0` can accept connections for
/// `127.0.0.1`.
///
///  # Example
///
///  Simple example:
///
///  ```rust,ignore
///    lp.run(
///        BindMany::new(address_stream)
///        .sleep_on_error(TIME_TO_WAIT_ON_ERROR, &h2)
///        .map(move |(mut socket, _addr)| {
///             // Your future is here:
///             Proto::new(socket)
///             // Errors should not pass silently
///             // common idea is to log them
///             .map_err(|e| error!("Protocol error: {}", e))
///        })
///        .listen(MAX_SIMULTANEOUS_CONNECTIONS)
///    ).unwrap(); // stream doesn't end in this case
///  ```
///
///  Example using name resolution via abstract-ns + ns-env-config:
///
///  ```rust,ignore
///      extern crate ns_env_config;
///
///      let mut lp = Core::new().unwrap();
///      let ns = ns_env_config::init(&lp.handle()).unwrap();
///      lp.run(
///          BindMany::new(ns.resolve_auto("localhost", 8080)
///             .map(|addr| addr.addresses_at(0)))
///          .sleep_on_error(TIME_TO_WAIT_ON_ERROR, &h2)
///          .map(move |(mut socket, _addr)| {
///               // Your future is here:
///               Proto::new(socket)
///               // Errors should not pass silently
///               // common idea is to log them
///               .map_err(|e| eprintln!("Protocol error: {}", e))
///          })
///          .listen(MAX_SIMULTANEOUS_CONNECTIONS)
///      ).unwrap(); // stream doesn't end in this case
///  ```
///
///
pub struct BindMany<S> {
    addresses: S,
    retry_interval: Duration,
    retry_timer: Option<(Timeout, Vec<SocketAddr>)>,
    inputs: HashMap<SocketAddr, Incoming>,
    handle: Handle,
}

impl<S> BindMany<S> {
    /// Create a new instance
    pub fn new(s: S, handle: &Handle) -> BindMany<S>
    {
        BindMany {
            addresses: s,
            retry_interval: Duration::new(1, 0),
            retry_timer: None,
            handle: handle.clone(),
            inputs: HashMap::new(),
        }
    }

    /// Sets the retry interval
    ///
    /// Each time binding socket fails (including he first one on start) istead
    /// of immediately failing we log the error and sleep this interval to
    /// retry (by default 1 second).
    ///
    /// This behavior is important because if your configuration changes
    /// number of listening sockets, and one of them is either invalid or
    /// just DNS is temporarily down, you still need to serve other addresses.
    ///
    /// This also helps if you have failover IP which can only be listened
    /// at when IP attached to the host, but server must be ready to listen
    /// it anyway (this one might be better achieved by non-local bind though).
    pub fn retry_interval(&mut self, interval: Duration) -> &mut Self {
        self.retry_interval = interval;
        self
    }
}

impl<S> Stream for BindMany<S>
    where S: Stream,
        S::Item: IntoIterator<Item=SocketAddr>,
{
    type Item = (TcpStream, SocketAddr);
    type Error = io::Error;
    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, io::Error> {
        loop {
            match self.addresses.poll() {
                Ok(Async::Ready(None)) => {
                    info!("Listening stream reached end-of-stream condition");
                    return Ok(Async::Ready(None));
                }
                Ok(Async::Ready(Some(new))) => {
                    let mut old = mem::replace(&mut self.inputs,
                                               HashMap::new());
                    let mut backlog = Vec::new();
                    for addr in new {
                        if let Some(listener) = old.remove(&addr) {
                            self.inputs.insert(addr, listener);
                        } else {
                            match TcpListener::bind(&addr, &self.handle) {
                                Ok(l) =>  {
                                    self.inputs.insert(addr, l.incoming());
                                }
                                Err(e) => {
                                    backlog.push(addr);
                                    error!("Error binding {:?}: {}, \
                                        will retry in {:?}",
                                        addr, e, self.retry_interval);
                                }
                            }
                        }
                    }
                    if backlog.len() > 0 {
                        self.retry_timer = Some((
                            Timeout::new(self.retry_interval,
                                 &self.handle)
                            .expect("timeout never fails"),
                            backlog));
                    } else {
                        self.retry_timer = None;
                    }
                }
                Ok(Async::NotReady) => break,
                Err(_) => {
                    error!("Error in address stream");
                    return Ok(Async::Ready(None));
                }
            }
        }
        loop {
            if let Some((ref mut timer, ref mut backlog)) = self.retry_timer {
                match timer.poll().expect("timeout never fails") {
                    Async::Ready(()) => {
                        for addr in mem::replace(backlog, Vec::new()) {
                            match TcpListener::bind(&addr, &self.handle) {
                                Ok(l) =>  {
                                    self.inputs.insert(addr, l.incoming());
                                }
                                Err(e) => {
                                    backlog.push(addr);
                                    // Lower level on retry
                                    debug!("Error binding {:?}: {}, \
                                        will retry in {:?}",
                                        addr, e, self.retry_interval);
                                }
                            }
                        }
                        if backlog.len() > 0 {
                            *timer = Timeout::new(
                                self.retry_interval, &self.handle)
                                .expect("timeout never fails");
                            continue;  // need to poll timer
                        }
                        // fallthrough to cleaning timer
                    }
                    Async::NotReady => break,
                }
            }
            self.retry_timer = None;
            break;
        }
        for inp in self.inputs.values_mut() {
            loop {
                match inp.poll() {
                    Ok(Async::Ready(pair)) => {
                        return Ok(Async::Ready(pair));
                    }
                    Ok(Async::NotReady) => break,
                    Err(e) => return Err(e),
                }
            }
        }
        return Ok(Async::NotReady);
    }
}
