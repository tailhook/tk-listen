#![warn(missing_docs)]

extern crate futures;
extern crate tokio_core;

#[macro_use] extern crate log;


mod simple;
mod config;
mod traits;
mod sleep_on_error;
mod listen;

pub use simple::spawn_tcp;
pub use traits::ListenExt;
pub use sleep_on_error::SleepOnError;
pub use listen::Listen;


use std::time::Duration;

/// Configuration of a listener
#[derive(Clone)]
pub struct Config {
    retry_interval: Duration,
    max_connections: usize,
}
