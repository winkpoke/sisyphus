# Tasks

- [ ] Define `EventEnvelope` and integrate it into `EventBus` publish/subscribe APIs
- [ ] Preserve topic-based routing and global auditing with enveloped events
- [ ] Update event logging to emit envelope metadata consistently
- [ ] Update server SSE stream (`/api/v1/events`) to send envelopes and set SSE id
- [ ] Update CLI/TUI event parsing (reqwest_eventsource) to handle envelope payloads
- [ ] Add tests for envelope serialization, ordering, and SSE event shape
- [ ] Run workspace test suite and fix failures
