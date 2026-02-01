//! Cache management for tool discovery
//!
//! This module provides persistent caching for discovered CLI tools,
//! storing tool paths and descriptions to avoid repeated PATH scans.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Cache file name in the config directory
const CACHE_FILE: &str = "tools_cache.json";

/// Cache refresh interval (24 hours)
pub const CACHE_REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Information about a discovered tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolInfo {
    /// Path to the tool binary
    pub path: String,
    /// Description extracted from --help or --version
    pub desc: String,
}

/// Cache structure for discovered tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCache {
    /// Last update timestamp (ISO 8601)
    pub last_updated: String,
    /// Map of tool names to their info
    pub tools: HashMap<String, ToolInfo>,
}

impl ToolsCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            last_updated: chrono::Utc::now().to_rfc3339(),
            tools: HashMap::new(),
        }
    }

    /// Get the age of the cache
    pub fn age(&self) -> Result<Duration, String> {
        let last_updated = chrono::DateTime::parse_from_rfc3339(&self.last_updated)
            .map_err(|e| format!("Invalid timestamp: {}", e))?;

        let now = chrono::Utc::now();
        let age = now.signed_duration_since(last_updated);

        Ok(Duration::from_secs(age.num_seconds().max(0) as u64))
    }

    /// Check if the cache needs refresh (older than CACHE_REFRESH_INTERVAL)
    pub fn needs_refresh(&self) -> bool {
        self.age()
            .map(|age| age >= CACHE_REFRESH_INTERVAL)
            .unwrap_or(true) // Treat errors as needing refresh
    }

    /// Update the timestamp to now
    pub fn update_timestamp(&mut self) {
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }
}

impl Default for ToolsCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the path to the tools cache file
pub fn cache_path() -> PathBuf {
    crate::config_dir().join(CACHE_FILE)
}

/// Load the tools cache from disk
pub fn load_cache() -> Result<ToolsCache, String> {
    let path = cache_path();

    if !path.exists() {
        return Err("Cache file does not exist".to_string());
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read cache: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse cache: {}", e))
}

/// Save the tools cache to disk
pub fn save_cache(cache: &ToolsCache) -> Result<(), String> {
    let dir = crate::config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config directory: {}", e))?;

    let content = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("Failed to serialize cache: {}", e))?;

    let path = cache_path();
    fs::write(&path, content).map_err(|e| format!("Failed to write cache: {}", e))
}

/// Load cache or create a new one if it doesn't exist
pub fn load_or_create_cache() -> ToolsCache {
    load_cache().unwrap_or_else(|_| ToolsCache::new())
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_info_creation() {
        let info = ToolInfo {
            path: "/usr/bin/git".to_string(),
            desc: "distributed version control".to_string(),
        };

        assert_eq!(info.path, "/usr/bin/git");
        assert_eq!(info.desc, "distributed version control");
    }

    #[test]
    fn test_tools_cache_new() {
        let cache = ToolsCache::new();

        assert!(cache.tools.is_empty());
        assert!(!cache.last_updated.is_empty());
    }

    #[test]
    fn test_tools_cache_needs_refresh_new() {
        let cache = ToolsCache::new();

        // A newly created cache should not need refresh
        assert!(!cache.needs_refresh());
    }

    #[test]
    fn test_tools_cache_needs_refresh_old() {
        let mut cache = ToolsCache::new();

        // Set timestamp to 25 hours ago
        let old_time = chrono::Utc::now() - chrono::Duration::hours(25);
        cache.last_updated = old_time.to_rfc3339();

        assert!(cache.needs_refresh());
    }

    #[test]
    fn test_tools_cache_age() {
        let cache = ToolsCache::new();

        let age = cache.age().unwrap();
        // Age should be very small (just created)
        assert!(age.as_secs() < 5);
    }

    #[test]
    fn test_tools_cache_update_timestamp() {
        let mut cache = ToolsCache::new();
        let old_timestamp = cache.last_updated.clone();

        // Sleep a bit to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        cache.update_timestamp();

        assert_ne!(cache.last_updated, old_timestamp);
    }

    #[test]
    fn test_cache_serialization() {
        let mut cache = ToolsCache::new();
        cache.tools.insert(
            "git".to_string(),
            ToolInfo {
                path: "/usr/bin/git".to_string(),
                desc: "distributed version control".to_string(),
            },
        );

        let json = serde_json::to_string(&cache).unwrap();
        assert!(json.contains("git"));
        assert!(json.contains("/usr/bin/git"));

        let deserialized: ToolsCache = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tools.len(), 1);
        assert_eq!(deserialized.tools.get("git").unwrap().path, "/usr/bin/git");
    }

    #[test]
    fn test_cache_path_returns_correct_location() {
        let path = cache_path();

        assert!(path.ends_with("tools_cache.json"));
        assert_eq!(path.parent().unwrap(), crate::config_dir());
    }

    #[test]
    fn test_load_or_create_cache_creates_new() {
        // This should always succeed, creating a new cache if needed
        let cache = load_or_create_cache();

        // A newly created cache should have a timestamp
        assert!(!cache.last_updated.is_empty());
    }
}
