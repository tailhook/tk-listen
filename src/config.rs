use std::time::Duration;
use std::sync::Arc;

use {Config};

impl Config {
    /// New configuration builder
    ///
    /// Be sure to set at least `max_connections`.
    pub fn new() -> Config {
        Config {
            retry_interval: Duration::from_millis(200),
            max_connections: 1000,
        }
    }
    /// Amount of time to delay accepting new connections if fatal error occurs
    ///
    /// Fatal error is something that probably blocks application ability
    /// to accept new connections, such as connection limit exceeded. The
    /// point of sleeping is to give other network subsystems to free some
    /// resources (e.g. close connections).
    ///
    /// Note: to prevent tight loop on unknown errors we always fallback to
    /// delaying next accept unless the error is known (for example it's
    /// known that `ConnectionReset` means that connection we were to accept
    /// dropped by peer, so next connection is probably fine to accept
    /// immediately).
    ///
    /// Default value is 200ms
    pub fn fatal_error_retry_interval(&mut self, value: Duration)
        -> &mut Self
    {
        self.retry_interval = value;
        self
    }
    /// Maximum number of simultaneous connections open
    ///
    /// By default we've chosen connection limit of 1000. This value is
    /// arbitrary you should configure it in 99% of cases.
    ///
    /// Note: currently we just stop accepting new connections if limit
    /// exceeded. It's responsibility of the application itself to time out
    /// or close existing connections.
    pub fn max_connections(&mut self, conn: usize) -> &mut Self {
        self.max_connections = conn;
        self
    }
    /// Finish building config and wrap it into an Arc
    pub fn done(&mut self) -> Arc<Config> {
        Arc::new(self.clone())
    }
}

