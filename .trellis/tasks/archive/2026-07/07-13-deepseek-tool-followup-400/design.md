# Design: DeepSeek Tool-Call Follow-Up 400

## Investigation Result

The original serialization hypothesis was disproved. A controlled client matrix
and dependency A/B/A bisect isolated the corruption to schannel 0.1.28's async
renegotiation handling. Detailed evidence is recorded in
`research/deepseek-gateway-investigation.md`.

The dependency correction is implemented and the user confirmed live application
validation through the running Locus application.

## Boundary

The production boundary is dependency resolution only: schannel is updated from
0.1.28 to 0.1.29. Tool execution, request construction, provider logic, retry
policy, UI, endpoint settings, and persisted history remain outside the change.

## Approach

The lockfile update is the sole production correction. No DeepSeek-specific retry,
HTTP client switch, or serialized-request change is included.

## Validation

After an approved dependency update, rebuild Locus and replay the captured
Guohezu request through `stream_chat_native` and the OpenAI-compatible path.
The user completed this validation through the running Locus application and
confirmed the repair. The resolved dependency version is verified separately;
no credentials or request content are retained in committed artifacts.

## Rollback

No production change has been made. A future mitigation should remain isolated
enough that reverting its commit restores the current transport behavior.
