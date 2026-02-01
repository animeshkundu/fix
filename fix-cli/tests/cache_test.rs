//! Tests for cache module

use fix_lib::cache::{
    cache_path, load_cache, load_or_create_cache, save_cache, ToolInfo, ToolsCache,
    CACHE_REFRESH_INTERVAL,
};
use std::fs;

#[test]
fn test_tool_info_serialization() {
    let info = ToolInfo {
        path: "/usr/bin/git".to_string(),
        desc: "distributed version control".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: ToolInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.path, info.path);
    assert_eq!(deserialized.desc, info.desc);
}

#[test]
fn test_tools_cache_creation() {
    let cache = ToolsCache::new();

    assert!(cache.tools.is_empty());
    assert!(!cache.last_updated.is_empty());
}

#[test]
fn test_tools_cache_age_fresh() {
    let cache = ToolsCache::new();
    let age = cache.age().unwrap();

    // Newly created cache should be very fresh
    assert!(age.as_secs() < 5);
}

#[test]
fn test_tools_cache_needs_refresh_fresh() {
    let cache = ToolsCache::new();

    // Brand new cache should not need refresh
    assert!(!cache.needs_refresh());
}

#[test]
fn test_tools_cache_needs_refresh_stale() {
    let mut cache = ToolsCache::new();

    // Set timestamp to 25 hours ago (older than 24h threshold)
    let old_time = chrono::Utc::now() - chrono::Duration::hours(25);
    cache.last_updated = old_time.to_rfc3339();

    assert!(cache.needs_refresh());
}

#[test]
fn test_tools_cache_update_timestamp() {
    let mut cache = ToolsCache::new();
    let old_timestamp = cache.last_updated.clone();

    // Wait a bit to ensure timestamp difference
    std::thread::sleep(std::time::Duration::from_millis(10));

    cache.update_timestamp();

    assert_ne!(cache.last_updated, old_timestamp);
}

#[test]
fn test_cache_refresh_interval_is_24_hours() {
    assert_eq!(CACHE_REFRESH_INTERVAL.as_secs(), 24 * 60 * 60);
}

#[test]
fn test_save_and_load_cache() {
    // Create a test cache
    let mut cache = ToolsCache::new();
    cache.tools.insert(
        "test_tool".to_string(),
        ToolInfo {
            path: "/usr/bin/test_tool".to_string(),
            desc: "A test tool".to_string(),
        },
    );

    // Save it
    let save_result = save_cache(&cache);
    assert!(
        save_result.is_ok(),
        "Failed to save cache: {:?}",
        save_result
    );

    // Load it back
    let loaded = load_cache().unwrap();

    assert_eq!(loaded.tools.len(), cache.tools.len());
    assert!(loaded.tools.contains_key("test_tool"));
    assert_eq!(
        loaded.tools.get("test_tool").unwrap().path,
        "/usr/bin/test_tool"
    );

    // Clean up
    let _ = fs::remove_file(cache_path());
}

#[test]
fn test_load_or_create_cache_creates_if_missing() {
    // Remove cache file if it exists
    let _ = fs::remove_file(cache_path());

    let cache = load_or_create_cache();

    // Should create a new cache successfully
    assert!(!cache.last_updated.is_empty());
}

#[test]
fn test_cache_path_location() {
    let path = cache_path();

    assert!(path.ends_with("tools_cache.json"));
    assert!(path.to_string_lossy().contains("fix"));
}

#[test]
fn test_tools_cache_with_multiple_tools() {
    let mut cache = ToolsCache::new();

    cache.tools.insert(
        "git".to_string(),
        ToolInfo {
            path: "/usr/bin/git".to_string(),
            desc: "distributed version control".to_string(),
        },
    );
    cache.tools.insert(
        "docker".to_string(),
        ToolInfo {
            path: "/usr/local/bin/docker".to_string(),
            desc: "container runtime".to_string(),
        },
    );

    assert_eq!(cache.tools.len(), 2);
    assert!(cache.tools.contains_key("git"));
    assert!(cache.tools.contains_key("docker"));
}

#[test]
fn test_cache_serialization_format() {
    let mut cache = ToolsCache::new();
    cache.tools.insert(
        "git".to_string(),
        ToolInfo {
            path: "/usr/bin/git".to_string(),
            desc: "version control".to_string(),
        },
    );

    let json = serde_json::to_string_pretty(&cache).unwrap();

    // Verify JSON structure
    assert!(json.contains("last_updated"));
    assert!(json.contains("tools"));
    assert!(json.contains("git"));
    assert!(json.contains("/usr/bin/git"));
    assert!(json.contains("version control"));
}
