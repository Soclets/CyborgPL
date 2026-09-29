# `Synthia.ttsr` — Synthia's manifest (owner, 2026-09-29, in progress)

*Tankun's TOML Syntax Ripoff*: sections and `key = value` lines like TOML,
in the Limb's own voice, with dotted keys and no quotes.

```
[my identity]
name = shop
version = 1.0.0
time.created = 2026-09-29T17:40:00+07:00
time.stated_ver.created = 2026-09-29T17:40:00+07:00

[my needed things]
geometry.id = 3f9a2c1d4e5b6a708192a3b4c5d6e7f8091a2b3c4d5e6f708a9b0c1d2e3f4a5b
geometry.version = 1.2
money.path = ../money

[my target]
```

- **The sections are the owner's**: `[my identity]` (proposed by the owner
  as `[my credentials]`; renamed because "credentials" reads as passwords
  and tokens — owner: b), `[my needed things]`, `[my target]`.
- **Times** are ISO 8601 with the zone, `2026-09-29T17:40:00+07:00` (owner:
  a). `time.created` is when the Limb was first made; `time.stated_ver.created`
  when the version stated in `version` was made. **The publish time is the
  registry's**, recorded in its index, never written into the manifest
  (owner: b). A readable form of a time is a Synthia command's job, not a
  second line in the file (the owner first chose a readable line, then:
  "They could just run a Synthia CLI cmd"). Builds never read these times
  (reproducible builds).
- **A dependency is dotted keys** (owner: b): `geometry.version = 1.2`,
  `money.path = ../money`. A published one is named by **its ID**, which
  its creator generates: **256 bits of randomness, as 64 hex digits**,
  needed to register in the Limb registry (owner). The value takes no quotes
  or braces: the key already says it is an ID, and a line ends where its
  value does (owner: "yeah", after asking about `{}` and `""`).

Open: the rest of `[my identity]` (the Limb's own ID?), `[my target]`,
comments, what a registered name is for once IDs name Limbs.
