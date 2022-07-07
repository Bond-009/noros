use core::cell::UnsafeCell;
use core::hint;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

use super::{TryLockResult, TryLockError};

struct MovableMutex {
    inner: AtomicBool
}

impl MovableMutex {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: AtomicBool::new(false)
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        self.inner.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok()
    }

    #[inline]
    pub fn lock(&self) -> bool {
        while !self.try_lock() {
            self.lock_contended();
        }

        true
    }

    #[cold]
    fn lock_contended(&self) {
        while self.inner.load(Ordering::Relaxed) {
            hint::spin_loop();
        }
    }

    #[inline]
    pub fn unlock(&self) {
        self.inner.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed).unwrap();
    }
}

impl Default for MovableMutex {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Mutex<T> {
    inner: MovableMutex,
    data: UnsafeCell<T>
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner: MovableMutex::new(),
            data: UnsafeCell::new(data)
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock();
        MutexGuard::new(self)
    }

    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        if self.inner.try_lock() {
            Ok(MutexGuard::new(self))
        } else {
            Err(TryLockError::WouldBlock)
        }
    }
}


unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

#[must_use = "if unused the Mutex will immediately unlock"]
pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>,
    // PhantomData so Send doesn't get auto implemented
    // TODO: replace when negative trait bounds are implemented #68318
    _a: PhantomData<*const u8>
}

impl<'a, T> MutexGuard<'a, T> {
    const fn new(lock: &'a Mutex<T>) -> Self {
        Self {
            lock,
            _a: PhantomData
        }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.inner.unlock();
    }
}
