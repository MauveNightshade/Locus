# Implementation Plan

1. Completed: reproduce the Anthropic failure through Locus's original request and
   network paths using the captured Guohezu session.
2. Completed: compare reconstructed and captured bytes and isolate serialization,
   headers, proxy behavior, protocol negotiation, field order, and edge IPs.
3. Completed: record the evidence and correct the invalid-serialization hypothesis.
4. Completed: reproduce the new stable failure window with the same Rust-generated
   body through both Locus Rust transport and Python/httpx.
5. Completed: compare local wire framing and run Python, ureq, native-tls, reqwest
   body-mode, HTTP-version, and dependency-resolution controls.
6. Completed: isolate schannel 0.1.28 with a lockfile-preserving A/B/A bisect and
   verify the 0.1.29 renegotiation corruption fix in source.
7. Completed: update the locked schannel dependency to 0.1.29 and remove the
   executable diagnostic entry point.
8. Completed: run `cargo check --locked --bin locus`.
9. Completed: validate through Locus against both real official DeepSeek API formats. The user
   confirmed the repaired formal application path; only status/results are recorded, never
   credentials or full sensitive request bodies.
