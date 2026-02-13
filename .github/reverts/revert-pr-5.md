# Revert of PR #5 - Confirm

This PR documents the revert that was pushed directly to origin/main on commit 19c721e.

Summary:
- Reverted PR #5 (Merge commit c17c7c6), which added docs/AGENTS.md and specs/ffi_bridge.md.
- Revert commit: 19c721e (pushed to main).

Reason: Revert PR #5 to remove FFI/IPC docs from main while we audit the broader Rust vs hybrid agent migration plan and align stakeholders.

Requested action: Please confirm whether this revert is intended and whether we should keep the docs removed or reintroduce an updated version.

Approvers: @Coldaine (maintainer), @pmacl (initial author), and relevant stakeholders

---

Details:
- PR #5 added the FFI/IPC documentation (docs/AGENTS.md and specs/ffi_bridge.md).
- The revert removed those files while preserving the rest of the repository.

Context and rationale:
- PR #4 (feat/docs-foundation, merge commit 3bcfe50) introduced ADR-009 (pure Rust direction) and shifted the architecture away from hybrid agents, so the additional docs introduced in PR #5 now contradict the live plan unless we reconcile them first.
- PR #5 solely added `docs/AGENTS.md` and `specs/ffi_bridge.md`; the revert removes them while we clarify scope and confirm the intended direction.

Recommended next steps:
1. Confirm whether this revert should stay merged; if so, leave `main` without the FFI docs until we align the ADR and migration plan.
2. If the docs should live on `main`, prefer reintroducing them via a well-scoped PR that references ADR-009 and includes CI/test coverage for the Rust workspace.
3. Once agreement exists, add a targeted follow-up PR revising `specs/ffi_bridge.md` and `docs/AGENTS.md`, explaining how the FFI/Ipc bridge fits into the new architecture.

Key links:
- Revert commit: https://github.com/Coldaine/recall-pipeline/commit/19c721e7b27e9cfd86a577f996dc6b19a69b4515
- Original PR (#5): https://github.com/Coldaine/recall-pipeline/pull/5
- ADR impact: https://github.com/Coldaine/recall-pipeline/blob/main/docs/DECISIONS.md

Follow-up restoration:
- Restoration PR (#7): https://github.com/Coldaine/recall-pipeline/pull/7 reintroduces `docs/AGENTS.md` and `specs/ffi_bridge.md` with explicit hybrid + transitional disclaimers to correct earlier implication that a pure Rust-only architecture was finalized.
- Mixed pipeline status: Hybrid Rust + Python components remain supported; Rust-native agents are exploratory until a future ADR sets deprecation criteria.
