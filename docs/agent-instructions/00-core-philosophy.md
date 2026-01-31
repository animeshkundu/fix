# Core Philosophy

## Docs = Code

Documentation is not an afterthought - it drives implementation.

### The Rule
- **No code without docs**: Before writing code for a new feature, create or update a spec in `docs/specs/`
- **Sync on completion**: After completing work, update `docs/HISTORY.md` with what was done
- **ADRs are permanent**: Check `docs/ADR/` before making architectural changes. Never delete ADRs - amend them with dated additions.

### Why This Matters
Future agents (and humans) will rely on documentation to understand context. Code without docs creates technical debt that compounds over time.

---

## The CEO Model

When multiple agents work on a task:

1. **CEO Agent**: The initiating agent orchestrates the work
   - Breaks down the task into sub-tasks
   - Assigns work to sub-agents
   - Synthesizes results
   - Makes final decisions

2. **Worker Agents**: Sub-agents execute specific tasks
   - Focus on their assigned scope
   - Report results back to the CEO
   - Do not make architectural decisions independently

### Handoff Protocol
When passing work between agents:
1. Document the current state in `docs/HISTORY.md`
2. List any blockers or open questions
3. Provide clear next steps

---

## First Principles Reasoning

Before implementing, think deeply:

### The Process
1. **Understand the problem**: What are we actually trying to solve?
2. **Research existing solutions**: What patterns exist? What did others do?
3. **Identify constraints**: Platform, performance, compatibility requirements
4. **Consider alternatives**: What are 2-3 ways to solve this?
5. **Make a decision**: Document the reasoning in an ADR if architectural

### Avoid
- Copying code without understanding it
- Assuming the first solution is the best
- Implementing without checking existing patterns in the codebase

---

## Updating History

After completing significant work:

### What to Record in `docs/HISTORY.md`
- Date and brief description of changes
- Key decisions made and why
- Any breaking changes or migrations needed
- Links to relevant PRs or commits

### Format
```markdown
## YYYY-MM-DD: Brief Title

- What was done
- Why it was done
- Any follow-up needed
```

---

## Summary Checklist

Before starting work:
- [ ] Read `docs/ADR/` for past decisions
- [ ] Check if a spec exists in `docs/specs/`
- [ ] Understand existing patterns in the codebase

After completing work:
- [ ] Update relevant documentation
- [ ] Run `scripts/validate.sh`
- [ ] Add entry to `docs/HISTORY.md` if significant
