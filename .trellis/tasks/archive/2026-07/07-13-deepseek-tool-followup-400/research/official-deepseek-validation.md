# Official DeepSeek Validation (Superseded)

This note recorded an earlier hypothesis that adjacent `user` messages caused
the failure. A later reproduction through Locus's original request path disproved
that diagnosis: the exact same valid request bytes can intermittently receive
the characteristic JSON-parser 400 and later return 200 without any payload
change.

The earlier `200 OK` results therefore demonstrated only that those individual
requests succeeded. They did not validate the adjacent-message hypothesis or a
serialization fix.

See [deepseek-gateway-investigation.md](./deepseek-gateway-investigation.md) for
the current evidence and root-cause boundary.
