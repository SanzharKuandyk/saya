use anyhow::{Context, Result};

/// RAII guard for COM initialization
///
/// Ensures proper cleanup of COM resources by calling CoUninitialize
/// when the guard is dropped, even in case of panic or early return.
pub struct ComGuard;

impl ComGuard {
    /// Initialize COM for the current thread
    ///
    /// # Returns
    /// - `Ok(ComGuard)` if initialization succeeds
    /// - `Err` if COM initialization fails
    pub fn initialize() -> Result<Self> {
        unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                Some(std::ptr::null()),
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            )
            .ok()
            .with_context(|| "Failed to initialize COM")?;
        }
        Ok(ComGuard)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_com_guard_initialize() {
        let guard = ComGuard::initialize();
        assert!(guard.is_ok());
    }

    #[test]
    fn test_com_guard_drop() {
        {
            let _guard = ComGuard::initialize().unwrap();
            // Guard should clean up when dropped here
        }
        // If we can initialize again, cleanup worked
        let guard = ComGuard::initialize();
        assert!(guard.is_ok());
    }
}
