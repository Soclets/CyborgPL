# Concise CyborgPL (a joke)

The owner, 2026-10-22: "There should be a way to write concise CyborgPL", for "people tired of writing long CyborgPL
code"; "Should be added as a joke". It is the bytes of a CyborgPL file written in binary.

```
01001101 01000001 01001001 01001110 00100000 01111011
00100000 00100000 00100000 00100000 01110011 01110100 01100100 …
01111101
```

is `MAIN {`, `    std::print.useful["Hello, World!", newln];`, `}` (`examples/concise.cy`).

- **What makes a file concise:** it holds only `0`, `1`, spaces and line breaks (owner: "Only binary?"). No marker,
  no file ending: a real program always holds a letter, so none is taken for one.
- **How it is read:** each group of eight digits is one byte; the spaces between groups only separate them (a real
  space is `00100000`); a line break stays a line break. The decoded text is then read as any file, so a concise
  file can say anything CyborgPL can (a UTF-8 character is its bytes: `é` is two groups).
- **`.cy` and `.cyr` alike:** the manifest `CyJet.cyr` may be concise too (owner: "A manifest can be concise too";
  `examples/concise_kit`). A file brought in by `incl.bytes` or `incl.str.utf8` is data, never decoded.
- **The final build ships it** (owner: "it's not dangerous, just practically un-maintainable").
- Design claude's readings, not asked: an error names the line of the decoded text and shows that line, since the
  binary file's line and column mean nothing; a group not eight digits long is refused naming its line; a plain `{`
  or `}` in an otherwise binary file is refused with their binary spellings (`01111011`, `01111101`); the build
  report holds the decoded text (kayo's). Reject tests `concise_decoded_error`, `concise_short_group`,
  `concise_plain_braces`.
