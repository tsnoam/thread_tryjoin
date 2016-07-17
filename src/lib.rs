//! # thread_tryjoin
//!
//! Ever needed to wait for a thread to finish, but thought you can still do work until it
//! finishes?
//!
//! [`JoinHandle#join()`](http://doc.rust-lang.org/stable/std/thread/struct.JoinHandle.html#method.join)
//! waits for the thread to finish and is blocking, so it doesn't allow you to try again and again.
//!
//! Luckily there is a non-portable `pthread` API:
//! [`pthread_tryjoin_np`](http://linux.die.net/man/3/pthread_tryjoin_np)
//!
//! This library provides convenient access through a `try_join` method on `JoinHandle`.
//! It only works on Linux though.
//! It uses [`JoinHandleExt`](https://doc.rust-lang.org/stable/std/os/unix/thread/trait.JoinHandleExt.html)
//! to get to the underlying `pthread_t` handle.
//!
//! Use an additional `join` to get to the actual underlying result of the thread.
//!
//! # Example
//!
//! ```rust
//! # use std::time::Duration;
//! # use std::thread;
//! # #[cfg(not(target_os = "linux"))]
//! # fn main() {}
//! # #[cfg(target_os = "linux")]
//! # fn main() {
//! use thread_tryjoin::TryJoinHandle;
//!
//! let t = thread::spawn(|| { thread::sleep(Duration::from_secs(1)); });
//! assert!(t.try_join().is_err());
//! # }
//! ```
//!
//! To perform a join-with-timeout there is a `try_timed_join` method.
//!
//! # Example join-with-timeout
//!
//! ```rust
//! # use std::time::Duration;
//! # use std::thread;
//! # #[cfg(not(target_os = "linux"))]
//! # fn main() {}
//! # #[cfg(target_os = "linux")]
//! # fn main() {
//! use thread_tryjoin::TryJoinHandle;
//!
//! let t = thread::spawn(|| {
//!     thread::sleep(Duration::from_millis(200));
//! });
//! assert!(t.try_timed_join(Duration::from_millis(500)).is_ok());
//! # }
//! ```
#![deny(missing_docs)]

extern crate libc;

use std::{thread, ptr};
use std::os::unix::thread::JoinHandleExt;
use std::io::Error as IoError;
use std::time::{self, Duration, SystemTime};

#[cfg(target_os = "linux")]
extern "C" {
    fn pthread_tryjoin_np(thread: libc::pthread_t, retval: *mut *mut libc::c_void) -> libc::c_int;
    fn pthread_timedjoin_np(thread: libc::pthread_t,
                            retval: *mut *mut libc::c_void,
                            abstime: *const libc::timespec) -> libc::c_int;
}

/// Try joining a thread.
pub trait TryJoinHandle {
    /// Try joining a thread.
    fn try_join(&self) -> Result<(), IoError>;

    /// Try joining a thread with a timeout.
    ///
    /// This waits for the specified duration.
    /// If the timeout expires before the thread terminates, the call returns an error.
    /// Otherwise it succeeds.
    fn try_timed_join(&self, wait: Duration) -> Result<(), IoError>;
}

#[cfg(target_os = "linux")]
impl<T> TryJoinHandle for thread::JoinHandle<T> {
    fn try_join(&self) -> Result<(), IoError> {
        unsafe {
            let thread = self.as_pthread_t();

            match pthread_tryjoin_np(thread, ptr::null_mut()) {
                0 => Ok(()),
                err @ _ => Err(IoError::from_raw_os_error(err))
            }
        }
    }
    fn try_timed_join(&self, wait: Duration) -> Result<(), IoError> {
        unsafe {
            let thread = self.as_pthread_t();

            let now = SystemTime::now();
            let future = now + wait;
            let total = future.duration_since(time::UNIX_EPOCH).expect("Can't get time offset");
            let abstime = libc::timespec {
                tv_sec: total.as_secs() as i64,
                tv_nsec: total.subsec_nanos() as i64
            };

            match pthread_timedjoin_np(thread, ptr::null_mut(), &abstime as *const libc::timespec) {
                0 => Ok(()),
                err @ _ => Err(IoError::from_raw_os_error(err))
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
impl<T> TryJoinHandle for thread::JoinHandle<T> {
    fn try_join(&self) -> Result<(), IoError> {
        Err(IoError::from_raw_os_error(2))
    }

    fn try_timed_join(&self, _wait: Duration) -> Result<(), IoError> {
        Err(IoError::from_raw_os_error(2))
    }
}

#[cfg(all(test, target_os = "linux"))]
mod test {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn basic_join() {
        let t = thread::spawn(|| { "ok" });

        assert_eq!("ok", t.join().unwrap());
    }

    #[test]
    fn basic_try_join() {
        let t = thread::spawn(|| { "ok" });

        // Need to sleep just a tiny bit
        thread::sleep(Duration::from_millis(100));
        assert!(t.try_join().is_ok());
        assert_eq!("ok", t.join().unwrap());
    }

    #[test]
    fn failing_try_join() {
        let t = thread::spawn(|| { thread::sleep(Duration::from_millis(500)); });

        let err = t.try_join().unwrap_err();
        // 16 is EBUSY
        assert_eq!(Some(16), err.raw_os_error());

        thread::sleep(Duration::from_secs(1));

        assert!(t.try_join().is_ok());
    }

    #[test]
    fn basic_timed_join() {
        let t = thread::spawn(|| { "ok" });
        assert!(t.try_timed_join(Duration::from_secs(1)).is_ok());
    }

    #[test]
    fn timed_join_timeout() {
        let t = thread::spawn(|| { thread::sleep(Duration::from_millis(500)); });
        assert!(t.try_timed_join(Duration::from_millis(100)).is_err());
    }

    #[test]
    fn timed_join_works() {
        let t = thread::spawn(|| { thread::sleep(Duration::from_millis(100)); });
        assert!(t.try_timed_join(Duration::from_millis(500)).is_ok());
    }
}
