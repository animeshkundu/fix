# Research and Web Usage

## Internet Research is First-Class

AI agents should actively use web search to find current information. Do not rely solely on training data.

---

## When to Research

### Always Research Before
- Using a library or API you haven't used recently
- Implementing a pattern you're not 100% confident about
- Making architectural decisions
- Adding new dependencies

### Research Until Saturation
Continue searching until you find:
- Consistent patterns across multiple sources
- Official documentation or examples
- Known issues or gotchas to avoid

---

## What to Verify

### Library Versions
```
Before: "I'll use serde for JSON"
After: Search "serde latest version rust 2025" to confirm current API
```

### API Compatibility
- Check if APIs have changed since training data
- Verify deprecated methods
- Confirm platform-specific behavior

### Best Practices
- Search for "[topic] best practices 2025"
- Look for official style guides
- Find production-tested patterns

---

## Research Workflow

1. **Identify the gap**: What do you need to know?
2. **Search broadly**: Use multiple queries
3. **Verify sources**: Prefer official docs, GitHub repos, and recent articles
4. **Synthesize**: Combine findings into actionable knowledge
5. **Document**: If it took effort to find, add it to docs

---

## Project-Specific Resources

### Official Documentation
- [llama.cpp](https://github.com/ggerganov/llama.cpp) - Inference engine
- [llama-cpp-2 (Rust)](https://github.com/utilityai/llama-cpp-rs) - Rust bindings
- [HuggingFace Hub](https://huggingface.co/docs) - Model hosting

### This Project
- Model: [animeshkundu/cmd-correct](https://huggingface.co/animeshkundu/cmd-correct)
- Repo: [animeshkundu/fix](https://github.com/animeshkundu/fix)

---

## Do Not Hallucinate

If you cannot find information:
1. State what you searched for
2. Explain what you couldn't verify
3. Ask for clarification or suggest alternatives

**Never invent API methods, function signatures, or behaviors.**
