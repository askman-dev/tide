# Fix bug

## Configuration
- **Artifacts Path**: .zenflow/tasks/new-task-1309

---

## Workflow Steps

### [x] Step: Investigation and Planning

Analyze the bug report and design a solution.

1. Review the bug description, error messages, and logs
2. Clarify reproduction steps with the user if unclear
3. Check existing tests for clues about expected behavior
4. Locate relevant code sections and identify root cause
5. Propose a fix based on the investigation
6. Consider edge cases and potential side effects

Save findings to `.zenflow/tasks/new-task-1309/investigation.md` with:
- Bug summary
- Root cause analysis
- Affected components
- Proposed solution

### [x] Step: Implementation
Read `.zenflow/tasks/new-task-1309/investigation.md`
Implement the bug fixes.

1. Fix Splitter Width (layout.rs)
2. Fix Terminal Scrolling Direction (terminal.rs)
3. Fix File Tree Expansion Logic (panels.rs)
4. Verify/Fix File Tree Scrolling (app.rs/layout.rs)

If blocked or uncertain, ask the user for direction.
