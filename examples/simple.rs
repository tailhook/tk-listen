extern crate tokio_core;
extern crate futures;
extern crate tk_listen;
extern crate env_logger;

#[macro_use] extern crate log;

use std::io::Write;
use std::env;
use std::time::Duration;

use tokio_core::reactor::{Core, Timeout};
use tokio_core::net::{TcpListener};
use futures::{Future, Stream};

use tk_listen::ListenExt;


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init().expect("init logging");

    let mut lp = Core::new().unwrap();
    let h1 = lp.handle();
    let h2 = lp.handle();

    let addr = "0.0.0.0:8080".parse().unwrap();
    let listener = TcpListener::bind(&addr, &lp.handle()).unwrap();

    lp.run(
        listener.incoming()
        .sleep_on_error(Duration::from_millis(100), &h2)
        .map(move |(mut socket, _addr)| {
            Timeout::new(Duration::from_millis(500), &h1).unwrap()
            .and_then(move |_| socket.write(b"hello\n"))
            .map(|_nbytes| ())
            .map_err(|e| error!("Conn error: {}", e))
        })
        .listen(1000)  // max connections
    ).unwrap();
}
