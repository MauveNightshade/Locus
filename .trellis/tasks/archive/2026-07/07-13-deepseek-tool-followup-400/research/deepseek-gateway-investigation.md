# DeepSeek Tool Follow-Up 400 Investigation

## Root Cause

Locus was locked to `schannel 0.1.28`. Its TLS renegotiation path can resend
pending encrypted output after receiving `SEC_I_RENEGOTIATE`. Async reqwest
interleaves writes and reads while a large request is in flight, allowing a TLS
1.3 post-handshake message such as `NewSessionTicket` to trigger the defect.

The duplicated TLS output corrupts the HTTP request body after Locus has already
correctly serialized it. DeepSeek then reports a misleading JSON parser error
near the corrupted byte offset. Tool-call follow-ups are susceptible because their
large system prompt, history, tool results, and schemas make the request large
enough to expose the timing window.

schannel 0.1.29 adds an `is_renegotiating` guard that prevents this resend and
includes a multi-megabyte TLS data-corruption regression test.

## Production Reproduction

- Session: `576a7d1a-0573-45fb-8359-7d360ae1b40a`
- Workspace: `E:\UnityProjects\Guohezu`
- Official endpoint: `https://api.deepseek.com/anthropic`
- Model: `deepseek-v4-pro`
- The configured endpoint named `MICU-*` was never used.
- Latest failing body: 84,549 bytes, 4 wire messages, 27 tools
- SHA-256: `480FE80574E9134AD9C01031950B53DC496D5B23448A9E1881299CE78593F041`
- Failure: `HTTP 400 Failed to parse the request body as JSON` at column 82413

Locus generated three identical-body failures within 513 ms, each with a distinct
client request ID and the same parser error. A Rust diagnostic rebuilt the body
from the current Locus database; its six prepared history entries serialized to
the exact captured bytes. Sending those bytes through Locus's original
`network::reqwest_client` reproduced the same 400.

## Controls

The request body is valid JSON. A local recorder confirmed that reqwest and httpx
both sent HTTP/1.1 with `Content-Length: 84549`, no chunked encoding, and the
same received body SHA-256. Header ordering, casing, compression header variants,
body chunking, and forcing reqwest to HTTP/1-only did not alter the reqwest 400.

| Client or dependency state | Result |
| --- | --- |
| Running Locus / reqwest with Locus lockfile | parser 400 |
| Python httpx | 3 x 200 |
| Python raw TLS with Rust-style headers | 3 x 200 |
| Rust ureq + native-tls | 3 x 200 |
| Rust native-tls direct writes | 3 x 200 |
| reqwest 0.12.28 with a fresh compatible resolution | 3 x 200 |
| reqwest 0.12.28 with Locus lockfile + schannel 0.1.28 | 3 x 400 |

## Dependency A/B/A

Only the temporary probe lockfile changed during this comparison. The body,
reqwest version, headers, endpoint, HTTP version, and edge IP stayed constant.

| schannel version | Result |
| --- | --- |
| 0.1.28 | 3 x 400 |
| 0.1.29 | 3 x 200 |
| 0.1.28 again | 2 x 400 |
| 0.1.29 again | 2 x 200 |

## Fix

Update the resolved `schannel` dependency to 0.1.29 or newer. Do not change JSON
serialization, add a DeepSeek-specific retry, switch API clients, or use a
request-shape workaround.

## Validation

1. `cargo check --locked --bin locus` completed successfully with the updated lockfile.
2. The resolved runtime dependency graph is `reqwest -> native-tls -> schannel 0.1.29`.
3. The user replayed the Guohezu tool-call follow-up through the running Locus
   application and confirmed the repair for the configured official DeepSeek
   Anthropic and OpenAI-compatible paths.
4. No credentials or request contents are retained in committed artifacts.
