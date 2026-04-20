@AGENTS.md

## Claude Code Specific Workflows

This file imports the main repository specification from AGENTS.md and adds Claude Code-specific instructions.

### Task Management

When working on multi-step tasks, use TodoWrite to break down work and track progress.

### Agent Usage

- Use subagents for broad codebase searches when simple Grep/Glob isn't enough
- Delegate independent research to subagents to keep main context clean

### Testing

Run `make diff` before committing to verify template rendering is correct.
