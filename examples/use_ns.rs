extern crate abstract_ns;
extern crate env_logger;
extern crate futures;
extern crate ns_router;
extern crate ns_std_threaded;
extern crate tk_listen;
extern crate tokio_core;

#[macro_use] extern crate log;

use std::io::Write;
use std::env;
use std::time::Duration;

use abstract_ns::HostResolve;
use tokio_core::reactor::{Core, Timeout};
use futures::{Future, Stream};
use ns_router::SubscribeExt;

use tk_listen::{ListenExt, BindMany};


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init().expect("init logging");

    let mut lp = Core::new().unwrap();
    let h1 = lp.handle();
    let h2 = lp.handle();

    let ns = ns_router::Router::from_config(&ns_router::Config::new()
        .set_fallthrough(ns_std_threaded::ThreadedResolver::new()
            .null_service_resolver()
            .interval_subscriber(Duration::new(1, 0), &h1))
        .done(),
        &lp.handle());

    println!("This program will listen on `localhost:8080`. \
              You can edit the /etc/hosts and see rebinds.");
    lp.run(
        BindMany::new(
            ns.subscribe_many(&["localhost"], 8080)
                .map(|a| a.addresses_at(0)),
            &h1)
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
