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
//! use thread_tryjoin::TryJoinHandle;
//!
//! let t = thread::spawn(|| { thread::sleep(Duration::from_secs(1)); });
//! assert!(t.try_join().is_err());
//! ```
#![deny(missing_docs)]

extern crate libc;

use std::{thread, ptr};
use std::os::unix::thread::JoinHandleExt;
use std::io::Error as IoError;

#[cfg(unix)]
extern "C" {
    fn pthread_tryjoin_np(thread: libc::pthread_t, retval: *mut *mut libc::c_void) -> libc::c_int;
}

/// Try joining a thread
pub trait TryJoinHandle {
    /// Try joining a thread
    fn try_join(&self) -> Result<(), IoError>;
}

#[cfg(unix)]
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
}

#[cfg(not(unix))]
impl<T> TryJoinHandle for thread::JoinHandle<T> {
    fn try_join(&self) -> Result<(), IoError> {
        Err(IoError::from_raw_os_error(2))
    }
}

#[test]
fn basic_join() {
    let t = thread::spawn(|| { "ok" });

    assert_eq!("ok", t.join().unwrap());
}

#[test]
fn basic_try_join() {
    use std::time::Duration;
    let t = thread::spawn(|| { "ok" });

    // Need to sleep just a tiny bit
    thread::sleep(Duration::from_millis(100));
    assert!(t.try_join().is_ok());
    assert_eq!("ok", t.join().unwrap());
}

#[test]
fn failing_try_join() {
    use std::time::Duration;
    let t = thread::spawn(|| { thread::sleep(Duration::from_millis(500)); });

    let err = t.try_join().unwrap_err();
    // 16 is EBUSY
    assert_eq!(Some(16), err.raw_os_error());

    thread::sleep(Duration::from_secs(1));

    assert!(t.try_join().is_ok());
}
