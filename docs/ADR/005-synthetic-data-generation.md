# ADR 005: Synthetic Data Generation Strategy

## Status
Accepted

## Context
We need training data for shell command correction. Real-world error data is scarce and hard to collect. We need a scalable approach to generate diverse, realistic training examples.

## Decision
Generate ~150,000 synthetic training examples using four complementary data sources:

| Dataset | Count | Focus |
|---------|-------|-------|
| DS1: Single Commands | 35,000 | Typos, flags, permissions, paths, syntax |
| DS2: Chained Commands | 35,000 | Pipes, redirects, command chaining |
| DS3: Natural Language | 50,000 | NL to command translation |
| DS4: Top 100 Tools | 30,000 | Tool-specific corrections |

## Error Type Distribution (DS1)

| Type | Weight | Description |
|------|--------|-------------|
| Typos | 15% | Character swaps, deletions, insertions |
| Wrong flags | 25% | Single vs double dash, wrong flags |
| Permission errors | 15% | Missing/extra sudo |
| Path errors | 20% | Relative/absolute, typos in paths |
| Syntax errors | 25% | Quotes, escapes, operators |

## Shell Distribution

| Shell | Weight | Rationale |
|-------|--------|-----------|
| bash | 35% | Most common Unix shell |
| zsh | 25% | Default on macOS |
| powershell | 20% | Windows primary |
| cmd | 12% | Windows legacy |
| fish | 5% | Growing popularity |
| tcsh | 3% | BSD systems |

## Consequences

### Positive
- Scalable data generation
- Controllable error distributions
- Reproducible dataset
- Shell-specific templates
- Covers realistic error patterns

### Negative
- May not capture all real-world errors
- Synthetic patterns could be too regular
- Requires careful template design
- Need to validate data quality

## Quality Controls
- Single-char corrections limited to <5%
- No null corrections (incorrect == correct)
- Shell diversity validation
- Category balance checks
