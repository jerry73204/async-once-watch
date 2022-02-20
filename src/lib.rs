use event_listener::Event;
use std::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering::*},
};

#[derive(Debug)]
pub struct OnceWatch<T>
where
    T: Sync,
{
    event: Event,
    data: AtomicPtr<T>,
}

impl<T> OnceWatch<T>
where
    T: Sync,
{
    pub fn new() -> Self {
        let event = Event::new();
        Self {
            event,
            data: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn set(&self, data: T) -> Result<(), T> {
        let data = Box::new(data);
        let ptr = Box::into_raw(data);

        let result = self
            .data
            .compare_exchange(ptr::null_mut(), ptr, AcqRel, Relaxed);

        match result {
            Ok(_) => {
                self.event.notify_additional(usize::MAX);
                Ok(())
            }
            Err(_) => {
                let data = unsafe { *Box::from_raw(ptr) };
                Err(data)
            }
        }
    }

    pub async fn get(&self) -> &T {
        let listener = self.event.listen();
        listener.await;
        let ptr = self.data.load(Acquire);
        unsafe { ptr.as_ref().unwrap() }
    }

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
