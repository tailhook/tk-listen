#![warn(missing_docs)]

extern crate futures;
extern crate tokio_core;

#[macro_use] extern crate log;


mod simple;
mod config;

pub use simple::spawn_tcp;


use std::time::Duration;

/// Configuration of a listener
#[derive(Clone)]
pub struct Config {
    retry_interval: Duration,
    max_connections: usize,
}
