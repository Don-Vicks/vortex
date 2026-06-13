use crate::LifecycleEvent;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn append_event(log_path: &str, event: &LifecycleEvent) -> Result<()> {
    let mut events: Vec<LifecycleEvent> = if Path::new(log_path).exists() {
        let content = fs::read_to_string(log_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    events.push(event.clone());

    let json = serde_json::to_string_pretty(&events)?;
    fs::write(log_path, json)?;

    Ok(())
}

pub fn read_events(log_path: &str) -> Result<Vec<LifecycleEvent>> {
    if !Path::new(log_path).exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(log_path)?;
    let events = serde_json::from_str(&content)?;
    Ok(events)
}
