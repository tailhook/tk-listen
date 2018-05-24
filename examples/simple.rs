extern crate tokio;
extern crate futures;
extern crate tk_listen;
extern crate env_logger;

#[macro_use] extern crate log;

use std::io::Write;
use std::env;
use std::time::{Duration, Instant};

use tokio::net::TcpListener;
use tokio::runtime::run;
use tokio::timer::Delay;
use futures::{Future, Stream};

use tk_listen::ListenExt;


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let addr = "0.0.0.0:8080".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    run(
        listener.incoming()
        .sleep_on_error(Duration::from_millis(100))
        .map(move |mut socket| {
            Delay::new(Instant::now() + Duration::from_millis(500))
            .map(move |_| socket.write(b"hello\n"))
            .map(|result| {
                match result {
                    Ok(_) => (),
                    Err(e) => error!("Conn error: {}", e),
                }
            })
            .map_err(|_| ())
        })
        .listen(1000)  // max connections
    );
}
