use crate::LifecycleEvent;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn append_event(log_path: &str, event: &LifecycleEvent) -> Result<()> {
    // Step 1 — Read existing events if file exists
    let mut events: Vec<LifecycleEvent> = if Path::new(log_path).exists() {
        let content = fs::read_to_string(log_path)?;
        serde_json::from_str(&content)?
    } else {
        Vec::new()
    };

    // Step 2 — Push the new event
    events.push(event.clone());

    // Step 3 — Write the whole array back
    let json = serde_json::to_string_pretty(&events)?;
    fs::write(log_path, json)?;

    Ok(())
}
