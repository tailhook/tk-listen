extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate tk_http;
extern crate tk_listen;
extern crate env_logger;

use std::env;
use std::time::Duration;

use tokio_core::reactor::Core;
use tokio_core::net::{TcpListener};
use tokio_io::{AsyncRead, AsyncWrite};
use futures::{Future, Stream};
use futures::future::{FutureResult, ok};

use tk_http::{Status};
use tk_http::server::buffered::{Request, BufferedDispatcher};
use tk_http::server::{self, Encoder, EncoderDone, Proto, Error};
use tk_listen::ListenExt;

const BODY: &'static str = "Hello World!";

fn service<S:AsyncRead+AsyncWrite>(_: Request, mut e: Encoder<S>)
    -> FutureResult<EncoderDone<S>, Error>
{
    e.status(Status::Ok);
    e.add_length(BODY.as_bytes().len() as u64).unwrap();
    e.add_header("Server", "tk-listen/http/example").unwrap();
    if e.done_headers().unwrap() {
        e.write_body(BODY.as_bytes());
    }
    ok(e.done())
}


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init().expect("init logging");

    let mut lp = Core::new().unwrap();
    let h1 = lp.handle();
    let h2 = lp.handle();

    let addr = "0.0.0.0:8080".parse().unwrap();
    let scfg = server::Config::new().done();
    let listener = TcpListener::bind(&addr, &lp.handle()).unwrap();

    lp.run(
        listener.incoming()
        .sleep_on_error(Duration::from_millis(100), &h2)
        .map(move |(socket, addr)| {
            Proto::new(socket, &scfg,
                BufferedDispatcher::new(addr, &h1, || service),
                &h1)
            // Log error, effectively making error type of a future nil `()`
            .map_err(|e| { println!("Connection error: {}", e); })
        })
        .listen(1000)  // max connections
    ).unwrap();
}
