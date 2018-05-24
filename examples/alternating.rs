extern crate tokio;
extern crate futures;
extern crate tk_listen;
extern crate env_logger;

#[macro_use] extern crate log;

use std::io::Write;
use std::env;
use std::time::{Duration, Instant};

use tokio::runtime::Runtime;
use tokio::timer::{Delay, Interval};

use futures::{Future, Stream, Sink};
use futures::sync::mpsc::channel;

use tk_listen::{ListenExt, BindMany};


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let addr1 = "0.0.0.0:8001".parse().unwrap();
    let addr2 = "0.0.0.0:8002".parse().unwrap();
    let (mut tx, rx) = channel(1);
    let listener = BindMany::new(rx.map_err(|_| "Error"));
    let mut n = 0;
    tx.start_send(vec![addr1]).unwrap();

    println!("This program will alternate listening \
             on ports 8001, 8002 each five seconds");

    let mut runtime = Runtime::new().unwrap();

    runtime.spawn(
        listener
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

    tokio::run(
        Interval::new(
            Instant::now() + Duration::new(5, 0),
            Duration::new(5, 0)
        )
            .for_each(move |_| {
                n += 1;
                let addr = if n % 2 == 0  { addr1 } else { addr2 };
                println!("Listening on {:?}", addr);
                tx.start_send(vec![addr]).unwrap();
                Ok(())
            })
            .map_err(|_| unreachable!())
    );
}
