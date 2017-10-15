Tokio Listen Helpers
====================

**Status: Beta**

[Documentation](https://docs.rs/tk-listen) |
[Github](https://github.com/tailhook/tk-listen) |
[Crate](https://crates.io/crates/tk-listen)


A library that allows to listen network sockets with proper resource limits
and error handling.

Basic challenges:

* Some connection accept errors (like "connection reset") must be ignored, some
  (like "too many files open") may consume 100% CPU when ignored. You need
  to know what to do with them every time
* Server must accept connections up to a certain limit to avoid DoS attacks
* Shutting down listener and update the set of addresses listened
  should be obvious to implement


Example
=======

Here is the basic example:

```rust

let TIME_TO WAIT_ON_ERROR = Duration::from_millis(100);
let MAX_SIMULTANEOUS_CONNECTIONS = 1000;

let mut lp = Core::new().unwrap();
let listener = TcpListener::bind(&addr, &lp.handle()).unwrap();
lp.run(
    listener.incoming()
    .sleep_on_error(TIME_TO_WAIT_ON_ERROR, &h2)
    .map(move |(mut socket, _addr)| {
         // Your future is here:
         Proto::new(socket)
         // Errors should not pass silently
         // common idea is to log them
         .map_err(|e| error!("Protocol error: {}", e))
    })
    .listen(MAX_SIMULTANEOUS_CONNECTIONS)
).unwrap(); // stream doesn't end in this case
```

More in [docs] and [examples]

[docs]: http://docs.rs/tk-listen/
[examples]: https://github.com/tailhook/tk-listen/tree/master/examples

License
=======

Licensed under either of

* Apache License, Version 2.0,
  (./LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (./LICENSE-MIT or http://opensource.org/licenses/MIT)
  at your option.

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

