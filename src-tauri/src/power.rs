#[cfg(target_os = "windows")]
mod platform {
    #![allow(non_snake_case)]
    use std::sync::Mutex;

    const ES_CONTINUOUS: u32 = 0x80000000;
    const ES_SYSTEM_REQUIRED: u32 = 0x00000001;
    const ES_DISPLAY_REQUIRED: u32 = 0x00000002;

    type ExecutionState = u32;

    extern "system" {
        fn SetThreadExecutionState(es_flags: ExecutionState) -> ExecutionState;
    }

    pub struct PowerHandle {
        _private: (),
    }

    static HELD: Mutex<bool> = Mutex::new(false);

    pub fn prevent_sleep() -> Option<PowerHandle> {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
        }
        let mut held = HELD.lock().ok()?;
        *held = true;
        Some(PowerHandle { _private: () })
    }

    pub fn allow_sleep(handle: Option<PowerHandle>) {
        if handle.is_some() {
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
            if let Ok(mut held) = HELD.lock() {
                *held = false;
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::sync::Mutex;

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOPMAssertionCreateWithName(
            assertion_type: *const std::os::raw::c_char,
            level: u32,
            name: *const std::os::raw::c_char,
            id: *mut u32,
        ) -> i32;
        fn IOPMAssertionRelease(id: u32) -> i32;
    }

    pub struct PowerHandle(u32);

    static HELD: Mutex<Option<u32>> = Mutex::new(None);

    pub fn prevent_sleep() -> Option<PowerHandle> {
        let reason = std::ffi::CString::new("Resoná is playing").ok()?;
        let assertion = std::ffi::CString::new("PreventUserIdleSystemSleep").ok()?;
        let mut id: u32 = 0;
        unsafe {
            let ret = IOPMAssertionCreateWithName(assertion.as_ptr(), 255, reason.as_ptr(), &mut id);
            if ret == 0 {
                if let Ok(mut held) = HELD.lock() {
                    *held = Some(id);
                }
                Some(PowerHandle(id))
            } else {
                None
            }
        }
    }

    pub fn allow_sleep(handle: Option<PowerHandle>) {
        if let Some(h) = handle {
            unsafe {
                IOPMAssertionRelease(h.0);
            }
            if let Ok(mut held) = HELD.lock() {
                *held = None;
            }
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    pub struct PowerHandle(());

    pub fn prevent_sleep() -> Option<PowerHandle> {
        let _ = std::process::Command::new("systemd-inhibit")
            .arg("--what=sleep:idle")
            .arg("--mode=block")
            .arg("--who=Resoná")
            .arg("--why=Playing audio")
            .arg("sleep")
            .arg("infinity")
            .spawn();
        Some(PowerHandle(()))
    }

    pub fn allow_sleep(_handle: Option<PowerHandle>) {}
}

pub use platform::*;
