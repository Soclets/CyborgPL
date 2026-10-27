# A kit for uses_paths: other kits write its names with `greet::` (owner, 2026-10-22), and its operator `merge` bare
# or with the path, and its glyph `<+>` the same.

pub fn hello [] -> str.utf8:
    ret "hello";
end of fn hello

pub fn wave [] -> str.utf8:
    ret "wave";
end of fn wave

pub fn formal [] -> str.utf8:
    ret "good day";
end of fn formal

pub rec Tally:
    n: int.sign.32;
end of rec Tally

pub op merge [$a: Tally, $b: Tally] -> Tally:
    ret Tally[n = $a.n plus $b.n];
end of op merge

pub rec Note:
    text: str.utf8;
end of rec Note

pub union.tag Tone:
    soft;
    loud;
end of union.tag Tone

pub op <+> [$a: Tally, $b: Tally] -> Tally:
    ret Tally[n = $a.n plus $b.n plus 1];
end of op <+>
