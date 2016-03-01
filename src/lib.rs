//! thread_tryjoin
//!
//! **Please don't use this as is**
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
//! It only works on Linux though. And it works by transmuting the internal struct into a
//! crate-local struct, as the internals of `JoinHandle` are private to libstd.
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

use std::{thread, mem, ptr};
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::any::Any;

extern "C" {
    fn pthread_tryjoin_np(thread: libc::pthread_t, retval: *const libc::c_void) -> libc::c_int;
}

struct ImpThread {
    id: libc::pthread_t,
}

type AnyResult<T> = Result<T, Box<Any + Send + 'static>>;

struct Packet<T>(Arc<UnsafeCell<Option<AnyResult<T>>>>);
struct JoinInner<T> {
    native: Option<ImpThread>,
    _thread: thread::Thread,
    _packet: Packet<T>,
}
struct JoinHandle<T>(JoinInner<T>);

impl<T> JoinHandle<T> {
    fn as_inner(&self) -> &ImpThread { self.0.native.as_ref().unwrap()  }
}

unsafe fn join_handle_to_my<T>(handle: &thread::JoinHandle<T>) -> &JoinHandle<T> {
    mem::transmute(handle)
}

/// Try joining a thread
pub trait TryJoinHandle {
    /// Try joining a thread
    fn try_join(&self) -> Result<(), ()>;
}

#[cfg(unix)]
impl<T> TryJoinHandle for thread::JoinHandle<T> {
    fn try_join(&self) -> Result<(), ()> {
        unsafe {
            let h = join_handle_to_my(self);
            let inner = h.as_inner();

            match pthread_tryjoin_np(inner.id, ptr::null()) {
                0 => Ok(()),
                _ => Err(())
            }
        }
    }
}

#[cfg(not(unix))]
impl<T> TryJoinHandle for thread::JoinHandle<T> {
    fn try_join(&self) -> Result<(), ()> {
        Err(())
    }
}

#[test]
fn basic_join() {
    use std::time::Duration;
    let t = thread::spawn(|| { thread::sleep(Duration::from_secs(1)); });

    assert!(t.join().is_ok());
}

#[test]
fn basic_try_join() {
    use std::time::Duration;
    let t = thread::spawn(|| { thread::sleep(Duration::from_secs(1)); });
    assert!(t.try_join().is_err());
    thread::sleep(Duration::from_secs(2));
    assert!(t.try_join().is_ok());
}
