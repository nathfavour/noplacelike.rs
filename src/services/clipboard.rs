use arboard::Clipboard as SystemClipboard;
use std::sync::{Arc, Mutex};

/// A shareable clipboard type that can be used across the application
#[derive(Debug, Clone)]
pub struct SharedClipboard {
    data: Arc<Mutex<String>>,
}

impl SharedClipboard {
    /// Create a new empty shared clipboard
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(String::new())),
        }
    }
    
    /// Set the clipboard content
    pub fn set(&self, text: &str) -> Result<(), String> {
        // Update our internal clipboard
        {
            let mut guard = self.data.lock().map_err(|e| format!("Lock error: {}", e))?;
            *guard = text.to_string();
        }
        
        // Try to update system clipboard if available
        if let Ok(mut clipboard) = SystemClipboard::new() {
            if let Err(e) = clipboard.set_text(text) {
                eprintln!("Failed to update system clipboard: {}", e);
                // We don't return error here because our internal clipboard was updated
            }
        }
        
        Ok(())
    }
    
    /// Get the clipboard content
    pub fn get(&self) -> Result<String, String> {
        self.data.lock()
            .map(|guard| guard.clone())
            .map_err(|e| format!("Lock error: {}", e))
    }
    
    /// Try to get text from the system clipboard
    pub fn get_from_system(&self) -> Result<String, String> {
        match SystemClipboard::new() {
            Ok(clipboard) => {
                clipboard.get_text()
                    .map_err(|e| format!("Failed to get system clipboard: {}", e))
            },
            Err(e) => Err(format!("Failed to access system clipboard: {}", e)),
        }
    }
    
    /// Try to sync from system clipboard to our internal clipboard
    pub fn sync_from_system(&self) -> Result<(), String> {
        match self.get_from_system() {
            Ok(text) => self.set(&text),
            Err(e) => Err(e),
        }
    }
}

impl Default for SharedClipboard {
    fn default() -> Self {
        Self::new()
    }
}
