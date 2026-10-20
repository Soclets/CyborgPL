# Collections

Decided 2026-10-13 (owner), built the same day in the oracle and kayo.
The rulings, with the owner's words, are in `DECISIONS.md` under 2026-10-13;
this note says what they add up to. The example is `examples/collections.cy`.

> **2026-10-13, later (owner):** a collection always says its kind, and a
> bare `coll` is refused (and `collection.`, the spelling until 2026-10-19). Most of this note is **`coll.fn`**;
> the second kind, **`coll.handler`**, holds named `otherwise` blocks
> (the last section). Both built in the oracle the same day.

A collection writes a function and its dot narrowings (`cum`, `cum.quick`,
`cum.double`) in one block, instead of a `fn cum` and a `fn cum.quick` and a
`fn cum.double` each with its own `pub` and `bare.` mark.

## The shape

```
pub coll.fn cum {
    bare.never;
    cum ['x': int32, 'step': int32] -> int32 { return 'x' plus 'step'; }
    cum.quick ['step' = 2];
    cum.double ['x': int32] -> int32 { return 'x' times 2; }
}
```

- `coll.fn`, then the name, bare as a function's is (`cum`, never
  `'cum'`), then `{ }`.
- **The own function** carries the collection's name: `cum [ … ] -> T
  { … }`, called `cum[…]`, or `name::cum[…]` from another kit. ("Own
  function" is design claude's word for it; it has nothing to do with
  `MAIN`.)
- **A member** is a name, its parameters and its body, `double [ … ] -> T
  { … }`, called `cum.double[…]`.
- Nothing inside takes `fn` (owner: "maybe not `fn` Infront"); `fn` in a
  collection is refused.
- **A preset** is a member that only fills some of the own function's
  arguments, one line: `quick ['step' = 2];`. `cum.quick['x']` is
  `cum['x', 2]`. It takes the parameters it leaves open, in their order;
  each is passed on as the own function takes it (`lend` for `lent`,
  `lend.mut` for `lent.mut`, `give` for an owned value that is not plain,
  the value itself for a plain one). If the own function can fail, the
  preset fails the same ways. A preset names only the own function's
  parameters, and needs an own function to fill.

**Since 2026-10-19 (owner): `coll.`, and every member line writes its full name.** The header is `coll.fn` /
`coll.handler` (`collection.` refused as the old spelling), and each member line says what it makes:
`cum.quick ['step' = 2];`, `cum.double […] { … }`, so a line alone shows its name. A short `quick [ … ]` is refused,
naming `cum.quick`. Generated members are written the same way.

### A member written elsewhere (owner, 2026-10-19)

A member may point at a function written elsewhere in the kit, so a kit's `kit.cy` can be its front door:

```
pub coll.fn area {
    bare.never;
    area ['w': int64, 'h': int64] -> int64 { return 'w' x 'h'; }
    area.square = shapes.square.area;           # body in shapes/square.cy
    area.triangle = shapes.triangle.area;
    area.unit ['w' = 1, 'h' = 1];               # a preset of the own function
}
# users write geometry::area.square[5]
```

The path is read from the collection's module (`my.` for the kit's root), and the member is that function under
its new name, with the collection's visibility. `examples/coll_front_door/`. The single-line `pub fn area = …;`
(2026-09-23) stays for a one-off name.

## Versions: one name, several functions (owner, 2026-10-20)

A collection may hold **several versions** of its own function, each with its own parameters, and the call is
picked by how many arguments it has and their types. This is the only place overloading is written: the same
`fn` twice anywhere else is refused, naming `coll.fn` (owner: "(b)").

```
pub coll.fn round {
    bare.optional;
    round.near ['x': bin64] -> bin64 { … }
    round.near ['x': deci64] -> deci64 { … }
    round.near ['x': bin64, 'decimals': int32] -> bin64 { … }
}
```

- **Who**: any kit's functions, std's and yours (owner: "(a)"). Two versions with the same types are refused.
- **Across kits nothing joins** (owner: "(b)"): your `round.near ['x': Money]` beside std's bare one is the old
  clash wherever `round.near` is written without a path (`my.round.near[…]`, `std::round.near[…]`).
- **The count picks too** (owner: "(a)"): a function has no optional arguments, so `round.near['x']` and
  `round.near['x', decimals 2]` are two versions.
- Design claude's readings, not asked: a member may have versions as the own function may (std's `round.near`
  above); a preset beside several versions is refused until it is decided which one it fills.
- **Versions differ by exact type, widths counting** (owner: "(a)"): `bin16` and `bin64` versions stand side by
  side, and a call two could take is refused at the call: `round.near[2.5]`, a bare literal fitting both, names
  `2.5: bin64`. A variable has its type, so `round.near['x']` never is.

## Who sees it

- A collection is **private unless `pub coll`**, as everything else
  is (owner: "(a)").
- A `pub` collection makes every member public, under **one `bare.` mark**
  for them all, written in the block: `bare.never;` (first, by habit, as in
  the sketch; the oracle takes it anywhere in the block). Two marks are
  refused. The mark is the usual one (`ownership-syntax.md`, 2026-10-06):
  `bare.never`, `bare.optional` or `bare.only`, with no default.

## A collection with no own function

Allowed (owner, 2026-10-13: "(a)"). The name then only groups its members,
as `std::print` does: `std::print.useful[…]` and `std::print.stored[…]`
exist, a bare `std::print[…]` does not.

```
coll.fn zero {
    zero.of_int16 [] -> int16 { return 0; }
    zero.of_int64 [] -> int64 { return 0; }
}
```

`zero.of_int16[]` exists, `zero[…]` does not. Such a collection has no
presets, since a preset fills the own function.

## Templates

A template makes members whose names follow a pattern, for what a preset
cannot do (Good practice 14, `style.md`): where the value changes a
member's signature, or no own function takes it.

```
coll.fn zero {
    for 'T' in [int16, int64] {
        named "of_" ++ 'T';
        [] -> 'T' { return 0; }
    }
    #generated_2_lines_below
    zero.of_int16 [] -> int16 { return 0; }
    zero.of_int64 [] -> int64 { return 0; }
}
```

- **The loop is the usual `for`** (owner: "(a)"): over a list, `for 'n' in
  [2, 4, 8]`, or counting, `for 'n' from 1 to 8 by 1` (`before` too). Its
  values are known before building: literals, consts, and type names
  (`for 'T' in [int16, int32]`, allowed only in a template). Inside a
  collection a `for` can only be a template.
- **`named` comes first** and works out the member's name like text
  (owner: "(b)", over a `times_{'n'}` hole): `named "times_" ++
  to.stored.str.utf8['n'];`. A name that needs logic calls a function of the
  file, run while generating: `named size_name['n'];`, the function written
  with ordinary `if`s. A type is its name's text there (`"int16"`). `named`
  anywhere else is refused, and so is a template without it.
- **Then the member, with no name**: `[ … ] -> T { … }`, using the loop's
  value as `'n'` or `'T'`.

## The generated part

- **`synapse generate` writes it**, into the same file, right under the
  template (owner: "(b)", "(a)"), so a reader sees and can search every
  name.
- It is marked **`#generated_N_lines_below`**, N being the lines under the
  marker (owner: "`#generated_**21**_lines_below`"), a comment of the same
  family as `#3l`. `synapse generate` keeps N right.
- **The build only reads it**, and refuses a part that no longer matches
  its template, whether the template changed, the part was edited by
  hand, or it was never written: "the generated part under the template at
  line 9 does not match it: run `synapse generate`". So builds stay
  repeatable from the source alone, and a hand edit is never lost
  silently.
- In the oracle the command is `oracle --generate PATH`.

## Readings, not rulings

Picked by design claude while building; in the oracle's README until the
owner rules:

- The marker stands right under the template's closing `}`.
- In a generated member, the loop's value is written as it stands in the
  list, so a const stays its name (`times 'big'`), while `named` sees its
  value (`times_100`).

## `coll.handler` (owner, 2026-10-13; built in the oracle)

```
coll.handler cum_failed {
    cum_failed.flow_back_to_bladder {
        odd { return 0; }
        negative 'v' { report['v']; return 0; }
    }
}

let 'b' := half[-4] otherwise cum_failed.flow_back_to_bladder;
```

- A member is a **named `otherwise` block**: the arms written inline today,
  one per reason, a payload bound to a quoted name.
- `otherwise cum_failed.flow_back_to_bladder` uses it **as if written at
  the call** (design claude's reading, not objected to): its arms may
  `return`, `break`, `continue` and `fail` from where it is used.
- It is refused on a call it does not cover: every reason the call can
  fail with needs its arm, as inline.
- A payload is bound as inline, `negative 'v'`, its type the call's
  (design claude's reading: nothing new to write).
- As built, design claude's picks until the owner rules: a member is
  checked at each use, as the arms written there would be; a handler is
  used in its own file; `pub`, a `bare.` mark and templates are refused as
  not decided. `examples/collection_handlers.cy`.

