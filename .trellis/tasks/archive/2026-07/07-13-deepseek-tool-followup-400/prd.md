# Fix DeepSeek tool-call follow-up 400

## Goal

Allow a conversation using the user-configured official DeepSeek `deepseek-v4-pro`
endpoint to continue after an agent tool call without a provider HTTP 400 error.

## Background

Users report the failure with the same DeepSeek model configured through both the
OpenAI Chat Completions and Anthropic Messages custom API formats. A captured
Anthropic request has been reproduced through Locus's original request path.

The captured body is valid JSON and byte-identical to Locus's reconstruction.
Cross-language and dependency-bisect validation on 2026-07-14 isolated the fault
to schannel 0.1.28. During async TLS renegotiation it can resend pending encrypted
output, corrupting large request bodies. The same locked reqwest probe flips from
the parser 400 to 200 when only schannel changes to 0.1.29, and flips back when
downgraded.

## Requirements

1. Reproduce the failure against the configured official DeepSeek endpoint, never
   the endpoint whose configured name starts with `MICU-`.
2. Do not alter request serialization without evidence that the emitted bytes are
   invalid or incompatible.
3. Correct the affected Windows TLS dependency rather than changing request JSON
   or retrying the corrupted request.
4. Preserve compatible behavior for other custom endpoint providers and for normal
   non-tool conversations.
5. Preserve the validated schannel version in dependency resolution and verify the
   existing request paths against the official endpoint.
6. Do not log, commit, or expose configured API credentials.

## Acceptance Criteria

- [x] A real OpenAI-format DeepSeek `deepseek-v4-pro` request can follow a completed
  tool call and tool result without HTTP 400.
- [x] A real Anthropic-format DeepSeek `deepseek-v4-pro` request can follow the same
  completed tool interaction without HTTP 400.
- [ ] No DeepSeek-specific retry or request-shape workaround is introduced.
- [x] The resolved Windows schannel dependency includes the renegotiation fix.
- [x] Existing relevant backend checks pass.

## Out Of Scope

- Changing the selected model, DeepSeek credentials, or unrelated provider behavior.
- Switching to, testing against, or modifying the `MICU-` endpoint.
