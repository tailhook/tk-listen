use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, IntoFuture, Stream};
use futures::future::{ok, Either};
use tokio_core::reactor::{Handle, Timeout};
use tokio_core::net::{TcpStream, TcpListener};

use {Config};


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

/// Spawn a TCP listener with specified config
///
/// When `shutter` future is done whole TCP listener is shut down
pub fn spawn_tcp<F, T, G>(
    addr: SocketAddr, config: &Arc<Config>, handle: &Handle, shutter: G, f: F)
    -> Result<(), io::Error>
    where F: Fn(TcpStream, SocketAddr) -> T + 'static,
          T: IntoFuture,
          G: IntoFuture,
          G::Future: 'static,
{
    let h1 = handle.clone();
    let listener = TcpListener::bind(&addr, &handle)?;
    let c1 = config.clone();
    let future = listener.incoming()
        .then(move |item| match item {
            Ok(x) => Either::A(ok(Some(x))),
            Err(ref e) if connection_error(e) => {
                Either::A(ok(None)) // skip error and proceed
            }
            Err(e) => {
                warn!("Error accepting: {}", e);
                Either::B(Timeout::new(c1.retry_interval, &h1).unwrap()
                    .and_then(|()| ok(None)))
            }
        })
        .filter_map(|x| x)
        .map(move |(socket, saddr)| {
            f(socket, saddr)
            .into_future()
            .then(|_| Ok(()))
        })
        .buffer_unordered(config.max_connections)
        .for_each(|()| Ok(()));

    handle.spawn(future
        .select(shutter.into_future().then(|_| Ok(())))
        .map(move |_| info!("Listener {} exited", addr))
        .map_err(move |(e, _)| info!("Listener {} exited: {}", addr, e)));
    Ok(())
}
