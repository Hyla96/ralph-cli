# PRD: Ralph Auto-Loop & Runner Status Bar

## Introduction

Enable the ralph runner to operate fully autonomously: detect when Claude has completed a task,
automatically spawn the next Claude session without user intervention, retry on failure, and stop
when all tasks pass or the iteration cap is hit. A redesigned runner status bar provides context
at a glance (current task, progress, loop count) while a toggle lets users switch between
automated and manual confirmation modes.

---

## Goals

- Reliably detect when a Claude session has finished its task via an explicit output sentinel.
- Let the ralph loop run end-to-end without manual keypresses when auto-continue is enabled.
- Retry failed Claude sessions automatically (non-zero exit) up to the iteration cap.
- Give the user an in-runner toggle to switch between auto and manual continuation.
- Show task name, task progress, and iteration count in the runner status bar.
- Show context-sensitive keybinding hints on the left of the status bar.

---

## User Stories

### US-001: Update the ralph agent to emit the completion sentinel

**Description:** As a ralph agent, I want to print `<promise>COMPLETE</promise>` to stdout when I
finish implementing a task so that the runner can detect completion without relying solely on exit
code.

**Background:** The runner (`src/ralph/runner.rs`) already scans PTY output for
`<promise>COMPLETE</promise>` and sends `RunnerEvent::Complete` when found. The problem is that
the ralph Claude agent does not yet emit this sentinel. This story updates the agent configuration
(the system prompt or CLAUDE.md seen by `claude --agent ralph`) so that Claude always prints the
sentinel after completing its work.

**Acceptance Criteria:**

- [ ] The ralph agent system prompt (e.g. `.claude/agents/ralph.md` or equivalent) instructs
      Claude to print `<promise>COMPLETE</promise>` on its own line as the final action when it
      finishes implementing a task.
- [ ] Running `claude --agent ralph "Implement the next task."` in a repo produces the sentinel
      string in its output when Claude considers the task done.
- [ ] The existing `RunnerEvent::Complete` path in `drain_tab_channel` is triggered correctly
      (transitions tab to `Done` and calls `load_current_workflow()`).
- [ ] `cargo build` and `cargo clippy -- -D warnings` pass.

---

### US-002: Add auto-continue toggle to `RunnerTab`

**Description:** As a user, I want to toggle between auto-continue and manual-continue modes
directly inside a runner tab so that I can let the loop run unattended or pause after each task.

**Details:**

- Add `auto_continue: bool` to `RunnerTab` (default: `false`).
- Add keybinding `a` in the runner tab (not forwarded to PTY) to toggle `auto_continue`.
- The toggle flips the value and shows a brief status message: `"Auto-continue ON"` or
  `"Auto-continue OFF"` (expires after 2 seconds, same pattern as other transient messages).
- The toggle is per-tab; tabs are independent.

**Acceptance Criteria:**

- [ ] `RunnerTab` has `pub auto_continue: bool`, initialised to `false`.
- [ ] Pressing `a` in a runner tab toggles `auto_continue` and shows a 2-second status message.
- [ ] `a` is added to the list of keys NOT forwarded to PTY (comment in `handle_events`).
- [ ] `cargo build` and `cargo clippy -- -D warnings` pass.

---

### US-003: Implement auto-loop behavior

**Description:** As a user with auto-continue enabled, I want the runner to automatically
spawn the next Claude session after each exit (and retry on failure) without showing the
`ContinuePrompt` dialog, so that the workflow completes without my intervention.

**Details:**

The change lives in `drain_tab_channel` in the `if done { … }` block, after the workflow is
reloaded. The current logic always shows `ContinuePrompt`; replace it with a branch on
`auto_continue`:

**Manual mode (`auto_continue = false`)** — existing behavior unchanged:
- Exit within iteration limit and tasks remaining → show `ContinuePrompt`.

**Auto mode (`auto_continue = true`)** — new behavior:
- `RunnerEvent::Complete` was received (sentinel) OR process exited with code 0 AND tasks remain:
  → call `spawn_next_iteration()` immediately (no dialog).
- Process exited with non-zero code AND tasks remain AND `iteration < MAX_ITERATIONS`:
  → write a log line `"\r\n[runner] Task failed (exit {code}), retrying… ({iteration}/{MAX})\r\n"`
  → call `spawn_next_iteration()`.
- `iteration >= MAX_ITERATIONS` → write `"\r\n[runner] Max iterations reached. Stopping.\r\n"`
  → set state to `Done`.
- Workflow complete (`is_complete()`) → set state to `Done`.

**Note:** `spawn_next_iteration()` already increments `iteration`. No other changes needed there.

**Acceptance Criteria:**

- [ ] With `auto_continue = true`, the `ContinuePrompt` dialog is never shown.
- [ ] With `auto_continue = true` and tasks remaining after a successful exit, a new Claude
      session starts automatically within one event-loop tick.
- [ ] With `auto_continue = true` and a non-zero exit code, a retry log line is written and a
      new Claude session starts.
- [ ] With `auto_continue = false`, behavior is identical to before this story (ContinuePrompt
      dialog, no auto-spawn).
- [ ] When all tasks pass (`is_complete()`) in auto mode, the tab transitions to `Done` without
      spawning another session.
- [ ] When `iteration >= MAX_ITERATIONS` in auto mode, the tab transitions to `Done` and the
      max-iterations message is written to the terminal.
- [ ] `cargo build` and `cargo clippy -- -D warnings` pass.

---

### US-004: Store current task info in `RunnerTab`

**Description:** As a developer, I need `RunnerTab` to record which task is currently being
executed so that the status bar can display the task name and progress.

**Details:**

- Add two fields to `RunnerTab`:
  ```rust
  pub current_task_id: Option<String>,
  pub current_task_title: Option<String>,
  ```
- Populate them in `start_runner()` and `spawn_next_iteration()` by loading the workflow and
  calling `next_task()` immediately before spawning the child process:
  ```rust
  let next = Workflow::load(&workflow_dir).ok().and_then(|w| w.next_task().map(|t| (t.id.clone(), t.title.clone())));
  tab.current_task_id = next.as_ref().map(|(id, _)| id.clone());
  tab.current_task_title = next.as_ref().map(|(_, title)| title.clone());
  ```
- Also store `iterations_used: u32` on `RunnerTab` and update it each time a new Claude session
  starts (set to the new `iteration` value). This lets `Done` and `Error` states still display
  the final iteration count in the status bar.

**Acceptance Criteria:**

- [ ] `RunnerTab` has `current_task_id: Option<String>`, `current_task_title: Option<String>`,
      and `iterations_used: u32`.
- [ ] After `start_runner()`, `current_task_title` holds the title of the first pending task (or
      `None` if the workflow has no pending tasks).
- [ ] After `spawn_next_iteration()`, `current_task_title` is updated to the next pending task.
- [ ] `iterations_used` reflects the current `iteration` number each time a session spawns.
- [ ] `cargo build` and `cargo clippy -- -D warnings` pass.

---

### US-005: Redesign runner tab status bar

**Description:** As a user, I want the runner tab's bottom status bar to show keybindings on the
left and task context (task name, progress, iteration) on the right so that I can monitor the
loop at a glance.

**Layout:**

```
[s]top [a]uto:ON                       US-002: Add toggle  3/8 tasks  iter 4
```

- **Left side** — context-sensitive keybinding hints:
  - `Running`:  `[s]top  [a]uto:ON` or `[s]top  [a]uto:OFF`
  - `Done`:     `[x]close`
  - `Error`:    `[x]close  [q]uit`
- **Right side** — task context (right-aligned, same mechanism as `notification_right_spans`):
  - When `Running` or `Done`: `{task_title}  {done}/{total} tasks  iter {n}`
    - `task_title`: `tab.current_task_title` truncated to 30 chars if needed.
    - `done/total`: loaded from `app.current_workflow` if the active runner tab matches the
      selected workflow, otherwise loaded fresh from disk per the tab's `workflow_name`.
    - `n`: `iteration` from `RunnerTabState::Running { iteration }` or `tab.iterations_used`
      when Done/Error.
  - When `Error`: right side omitted (error message fills the bar as today).

**Implementation notes:**

- The right-side string is assembled as a plain `String` and passed to
  `notification_right_spans` (or an equivalent helper) so existing padding/truncation logic is
  reused.
- If both a transient `status_message` and task context are present, the transient message takes
  the left side and the task context takes the right side (existing precedence rule).
- The `[a]uto:ON/OFF` indicator must reflect the live value of `tab.auto_continue`.

**Acceptance Criteria:**

- [ ] Running state left side shows `[s]top  [a]uto:ON` or `[s]top  [a]uto:OFF` depending on
      `tab.auto_continue`.
- [ ] Done state left side shows `[x]close`.
- [ ] Error state left side shows `[x]close  [q]uit` (error message dropped to the status bar
      only if `app.status_message` is set, as today).
- [ ] Right side shows `{task_title}  {done}/{total} tasks  iter {n}` for Running and Done
      states, right-aligned with padding.
- [ ] Right side is absent (or gracefully truncated) when the terminal is too narrow to fit both
      sides without overlap (existing `notification_right_spans` truncation logic).
- [ ] Toggling `a` updates the `[a]uto:ON/OFF` indicator on the next draw without any additional
      code (it reads `tab.auto_continue` live).
- [ ] `cargo build` and `cargo clippy -- -D warnings` pass.

---

## Functional Requirements

- FR-1: The ralph agent system prompt must instruct Claude to print `<promise>COMPLETE</promise>`
  on its own line as the last output when finishing a task.
- FR-2: `RunnerTab` must expose `auto_continue: bool` (default `false`), toggled by `a` in the
  runner tab.
- FR-3: When `auto_continue = true` and a Claude session exits successfully with tasks remaining,
  `spawn_next_iteration()` must be called automatically with no dialog.
- FR-4: When `auto_continue = true` and a Claude session exits with a non-zero code, a retry log
  line must be written and `spawn_next_iteration()` must be called, up to `MAX_ITERATIONS`.
- FR-5: When `auto_continue = false`, the `ContinuePrompt` dialog behavior must be unchanged.
- FR-6: `RunnerTab` must store `current_task_title`, `current_task_id`, and `iterations_used`,
  updated on every spawn.
- FR-7: The runner tab status bar left side must show context-sensitive keybindings; right side
  must show task title, done/total tasks, and iteration count, right-aligned.

---

## Non-Goals

- No persistent configuration for `auto_continue` (it resets to `false` when a new tab opens).
- No email/desktop notifications when the loop finishes.
- No UI to change `MAX_ITERATIONS` (remains a compile-time constant for now).
- No changes to the Workflows tab layout or keybindings.
- No retry back-off or delay between retries.

---

## Technical Considerations

- The sentinel detection path (`<promise>COMPLETE</promise>`) already exists in `runner_task`
  and sends `RunnerEvent::Complete`. US-001 only requires updating the agent configuration, not
  the Rust code.
- `spawn_next_iteration()` is already called from `handle_dialog_key`; calling it from
  `drain_tab_channel` (tokio task context, via `self`) is safe since `drain_runner_channels()`
  runs synchronously on the main thread inside `run()`.
- Loading the workflow in `start_runner` and `spawn_next_iteration` adds a small disk read per
  spawn; this is acceptable given spawns are infrequent.
- The right-side task-context string for the status bar is a plain formatted `String`; reuse
  `notification_right_spans` (or factor out a helper) to avoid duplicating padding logic.
- `MAX_ITERATIONS` is currently `10` and `const`. The auto-loop retry path must reference the
  same constant so manual and auto paths stay in sync.

---

## Success Metrics

- A workflow with 5 pending tasks runs to completion end-to-end with a single `r` keypress and
  `a` toggle (no further interaction required).
- The bottom bar shows the correct task title and progress at every point in the loop.
- Toggling `a` is reflected immediately in the status bar label.

---

## Open Questions

- Should the right-side task title show the `id` (e.g. `US-002`) or just the `title` text, or
  both? Current proposal: both, truncated — `US-002: Add toggle` (id + `: ` + title, max 30
  chars total).
- If `auto_continue` is `true` and Claude emits the sentinel but then exits non-zero (partial
  failure), which signal wins? Proposal: `Complete` takes precedence — treat as success and
  advance to the next task.
