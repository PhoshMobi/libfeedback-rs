// Take a look at the license at the top of the repository in the LICENSE file.

use glib::translate::*;
use std::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering},
};

#[cfg(target_os = "macos")]
extern "C" {
    fn pthread_main_np() -> i32;
}

thread_local! {
    static IS_MAIN_THREAD: Cell<bool> = Cell::new(false)
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

// rustdoc-stripper-ignore-next
/// Asserts that this is the main thread and `lfb::init` has been called.
macro_rules! assert_initialized_main_thread {
    () => {
        #[allow(unknown_lints)]
        #[allow(clippy::if_then_panic)]
        if !crate::rt::is_initialized_main_thread() {
            if crate::rt::is_initialized() {
                panic!("Libfeedback may only be used from the main thread.");
            } else {
                panic!("Libfeedback has not been initialized. Call `lfb::init` first.");
            }
        }
    };
}

/// No-op.
macro_rules! skip_assert_initialized {
    () => {};
}

/// Asserts that `lfb::init` has not been called.
#[allow(unused_macros)]
macro_rules! assert_not_initialized {
    () => {
        assert!(
            !crate::rt::is_initialized(),
            "This function has to be called before `lfb::init`."
        );
    };
}

/// Returns `true` if Lfb has been initialized.
#[inline]
pub fn is_initialized() -> bool {
    skip_assert_initialized!();
    if cfg!(not(feature = "unsafe-assume-initialized")) {
        INITIALIZED.load(Ordering::Acquire)
    } else {
        true
    }
}

/// Returns `true` if Lfb has been initialized and this is the main thread.
#[inline]
pub fn is_initialized_main_thread() -> bool {
    skip_assert_initialized!();
    if cfg!(not(feature = "unsafe-assume-initialized")) {
        IS_MAIN_THREAD.with(|c| c.get())
    } else {
        true
    }
}

/// Informs this crate that Lfb has been initialized and the current thread is the main one.
///
/// # Panics
///
/// This function will panic if you attempt to initialize Lfb from more than
/// one thread.
///
/// # Safety
///
/// You must only call this if:
///
/// 1. You have initialized the underlying Lfb library yourself.
/// 2. You did 1 on the thread with which you are calling this function
/// 3. You ensure that this thread is the main thread for the process.
#[allow(unknown_lints)]
#[allow(clippy::if_then_panic)]
pub unsafe fn set_initialized() {
    skip_assert_initialized!();
    if is_initialized_main_thread() {
        return;
    } else if is_initialized() {
        panic!("Attempted to initialize Lfb from two different threads.");
    } else if !{ from_glib(ffi::lfb_is_initted()) } {
        panic!("Lfb was not actually initialized");
    }
    //  OS X has its own notion of the main thread and init must be called on that thread.
    #[cfg(target_os = "macos")]
    {
        assert_ne!(
            pthread_main_np(),
            0,
            "Attempted to initialize Lfb on OSX from non-main thread"
        );
    }
    INITIALIZED.store(true, Ordering::Release);
    IS_MAIN_THREAD.with(|c| c.set(true));
}

/// Tries to initialize Lfb.
///
/// Call this function for before using any other Lfb functions.
///
/// An Ok is returned if the windowing system was successfully
/// initialized otherwise an Err is returned.
#[doc(alias = "lfb_init")]
#[allow(unknown_lints)]
#[allow(clippy::if_then_panic)]
pub fn init(app_id: &str) -> Result<(), glib::BoolError> {
    skip_assert_initialized!();
    if is_initialized_main_thread() {
        return Ok(());
    } else if is_initialized() {
        #[cfg(not(test))]
        panic!("Attempted to initialize Lfb from two different threads.");
        #[cfg(test)]
        panic!("Use #[gtk::test] instead of #[test]");
    }

    unsafe {
        let mut error = std::ptr::null_mut();
        let is_ok;

        is_ok = from_glib(ffi::lfb_init(app_id.to_glib_none().0, &mut error));
        debug_assert_eq!(is_ok != true, !error.is_null());

        if is_ok {
            if !from_glib::<glib::ffi::gboolean, bool>(ffi::lfb_is_initted()) {
                return Err(glib::bool_error!("Lfb was not actually initialized"));
            }

            set_initialized();
            Ok(())
        } else {
            Err(glib::bool_error!("Failed to initialize Lfb"))
        }
    }
}
