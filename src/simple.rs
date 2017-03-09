use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, IntoFuture, Stream};
use futures::future::{ok, Either};
use tokio_core::reactor::{Handle, Timeout};
use tokio_core::net::{TcpStream, TcpListener};

use {Config};


/// Spawn a TCP listener with specified config
pub fn spawn_tcp<F, T>(
    addr: SocketAddr, config: &Arc<Config>, handle: &Handle,
    f: F) -> Result<(), io::Error>
    where F: Fn(TcpStream, SocketAddr) -> T + 'static,
          T: IntoFuture,
{
    let h1 = handle.clone();
    let listener = TcpListener::bind(&addr, &handle)?;
    let c1 = config.clone();
    handle.spawn(listener.incoming()
        .then(move |item| match item {
            Ok(x) => Either::A(ok(Some(x))),
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
        .for_each(|()| Ok(()))
        // TODO(tailhook) add shutdown somehow
        // .select(shutter.map_err(|_| unreachable!()))
        .map(move |()| info!("Listener {} exited", addr))
        .map_err(move |e| info!("Listener {} exited: {}", addr, e)));
    Ok(())
}
