# Tooling and Pipelines

## Automate Twice

If you perform a task twice, write a script for it.

### Why
- Reduces human error
- Creates documentation of the process
- Enables CI/CD automation
- Saves time on repetitive tasks

### Where to Put Scripts
```
scripts/
├── validate.sh     # Lint, format, test
└── [new-script].sh # Your automation
```

---

## CI/CD Priority

GitHub Actions is the primary automation platform.

### Existing Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push, PR | Format, lint, build, test on all platforms, E2E tests |
| `release.yml` | Push to fix-cli/ | Version bump, cross-platform build, release |
| `pages.yml` | Push to website/ | Deploy documentation site |
| `test-wsl.yml` | Push, PR | WSL-specific testing |
| `test-windows-shells.yml` | Push, PR | PowerShell and CMD deep testing |
| `test-install.yml` | Push, PR | Installation script testing |
| `test-distros.yml` | Push, PR | Linux distribution compatibility |

See [ADR-006](../ADR/006-cross-platform-testing-strategy.md) for testing strategy details.

### When to Modify Workflows
- Adding new build targets
- Changing test requirements
- Adding deployment steps

**Ask first** before modifying CI/CD workflows.

---

## Available Commands

### Development
```bash
# Build (macOS with Metal GPU)
cd fix-cli && cargo build --release --features metal

# Build (Linux/Windows CPU)
cd fix-cli && cargo build --release

# Run tests
cd fix-cli && cargo test

# Format code
cd fix-cli && cargo fmt

# Lint
cd fix-cli && cargo clippy
```

### Validation
```bash
# Run all checks (format, lint, test)
./scripts/validate.sh
```

### Manual Release
Releases are automated, but if needed:
```bash
# The release workflow handles this, but for reference:
git tag -a v0.x.x -m "Release v0.x.x"
git push origin v0.x.x
```

---

## Creating New Tools

When adding automation:

1. **Script location**: Put in `scripts/`
2. **Make executable**: `chmod +x scripts/your-script.sh`
3. **Add shebang**: `#!/bin/bash` or `#!/usr/bin/env python3`
4. **Document**: Add usage comments at the top
5. **Test**: Verify it works on a clean checkout

### Script Template
```bash
#!/bin/bash
# Description: What this script does
# Usage: ./scripts/your-script.sh [args]

set -e  # Exit on error

# Your code here
```

---

## Dependency Management

### Rust Dependencies
- Defined in `fix-cli/Cargo.toml`
- Lock file: `fix-cli/Cargo.lock` (committed)
- Update with: `cargo update`

### Adding Dependencies
1. Check if truly needed (prefer stdlib)
2. Verify the crate is maintained
3. Check license compatibility (MIT preferred)
4. **Ask first** for new dependencies

---

## Build Features

The Rust CLI supports feature flags:

| Feature | Platform | Purpose |
|---------|----------|---------|
| `metal` | macOS | GPU acceleration via Metal |
| `cuda` | Linux/Windows | GPU acceleration via CUDA |
| (none) | Any | CPU-only inference |

Example:
```bash
cargo build --release --features metal
```
