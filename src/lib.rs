//! Asynchronous and shareable container which value is set once.
//!
//! The [OnceWatch<T>](OnceWatch) is a container with [`get()`](OnceWatch::get) and [`set()`](OnceWatch::set) methods.
//! The values is set at most once. The readers call `get()` and wait until the value is ready.
//!
//! ```rust
//! use async_once_watch::OnceWatch;
//! use async_std::task::{sleep, spawn};
//! use once_cell::sync::Lazy;
//! use std::time::Duration;
//!
//! static STATE: Lazy<OnceWatch<u8>> = Lazy::new(OnceWatch::new);
//! let secret = 10;
//!
//! /* Writer */
//! spawn(async move {
//!     sleep(Duration::from_millis(500)).await;
//!
//!     // First write is fine
//!     let ok = STATE.set(secret).is_ok();
//!     assert!(ok);
//!
//!     // Second write is not allowed
//!     let ok = STATE.set(secret).is_ok();
//!     assert!(!ok);
//! });
//!
//! /* Reader */
//! spawn(async move {
//!     let received = *STATE.get().await;
//!     assert_eq!(received, secret);
//! });
//! ```

use std::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering::*},
};
use tokio::sync::Semaphore;

/// The shareable container which value is set once.
#[derive(Debug)]
pub struct OnceWatch<T>
where
    T: Sync,
{
    semaphore: Semaphore,
    data: AtomicPtr<T>,
}

impl<T> OnceWatch<T>
where
    T: Sync,
{
    /// Creates a new uninitialized instance.
    pub fn new() -> Self {
        Self {
            semaphore: Semaphore::new(0),
            data: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Sets the value to the container.
    ///
    /// It returns `Ok` in first call and `Err` in later calls.
    pub fn set(&self, data: T) -> Result<(), T> {
        let data = Box::new(data);
        let ptr = Box::into_raw(data);

        let result = self
            .data
            .compare_exchange(ptr::null_mut(), ptr, AcqRel, Relaxed);

        match result {
            Ok(_) => {
                self.semaphore.add_permits(usize::MAX >> 3);
                Ok(())
            }
            Err(_) => {
                let data = unsafe { *Box::from_raw(ptr) };
                Err(data)
            }
        }
    }

    /// Waits until the value is set and obtains the reference.
    pub async fn get(&self) -> &T {
        let _permit = self.semaphore.acquire().await.unwrap();
        let ptr = self.data.load(Acquire);
        unsafe { ptr.as_ref().unwrap() }
    }

    /// Try to get the value reference in non-blocking manner.
    ///
    /// It returns `None` if the value is not ready.
    pub fn try_get(&self) -> Option<&T> {
        let ptr = self.data.load(Acquire);
        unsafe { ptr.as_ref() }
    }
}

impl<T> Default for OnceWatch<T>
where
    T: Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for OnceWatch<T>
where
    T: Sync,
{
    fn drop(&mut self) {
        let ptr = self.data.load(Acquire);
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
    }
}
