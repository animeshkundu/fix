# Technical Specifications

This directory contains technical specifications for features and changes.

## Purpose

**Write a spec before implementing significant features.**

Specs serve as:
- Design documents for review
- Implementation guides
- Historical record of decisions

---

## When to Write a Spec

### Required
- New features with user-facing changes
- Architectural modifications
- Changes affecting multiple components
- Breaking changes

### Optional
- Bug fixes (unless complex)
- Refactoring (unless architectural)
- Documentation updates

---

## Spec Template

```markdown
# Spec: [Feature Name]

## Status
Draft | In Review | Approved | Implemented

## Summary
One paragraph describing what this spec covers.

## Motivation
Why is this change needed? What problem does it solve?

## Design

### Overview
High-level description of the approach.

### Details
Specific implementation details:
- Data structures
- API changes
- File modifications

### Alternatives Considered
Other approaches and why they were rejected.

## Implementation Plan
1. Step one
2. Step two
3. ...

## Testing Strategy
How will this be tested?

## Open Questions
- [ ] Question 1
- [ ] Question 2
```

---

## File Naming

Use kebab-case with a descriptive name:
```
docs/specs/
├── README.md
├── add-fish-shell-support.md
├── model-caching-strategy.md
└── windows-installer.md
```

---

## Workflow

1. **Create spec** in this directory
2. **Get review** (if working with others)
3. **Implement** following the spec
4. **Update status** to "Implemented"
5. **Link** from ADR if architectural decision was made
