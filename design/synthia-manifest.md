> **Abandoned 2026-10-22** (owner: "Synthia is abandoned now, CyJet is already friendly enough"): history only; CyborgPL's package manager is CyJet, manifest `CyJet.toml`.

# `Synthia.ttsr` — Synthia's manifest (owner, 2026-09-29, in progress)

*Tankun's TOML Syntax Ripoff*: sections and `key = value` lines like TOML,
in the strike's own voice, with dotted keys and no quotes.

```
[my identity]
id = 7c01e4b2d9f35a68e1b0c2d3f4a5968778695a4b3c2d1e0f9a8b7c6d5e4f3a2b
name = shop
version = 1.0.0
time.created = 2026-09-29T17:40:00+07:00
time.stated_ver.created = 2026-09-29T17:40:00+07:00

[my needed things]
core.asp.id = 4882d1f3bf7b11d59879e4921672f15e91369e25d55a10e3dcd24e9b3fbe676f
core.asp.version = 2026-10-06T18:04:29+07:00
limb.std.id = fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936
limb.std.version = 2026-10-06T12:57:14+07:00
limb.geometry.id = 3f9a2c1d4e5b6a708192a3b4c5d6e7f8091a2b3c4d5e6f708a9b0c1d2e3f4a5b
limb.geometry.version = 1.2
limb.money.path = ../money
power.operators_on_builtin_types = yes
power.c_calls.functions = sqrt, sin

[my target]
aarch64-macos.default = yes
aarch64-linux.runner = qemu-aarch64
x86_64-windows.runner = wine
```

- **The powers a strike uses are `power.` lines in `[my needed things]`**,
  beside `core.` and `limb.` (owner, 2026-10-08: "`a`", relayed by synthia
  claude, over a `[my powers]` section; asked "Does this say what it needs,
  or what it allows the others", synthia claude answered that it is what the
  strike itself uses, allowing or forbidding others being `deny`).
  `power.c_calls.functions = sqrt, sin` is written to `Ricochet.zv` as
  `ricochet::power.c_calls.functions["sqrt", "sin"]`; only the shape is checked,
  since plugins may add powers; `= no` is refused (an unused power has no
  line). **A program takes powers away in a `[my denied powers]` section**
  (owner, 2026-10-08: "`a`", relayed by synthia claude, over `deny.` lines in
  `[my needed things]`; a denial is a rule set for others, not a need),
  written to `Ricochet.zv`'s `deny`: `raw_memory = yes` (all of it),
  `c_calls.functions = system` (the things named), `c_calls.except = sqrt,
  sin` (all but these), `inline_assembly.for = fastjson` (only to that strike,
  by prefix); `= no` is refused.
- **The sections are the owner's**: `[my identity]` (proposed by the owner
  as `[my credentials]`; renamed because "credentials" reads as passwords
  and tokens — owner: b), `[my needed things]`, `[my target]`.
- **Times** are ISO 8601 with the zone, `2026-09-29T17:40:00+07:00` (owner:
  a). `time.created` is when the strike was first made; `time.stated_ver.created`
  when the version stated in `version` was made. **The publish time is the
  registry's**, recorded in its index, never written into the manifest
  (owner: b). A readable form of a time is a Synthia command's job, not a
  second line in the file (the owner first chose a readable line, then:
  "They could just run a Synthia CLI cmd"). **`synthia show`** shows
  the manifest (owner, 2026-09-29: "opt 1"); times as written, and
  **`synthia show --readable`** gives them readably (owner: "Yeah, not a
  separate command, just not exactly the same one", replacing "`c`, but it
  should be another command"): in the zone written in the file, plus how
  long ago — `29 September 2026, 17:40 (UTC+7), 2 hours ago`. The clock is
  read only for "ago", on display. `synthia show` prints the manifest
  **exactly as written**, comments, blank lines and order kept, once it is
  checked (owner: "Raw", "exactly, because there's `--readable` for the
  more readable ver"); `--readable` changes only the time values. Builds never read these times
  (reproducible builds).
- **A dependency is dotted keys** (owner: b), **each line starting with
  what it needs, `limb.` or `core.`** (owner, 2026-10-06, relayed by
  synthia claude: "`core.asp.id`, I think, and `limb.geometry.id`", then
  "yea" to a whole section; until then `geometry.version = 1.2`,
  `money.path = ../money`, a form now refused naming `limb.geometry.id`):
  `limb.geometry.version = 1.2`, `limb.money.path = ../money`. **A plugin is
  listed there by its ID and version** (`core.asp.id = …`,
  `core.asp.version = …`), not in a section of its own (owner: "inside
  `[my needed things]`", over a `[my cores]` section, which synthia claude
  recommended); a plugin and a strike may share a label. As in `Ricochet.zv`
  (2026-10-06: a plugin by registry ID; official IDs only until plugins can
  be loaded, and `core.x.path` refused for now; std needs the plugin ASP). A published one is named by **its ID**, which
  its creator generates: **256 bits of randomness, as 64 hex digits**,
  needed to register in the strike registry (owner). The value takes no quotes
  or braces: the key already says it is an ID, and a line ends where its
  value does (owner: "yeah", after asking about `{}` and `""`).

- **A strike states its own ID in `[my identity]`** (owner: a), generated
  once when it is created and never changed; the registry checks it
  matches when the strike registers.
- **The registry refuses an ID it already has, and a registered ID is
  permanent** (owner): it belongs to that strike for good, never released or
  reused. By accident two IDs never match — 2²⁵⁶ ≈ 1.16 × 10⁷⁷, and eight
  billion people making a billion strikes each would share one with chance
  ≈ 2.8 × 10⁻⁴⁰ — so the rule is against a copied ID, someone passing their
  strike off as another.

- **`[my target]` holds several targets, each with its settings** (owner:
  b), as dotted keys: a `runner` for a cross target's tests, `default` for
  the one built when none is named. One target is one line.

**Zomp's package manager can do whatever Synthia can** (owner, 2026-09-29:
"yea"), in its plainer Zag-syntax way; Synthia's edge is convenience,
never capability, so Zomp never needs it. The owner asked whether Zomp's
package manager could do "some Synthia shit", since otherwise "that feels
like the base lang not being able to do inline ASM". So `Ricochet.zv`'s
`target` became `targets`, a list of `ricochet::target[name = …, runner = …]`.

- **Needed in every manifest: `name`, `version`, and at least one
  target** (owner, 2026-09-29: b). `id`, `time.created` and
  `time.stated_ver.created` are needed only to publish, and checked then:
  a strike that never leaves its machine needs no ID.
- **A strike's own `version` is any text, at most 256 bytes** (owner,
  2026-09-29: "256 bytes it is"; "it doesn't require `.`"): room for a
  version written in Thai. What a needed strike's `version` means is the
  registry's, and open.
- **`prefix = cash`** in `[my identity]` is the word code writes in front
  of `::` to reach this strike (owner, 2026-09-29: "`prefix`"), as
  `Ricochet.zv`'s `'prefix'`; its `name` is a label apart from it.
  Required, as in `Ricochet.zv` (owner: "if it's not always required, it
  could be weird to CyborgPL users"). **Synthia's easy override** for two strikes with one
  prefix does what a Zomp user does by hand (owner, 2026-09-29: "Synthia
  just does what the CyborgPL programmer had to do"): it copies that strike
  into a folder of its own, changes the `prefix` in the copy, and hands
  Zomp an ordinary path dependency. The override is a dotted `prefix` line
  on the needed strike's key (owner, 2026-09-30: "Prefix shall be narrowed
  using a dot", "yea"): `money.path = ../money` with `money.prefix =
  cash2` (`limb.money.path`, `limb.money.prefix` since 2026-10-06), and code writes `cash2::`. The key (`money`) is this
  manifest's label for the strike, following the name rule; without a
  `prefix` line code writes the strike's own prefix. The copy lives in the
  project, in Synthia's own `.synthia/` folder beside `Synthia.ttsr`
  (owner, 2026-09-30: a): always safe to delete, since Synthia remakes
  it, and kept out of git.
- **Names follow the module-name rule** (owner, 2026-09-29: "yea"): a
  strike's `name` and each local name in `[my needed things]` are lowercase
  ASCII letters, digits and `_`, starting with a letter, not a keyword,
  not a Windows device name — each is written in code as `name::`.
  The prefix later took that job; `name` stays strict all the same, in
  both manifests (owner, 2026-09-30: "strict").
- **Comments are `#`**, a space after it, as in Zag (owner).
- **A line is written one way** (owner, 2026-09-29: "refuse"): `key =
  value` with one space each side of `=`, no indentation, and a comment on
  its own line. `name=shop`, an indented line, and a comment after a value
  are refused. The refusal shows the corrected line where there is one
  (`name=shop` → `name = shop`; owner: "I think it's correct"). A `#` that
  starts a value or follows a space is a comment after a value, and
  refused; a `#` inside a word is part of the value (`../money#2`; owner,
  2026-09-29: "yea").
- **A refusal is written in Zomp's message shape** (`diagnostics.md`;
  owner, 2026-09-29: "yea"): `error:`, `where:` (file, line, column,
  section), `code:`, and `fix:` when there is one right line to write.
- **Names may repeat; IDs never do** (owner: "Names could be the same, IDs
  can't"). A name is a label; the key before `.id` in `[my needed things]` is
  what this strike's code calls that dependency (`geometry.id = …`, now `limb.geometry.id`, →
  `geometry::area[…]`), so two strikes both called "geometry" live side by
  side as `geometry.id` and `geo2.id`, and a clash never breaks a build.
  Proposed alternative: names unique in the registry too, first come.
  Superseded the same day for `Ricochet.zv`: the word in front of `::` is
  the strike's own `'prefix'`, apart from its `name`, and a dependency names
  no word (`ricochet::dep[id = "…", version = "1.2"]`); see `tooling.md`. The
  `Synthia.ttsr` side is synthia claude's to ask.

- **`default` is `yes` or `no`** (owner, 2026-09-30: b), as `Ricochet.zv`'s
  `true` or `false`.
- **A target must be one Zomp can build** (owner, 2026-09-29: "Only
  what's actually doable"), from Zomp's own list, not any
  `architecture-system` name.
- **One target is marked the default** (owner, 2026-09-29: b), the one
  built when none is named: `aarch64-macos.default = yes` here, `default =
  true` on one `ricochet::target` in `Ricochet.zv`. With several targets exactly one
  is marked; a single target is the default by itself. Proposed
  alternatives: this machine's, if listed (compiler claude's suggestion,
  which takes the target from the host, against reproducible builds); the
  first listed; none.

Open: how an update proves it comes from the ID's owner.
