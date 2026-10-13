# Collections

Decided 2026-10-13 (owner), built the same day in the oracle and Synapse.
The rulings, with the owner's words, are in `DECISIONS.md` under 2026-10-13;
this note says what they add up to. The example is `examples/collections.nrv`.

A collection writes a function and its dot narrowings (`cum`, `cum.quick`,
`cum.double`) in one block, instead of a `fn cum` and a `fn cum.quick` and a
`fn cum.double` each with its own `pub` and `bare.` mark.

## The shape

```
pub collection cum {
    bare.never;
    cum ['x': int32, 'step': int32] -> int32 { return 'x' plus 'step'; }
    quick ['step' = 2];
    double ['x': int32] -> int32 { return 'x' times 2; }
}
```

- `collection`, then the name, bare as a function's is (`cum`, never
  `'cum'`), then `{ }`.
- **The own function** carries the collection's name: `cum [ … ] -> T
  { … }`, called `cum[…]`, or `name::cum[…]` from another Neuron. ("Own
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

## Who sees it

- A collection is **private unless `pub collection`**, as everything else
  is (owner: "(a)").
- A `pub` collection makes every member public, under **one `bare.` mark**
  for them all, written in the block: `bare.never;` (first, by habit, as in
  the sketch; the oracle takes it anywhere in the block). Two marks are
  refused. The mark is the usual one (`ownership-syntax.md`, 2026-10-06):
  `bare.never`, `bare.optional` or `bare.only`, with no default.

## A collection with no own function

Allowed (owner, 2026-10-13: "(a)"). The name then only groups its members,
as `cord::print` does: `cord::print.useful[…]` and `cord::print.stored[…]`
exist, a bare `cord::print[…]` does not.

```
collection zero {
    of_int16 [] -> int16 { return 0; }
    of_int64 [] -> int64 { return 0; }
}
```

`zero.of_int16[]` exists, `zero[…]` does not. Such a collection has no
presets, since a preset fills the own function.

## Templates

A template makes members whose names follow a pattern, for what a preset
cannot do (Good practice 14, `style.md`): where the value changes a
member's signature, or no own function takes it.

```
collection zero {
    for 'T' in [int16, int64] {
        named "of_" ++ 'T';
        [] -> 'T' { return 0; }
    }
    #generated_2_lines_below
    of_int16 [] -> int16 { return 0; }
    of_int64 [] -> int64 { return 0; }
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
