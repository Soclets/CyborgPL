# Licensing

Owner's intent, decided 2026-09-14. The licence texts themselves are for
counsel to draft; this records what they must achieve.

## Intent

1. **Using the language is free, with no credit required anywhere.** Writing
   CyborgPL, compiling it, shipping the binary: nothing owed, nothing to
   mention. The code that lands in a user's binary — the runtime and the
   standard Limb — must carry no attribution obligation for compiled output.
2. **Modifying the compiler is free, with credit everywhere** — in the
   source *and* wherever the user of the modified compiler can see: its
   version output, its documentation, its website, its distribution page.
   A fork must say it is derived from CyborgPL in every place a person would
   look.

## Proposed mapping (Claude, for counsel)

| Piece | Licence | Effect |
|---|---|---|
| Runtime, standard Limb | Apache-2.0 WITH LLVM-exception | The exception waives notice requirements for compiled binaries — users owe nothing. Same licence LLVM uses and Xag used. |
| Compiler, Precog, JetPack | Apache-2.0 **plus a CyborgPL attribution term** | Apache alone requires credit in source (licence, copyright, `NOTICE`, marking of changed files). The additional term extends it to user-visible places. |
| Spec, docs, decision log | CC BY 4.0 | Anyone may implement the language; the text is credited. |
| Names — CyborgPL, Precog, Limb, JetPack | Trademark (separate from copyright) | A fork credits CyborgPL but may not call itself CyborgPL. The stronger tool for "credit everywhere". |

## Known consequences of the attribution term (for counsel to weigh)

- The compiler's licence becomes non-standard ("Apache-2.0 plus terms"),
  which some corporate legal departments reject on sight. It will not be
  OSI-approved as-is.
- It resembles the old BSD advertising clause, dropped industry-wide for
  licence-compatibility reasons. For a single compiler this bites less than
  for a library.
- Neither consequence affects *users* of the language, who are under the
  runtime licence only.

## Flags for counsel

- **JetPack** is already a software trademark twice: Google's Android
  Jetpack and Automattic's Jetpack (WordPress). Check before the name is
  public.
- **Precog** is used by unrelated companies (an AI platform; a
  vulnerability scanner). Lower risk, still worth a look.
- Dependencies (LLVM: Apache-2.0 + LLVM exception; Z3: MIT; Rust crates:
  MIT/Apache) impose nothing that conflicts with any of the above.
