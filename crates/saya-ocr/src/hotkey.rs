use anyhow::{Context, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
}

impl HotkeyManager {
    /// Create a new hotkey manager with Ctrl+Shift+S
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new().context("Failed to create hotkey manager")?;

        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);

        manager
            .register(hotkey)
            .context("Failed to register hotkey")?;

        Ok(Self { manager, hotkey })
    }

    /// Create with F9 hotkey
    pub fn new_f9() -> Result<Self> {
        let manager = GlobalHotKeyManager::new().context("Failed to create hotkey manager")?;

        let hotkey = HotKey::new(None, Code::F9);

        manager
            .register(hotkey)
            .context("Failed to register hotkey")?;

        Ok(Self { manager, hotkey })
    }

    /// Create with custom hotkey
    pub fn with_hotkey(modifiers: Modifiers, code: Code) -> Result<Self> {
        let manager = GlobalHotKeyManager::new().context("Failed to create hotkey manager")?;

        let hotkey = HotKey::new(Some(modifiers), code);

        manager
            .register(hotkey)
            .context("Failed to register hotkey")?;

        Ok(Self { manager, hotkey })
    }

    /// Check if hotkey was pressed (non-blocking)
    pub fn poll(&self) -> bool {
        let receiver = GlobalHotKeyEvent::receiver();
        if let Ok(event) = receiver.try_recv() {
            let is_match = event.id == self.hotkey.id();
            if is_match {
                println!("Hotkey event matched! ID: {:?}", event.id);
            } else {
                println!("Hotkey event but wrong ID. Got: {:?}, Expected: {:?}", event.id, self.hotkey.id());
            }
            is_match
        } else {
            false
        }
    }

    /// Wait for hotkey press (blocking)
    pub fn wait(&self) -> Result<()> {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            let event = receiver.recv().context("Failed to receive event")?;
            if event.id == self.hotkey.id() {
                return Ok(());
            }
        }
    }

    /// Get the hotkey ID for matching events
    pub fn id(&self) -> u32 {
        self.hotkey.id()
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.hotkey);
    }
}
