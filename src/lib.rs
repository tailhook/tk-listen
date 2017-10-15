//!  A library that allows to listen network sockets with proper resource
//!  limits and error handling.
//!
//!  Library constists of three things:
//!
//!  * [`sleep_on_error`][1] -- filters `Stream` of accepted sockets for
//!    errors.  Simple errors like `ConnectionReset` are just ignored. Severe
//!    errors like `Too many files open` will delay next `accept()` call for
//!    the delay specified, effectively allowing other connections to be
//!    processed and release resources for new ones.
//!    [Replaces code like this][2].
//!  * [`listen`][3] -- iterates over a stream using [`buffer_unordered`][4]
//!    combinator. It also suppresses errors in futures (because otherwise
//!    every connection error would shut down the whole stream). And returns
//!    `ForEach`-like future, you can `run()` or combine with other futures.
//!    [Stands for code like this][5].
//!  * [`BindMany`] allows to bind to list of addresses and update that list
//!    (i.e. allow configuration reload), resulting into a single stream with
//!    accepted sockets. This a good idea to use it with [abstract-ns] to
//!    resolve list of names to addresses and keep them updated.
//!
//!  [1]: trait.ListenExt.html#method.sleep_on_error
//!  [2]: https://git.io/vy9vi#L41-L52
//!  [3]: trait.ListenExt.html#method.listen
//!  [4]: https://docs.rs/futures/0.1.11/futures/stream/trait.Stream.html#method.buffer_unordered
//!  [5]: https://git.io/vy9vi#L56-L59
//!  [abstract-ns]: https://docs.rs/abstract-ns
//!
//!  # Example
//!
//!  Simple example looks like this:
//!
//!  ```rust,ignore
//!    let TIME_TO_WAIT_ON_ERROR = Duration::from_millis(100);
//!    let MAX_SIMULTANEOUS_CONNECTIONS = 1000;
//!
//!    let mut lp = Core::new().unwrap();
//!    let listener = TcpListener::bind(&addr, &lp.handle()).unwrap();
//!    lp.run(
//!        listener.incoming()
//!        .sleep_on_error(TIME_TO_WAIT_ON_ERROR, &h2)
//!        .map(move |(mut socket, _addr)| {
//!             // Your future is here:
//!             Proto::new(socket)
//!             // Errors should not pass silently
//!             // common idea is to log them
//!             .map_err(|e| error!("Protocol error: {}", e))
//!        })
//!        .listen(MAX_SIMULTANEOUS_CONNECTIONS)
//!    ).unwrap(); // stream doesn't end in this case
//!  ```
//!
//!  # Example With Listener Shutdown
//!
//!  Because tk-listen works as a combinator trait, you can easily add
//!  things, like shutdown:
//!
//!  ```rust,ignore
//!    let (tx, rx) = oneshot::channel();
//!    lp.run(
//!        listener.incoming()
//!        .sleep_on_error(TIME_TO_WAIT_ON_ERROR, &h2)
//!        .map(move |(mut socket, _addr)| {
//!             // Your future is here:
//!             Proto::new(socket)
//!             // Errors should not pass silently
//!             // common Idea is to log them
//!             .map_err(|e| error!("Protocol error: {}", e))
//!        })
//!        .listen(MAX_SIMULTANEOUS_CONNECTIONS)
//!        .select(|_| rx)
//!    )
//!  ```
//!
//!  Now listener will be shut down either when `tx` is dropped or when
//!  you send a message via `tx`.
//!
//!  This is a "force shutdown", meaning it will close all active connections
//!  immediately. It's also possible to stop accepting by closing original
//!  stream (e.g. using `take_while`) and wait until all connections
//!  shutdown gracefully.
#![warn(missing_docs)]

extern crate futures;
extern crate tokio_core;

#[macro_use] extern crate log;

mod bind;
mod traits;
mod sleep_on_error;
mod listen;

pub use traits::ListenExt;
pub use sleep_on_error::SleepOnError;
pub use listen::Listen;
pub use bind::BindMany;
