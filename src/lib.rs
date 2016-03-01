extern crate libc;

use std::{thread, mem, ptr};
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::any::Any;

extern "C" {
    fn pthread_tryjoin_np(thread: libc::pthread_t, retval: *const libc::c_void) -> libc::c_int;
}

pub struct ImpThread {
    id: libc::pthread_t,
}

pub type AnyResult<T> = Result<T, Box<Any + Send + 'static>>;

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

trait TryJoinHandle {
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
