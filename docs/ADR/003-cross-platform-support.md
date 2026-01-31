# ADR-003: Cross-Platform Support

## Status
Accepted

## Context

The CLI needs to work on Windows, Linux, and macOS across different shells (bash, zsh, fish, powershell, cmd). This affects:
- Config file locations
- GPU backends
- Shell detection
- Log suppression

## Decision

Implement cross-platform support using:
1. **dirs crate** for platform-appropriate paths
2. **Cargo features** for GPU backends
3. **Conditional compilation** for platform-specific code
4. **Environment variable detection** for shells

## Rationale

1. **dirs crate**: Battle-tested, follows platform conventions
2. **Cargo features**: Clean separation of platform code
3. **cfg attributes**: Compile-time platform detection
4. **Minimal dependencies**: No heavy cross-platform frameworks

## Consequences

### Positive
- Single codebase for all platforms
- Platform-appropriate config locations
- Clean conditional compilation

### Negative
- Need to test on all platforms
- Some features (stderr redirect) are platform-specific
- Build matrix complexity

## Implementation

### Config Paths

```rust
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("cmd-correct")
}
```

| Platform | Result |
|----------|--------|
| macOS | `~/Library/Application Support/cmd-correct/` |
| Linux | `~/.config/cmd-correct/` |
| Windows | `C:\Users\<user>\AppData\Roaming\cmd-correct\` |

### Shell Detection

```rust
fn detect_shell() -> String {
    // Unix: check $SHELL
    if let Ok(shell_path) = env::var("SHELL") {
        if let Some(name) = shell_path.rsplit('/').next() {
            return name.to_string();
        }
    }
    // PowerShell (any platform)
    if env::var("PSModulePath").is_ok() {
        return "powershell".to_string();
    }
    // Windows fallback
    if cfg!(windows) { return "cmd".to_string(); }
    // Unix fallback
    "bash".to_string()
}
```

### Conditional Dependencies

```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

### Platform-Specific Code

```rust
#[cfg(unix)]
mod stderr_redirect {
    pub fn redirect() -> Option<i32> { /* Unix implementation */ }
    pub fn restore(saved: i32) { /* ... */ }
}

#[cfg(windows)]
mod stderr_redirect {
    pub fn redirect() -> Option<()> { None }
    pub fn restore(_: ()) {}
}
```
