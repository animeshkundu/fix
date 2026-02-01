# Testing and Validation

## Coverage Target: 90%

All new code should have comprehensive test coverage.

---

## Test-Driven Development

### The Workflow
1. **Write the test first** (or alongside implementation)
2. **Run the test** - it should fail initially
3. **Implement the feature** - make the test pass
4. **Refactor** - clean up while tests stay green

### Why This Order
- Forces clear thinking about requirements
- Catches edge cases early
- Creates documentation through tests

---

## Test Categories

### Unit Tests
- Test individual functions in isolation
- Mock external dependencies
- Fast execution (< 1 second per test)

```rust
#[test]
fn test_shell_detection() {
    // Test specific function behavior
}
```

### Integration Tests
- Test component interactions
- Use real dependencies where practical
- May be slower but verify real behavior

### CLI Tests
For this project, test the command-line interface:
- Correct output for valid inputs
- Appropriate errors for invalid inputs
- Flag combinations work as documented

---

## Self-Correction Workflow

Before committing code:

```bash
# Run the validation script
./scripts/validate.sh

# Or manually:
cd fix-cli
cargo fmt --check   # Formatting
cargo clippy        # Linting
cargo test          # Tests
```

### If Tests Fail
1. **Read the error message** carefully
2. **Identify the root cause** - don't just fix symptoms
3. **Fix the issue** in both code and tests if needed
4. **Re-run all tests** - ensure no regressions

---

## What to Test

### Must Test
- Public API functions
- Error handling paths
- Edge cases (empty input, large input, special characters)
- Configuration parsing

### Can Skip
- Private helper functions (tested indirectly)
- Trivial getters/setters
- External library behavior

---

## Test File Organization

```
fix-cli/
├── src/
│   └── main.rs         # Implementation with unit tests
└── tests/              # Integration tests
    ├── cli_test.rs     # Binary execution tests
    ├── config_test.rs  # Cross-platform config tests
    ├── wsl_test.rs     # WSL-specific tests
    └── e2e_test.rs     # Model inference tests (requires model)
```

### Running E2E Tests
```bash
# E2E tests are ignored by default (require model download)
cargo test --test e2e_test -- --ignored --test-threads=1
```

For unit tests, place them in the same file:

```rust
// In main.rs or lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // ...
    }
}
```

---

## Checklist Before Commit

- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] New code has tests
- [ ] Edge cases are covered
