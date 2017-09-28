# thread_tryjoin

It is forked from https://github.com/badboy/thread_tryjoin-rs

Ever needed to wait for a thread to finish, but thought you can still do work until it
finishes?

[`JoinHandle#join()`](http://doc.rust-lang.org/stable/std/thread/struct.JoinHandle.html#method.join)
waits for the thread to finish and is blocking, so it doesn't allow you to try again and again.

Luckily there is a non-portable `pthread` API:
[`pthread_tryjoin_np`](http://linux.die.net/man/3/pthread_tryjoin_np)

This library provides convenient access through a `try_join` method on `JoinHandle`.
It only works on Linux though.

It uses [`JoinHandleExt`](https://doc.rust-lang.org/stable/std/os/unix/thread/trait.JoinHandleExt.html) to get to the underlying `pthread_t` handle.

# Usage

```toml
[dependencies]
thread_tryjoin = "0.2"
```

[Documentation](https://docs.rs/wild_thread_pool)

# Example

```rust
use std::time::Duration;
use std::thread;
use thread_tryjoin::TryJoinHandle;

let t = thread::spawn(|| { thread::sleep(Duration::from_secs(1)); });
assert!(t.try_join().is_err());
```

# License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.