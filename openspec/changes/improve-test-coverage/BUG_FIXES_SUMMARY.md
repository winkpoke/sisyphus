# Bug Fixes and Design Flaw Remediation Summary

**Date**: January 17, 2026
**Change**: improve-test-coverage
**Reviewed by**: Sisyphus (Automated Analysis)

## Overview

This document consolidates 24 bugs and design flaws identified during review of the test coverage improvement proposal. All critical issues have been addressed in the updated proposal files.

---

## Critical Bugs Fixed (14 issues)

### Bug #1: Duplicate insta dependency addition ✅ FIXED
**Location**: `tasks.md` Task 1.3 vs Task 5.2
**Fix**: Removed duplicate insta addition from Task 5.2, added note that dependency already added in Task 1.3

### Bug #2: Missing tokio-test dependency ✅ FIXED
**Location**: `tasks.md` Task 1.3, `proposal.md`
**Fix**: Added `tokio-test = "0.4"` to Task 1.3 and proposal.md dependencies list

### Bug #3: Missing tempfile dependency ✅ FIXED (was already present)
**Location**: `tasks.md`
**Fix**: Verified tempfile already exists in `crates/core/Cargo.toml` and `crates/tools/Cargo.toml`, added verification step to Task 1.3

### Bug #4: Contradictory coverage target for agent.rs ✅ FIXED
**Location**: `tasks.md` Task 2.3 vs `specs/workspace/spec.md`
**Fix**: Changed Task 2.3 from 85%+ to 90%+ to match spec

### Bug #5: Contradictory coverage target for session.rs ✅ FIXED
**Location**: `tasks.md` Task 2.4 vs `specs/workspace/spec.md`
**Fix**: Changed Task 2.4 from 80%+ to 90%+ to match spec

### Bug #6: Contradictory coverage target for openai.rs ✅ FIXED
**Location**: `tasks.md` Task 3.1 vs `specs/workspace/spec.md`
**Fix**: Changed Task 3.1 from 75%+ to 80%+ to match spec

### Bug #7: Contradictory coverage target for server/lib.rs ✅ FIXED
**Location**: `tasks.md` Task 3.3 vs `specs/workspace/spec.md`
**Fix**: Changed Task 3.3 from 70%+ to 80%+ to match spec

### Bug #8: Contradictory coverage target for cli-core ✅ FIXED
**Location**: `tasks.md` Task 4.1 vs `specs/workspace/spec.md`
**Fix**: Changed Task 4.1 from 65%+ to 70%+ to match spec

### Bug #9: Contradictory coverage target for client.rs ✅ FIXED
**Location**: `tasks.md` Task 4.2 vs `specs/workspace/spec.md`
**Fix**: Changed Task 4.2 from 60%+ to 70%+ to match spec

### Bug #10: Contradictory coverage target for tui ✅ VERIFIED (no issue)
**Location**: `tasks.md` Task 4.3 vs `specs/workspace/spec.md`
**Status**: Both specify 60%+, no change needed

### Bug #11: tempfile already exists but task didn't verify ✅ FIXED
**Location**: `tasks.md` Task 1.3
**Fix**: Added verification step to check existing tempfile dependency, updated proposal.md to note existing dependencies

### Bug #12: Existing custom mocks ignored in migration plan ✅ FIXED
**Location**: `tasks.md` Task 5.1, `design.md`
**Fix**: Added documentation of existing custom mocks in Task 5.1:
- `MockProvider` in `agent_test.rs`
- `MockTool` in `guardrails_test.rs`
- `MockProvider` in `repro_deny_test.rs`

### Bug #13: No verification for dev-dependency conflicts ✅ FIXED
**Location**: `tasks.md` Task 1.3
**Fix**: Added `cargo tree --dev` step to verify no dependency conflicts

### Bug #14: Missing task for context.rs module ✅ FIXED
**Location**: `tasks.md` Task 2.4, `specs/workspace/spec.md`
**Fix**: Added explicit task to test `core/src/session/context.rs` in Task 2.4 with 80%+ target

---

## Design Flaws Addressed (10 issues)

### Flaw #1: Inadequate security testing for shell injection ✅ ADDRESSED
**Location**: `specs/tooling/spec.md`
**Fix**: Added additional test scenarios:
- Command substitution (`$(rm -rf /)`)
- Environment variable expansion (`HOME=/tmp`)
- Heredocs (`<<EOF`)
- Backticks (kept existing)

### Flaw #2: Missing concurrency behavior specification ✅ ADDRESSED
**Location**: `specs/agent-core/spec.md`
**Fix**: Split concurrent scenarios:
- Same session: Returns error, no data races
- Different sessions: Process concurrently, isolated

### Flaw #3: Unrealistic coverage targets without baseline ✅ ADDRESSED
**Location**: `design.md`
**Fix**: Added "Incremental Milestones" section:
- Week 1: 21.7% → 40% overall
- Week 2: 40% → 60% overall
- Week 3: 60% → 80% overall
- Added reassessment protocol if milestone missed

### Flaw #4: Property-based testing not documented ✅ ADDRESSED
**Location**: `tasks.md` Task 5.3, `design.md`
**Fix**: Made property-based testing optional:
- Documented invariants to test
- Marked as "if time permits"
- Clarified as non-blocking if core deadlines at risk

### Flaw #5: Missing baseline measurement for 30s target ✅ ADDRESSED
**Location**: `tasks.md` Task 6.2
**Fix**: Added steps:
- Measure baseline with `cargo test --workspace --timings`
- Document baseline and optimized times
- Make 30s target after optimization

### Flaw #6: Missing server integration tests ✅ ADDRESSED
**Location**: `specs/server-core/spec.md`
**Fix**: Added new requirement "Server Integration Testing" with scenarios:
- Full request lifecycle
- Concurrent request handling
- Load test with multiple sessions

### Flaw #7: No mock-real drift detection mechanism ✅ ADDRESSED
**Location**: `design.md`, `tasks.md`
**Fix**: Added Task 6.4 (optional):
- `live-test` feature flag
- Contract tests comparing mock to real provider
- Document drift or update mocks

### Flaw #8: TUI test scope too narrow ✅ ADDRESSED
**Location**: `specs/cli-architecture/spec.md`
**Fix**: Added test scenarios:
- Keyboard event processing
- Terminal resize handling
- Screen component positioning

### Flaw #9: No test data management strategy ✅ ADDRESSED
**Location**: `design.md`, `tasks.md`
**Fix**: Added Decision 9 in design.md:
- Directory structure (fixtures/, snapshots/, golden/)
- Guidelines for test data organization
- Added test data organization to Task 1.4

### Flaw #10: Missing test cleanup/teardown scenarios ✅ ADDRESSED
**Location**: `specs/session-core/spec.md`, `specs/tooling/spec.md`
**Fix**: Added cleanup scenarios:
- Test cleanup on failure
- Temp directory cleanup verification
- Shell tool cleanup after timeout

---

## Files Modified

1. **proposal.md**
   - Added tokio-test to dependencies
   - Noted tempfile already present
   - Added "Known Issues Addressed" section

2. **tasks.md**
   - Fixed dependency conflicts (Bug #1, #2, #3)
   - Aligned all coverage targets with specs (Bug #4-#9)
   - Added context.rs testing (Bug #14)
   - Documented existing custom mocks (Bug #12)
   - Added dependency conflict verification (Bug #13)
   - Made property-based testing optional (Flaw #4)
   - Added baseline measurement (Flaw #5)
   - Added test data organization (Flaw #9)

3. **design.md**
   - Added mock-real drift detection (Flaw #7)
   - Added incremental milestones (Flaw #3)
   - Added test data management strategy (Flaw #9)

4. **specs/tooling/spec.md**
   - Added shell injection prevention scenarios (Flaw #1)
   - Added cleanup scenarios (Flaw #10)

5. **specs/agent-core/spec.md**
   - Specified concurrent chat behavior (Flaw #2)

6. **specs/session-core/spec.md**
   - Added cleanup scenarios (Flaw #10)

7. **specs/server-core/spec.md**
   - Added integration test requirement (Flaw #6)

8. **specs/cli-architecture/spec.md**
   - Added TUI test scenarios (Flaw #8)

---

## Validation Status

✅ All critical bugs fixed
✅ All design flaws addressed
✅ `openspec validate improve-test-coverage --strict` passed
✅ Coverage targets now consistent across tasks.md and specs/
✅ Existing codebase state accounted for
✅ Realistic milestones and timelines established

---

## Remaining Risks

### High Priority
1. **Test execution time may still exceed 30 seconds** even with optimization
   - **Mitigation**: Make some tests `#[ignore]` if necessary

2. **Property-based testing may be cut for time** (Task 5.3)
   - **Mitigation**: Already marked as optional

3. **Mock-real drift detection may not be feasible** (Task 6.4)
   - **Mitigation**: Already marked as optional

### Medium Priority
1. **CI/CD pipeline setup complexity** may delay Phase 1
   - **Mitigation**: Start with basic pipeline, iterate

2. **Learning curve for mockall** may slow migration
   - **Mitigation**: Migrate incrementally, start with critical tests

---

## Recommendations for Implementation

1. **Start with Phase 1 (Infrastructure)** immediately to establish baseline
2. **Track coverage incrementally** using the milestones in design.md
3. **Prioritize critical security modules** (fs.rs, cmd.rs) over lower-priority UI tests if time constrained
4. **Reassess timeline after Week 1** if any milestone is missed
5. **Keep Task 6.4 (mock drift detection) as stretch goal** only if ahead of schedule

---

## Conclusion

All 24 issues (14 critical bugs + 10 design flaws) have been addressed in the updated proposal. The proposal is now validated with `openspec validate --strict` and ready for review and implementation.

**Next Steps**:
1. Review updated proposal files
2. Request approval
3. Begin implementation with Phase 1 tasks
4. Track progress against weekly milestones
