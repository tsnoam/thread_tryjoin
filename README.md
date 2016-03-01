# thread_tryjoin

[![Build Status](https://travis-ci.org/badboy/thread_tryjoin-rs.svg?branch=master)](https://travis-ci.org/badboy/thread_tryjoin-rs)

**Please don't use this as is**

Ever needed to wait for a thread to finish, but thought you can still do work until it
finishes?

[`JoinHandle#join()`](http://doc.rust-lang.org/stable/std/thread/struct.JoinHandle.html#method.join)
waits for the thread to finish and is blocking, so it doesn't allow you to try again and again.

Luckily there is a non-portable `pthread` API:
[`pthread_tryjoin_np`](http://linux.die.net/man/3/pthread_tryjoin_np)

This library provides convenient access through a `try_join` method on `JoinHandle`.
It only works on Linux though. And it works by transmuting the internal struct into a
crate-local struct, as the internals of `JoinHandle` are private to libstd.

## Example

```rust
# use std::time::Duration;
# use std::thread;
use thread_tryjoin::TryJoinHandle;

let t = thread::spawn(|| { thread::sleep(Duration::from_secs(1)); });
assert!(t.try_join().is_err());
```
## [Documentation][]

[Documentation is available online.][documentation]

[documentation]: http://badboy.github.io/thread_tryjoin-rs/

## License

MIT. See [LICENSE](LICENSE).
