# A kit for uses_paths: other kits write its names with `greet::` (owner, 2026-10-22), and its operator `merge` bare
# or with the path, and its glyph `<+>` the same.

pub fn hello [] -> str.utf8 {
    return "hello";
}

pub fn wave [] -> str.utf8 {
    return "wave";
}

pub fn formal [] -> str.utf8 {
    return "good day";
}

pub struct Tally {
    n: i32
}

pub op merge ['a': Tally, 'b': Tally] -> Tally {
    return Tally[n = 'a'.n + 'b'.n];
}

pub struct Note {
    text: str.utf8
}

pub enum Tone {
    soft,
    loud
}

pub op <+> ['a': Tally, 'b': Tally] -> Tally {
    return Tally[n = 'a'.n + 'b'.n + 1];
}
