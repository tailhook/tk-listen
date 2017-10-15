extern crate tokio_core;
extern crate futures;
extern crate tk_listen;
extern crate env_logger;

#[macro_use] extern crate log;

use std::io::Write;
use std::env;
use std::time::Duration;

use tokio_core::reactor::{Core, Timeout, Interval};
use futures::{Future, Stream, Sink};
use futures::sync::mpsc::channel;

use tk_listen::{ListenExt, BindMany};


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init().expect("init logging");

    let mut lp = Core::new().unwrap();
    let h1 = lp.handle();
    let h2 = lp.handle();

    let addr1 = "0.0.0.0:8001".parse().unwrap();
    let addr2 = "0.0.0.0:8002".parse().unwrap();
    let (mut tx, rx) = channel(1);
    let listener = BindMany::new(rx.map_err(|_| "Error"), &h1);
    let mut n = 0;
    tx.start_send(vec![addr1]).unwrap();

    println!("This program will alternate listening \
             on ports 8001, 8002 each five seconds");
    h1.spawn(Interval::new(Duration::new(5, 0), &h1).unwrap()
        .for_each(move |()| {
            n += 1;
            let addr = if n % 2 == 0  { addr1 } else { addr2 };
            println!("Listening on {:?}", addr);
            tx.start_send(vec![addr]).unwrap();
            Ok(())
        })
        .map_err(|_| unreachable!()));
    lp.run(
        listener
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
