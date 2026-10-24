//! Concise CyborgPL, a joke (owner, 2026-10-22: "There should be a way to write concise CyborgPL", then "Only
//! binary?"): a file holding only `0`, `1`, spaces and line breaks is the bytes of a CyborgPL file, each group of
//! eight digits one byte. The spaces between groups only separate them (a real space is `00100000`); a line break
//! stays a line break. The decoded text is then read as any file. A real program always holds a letter, so none is
//! taken for one. A `.cyr` file may be concise too (owner: "A manifest can be concise too").

/// The CyborgPL text a concise file spells; `None` when the file is not concise.
pub fn decode(src: &str) -> Option<Result<String, String>> {
    if !src.contains(['0', '1']) || !src.chars().all(|c| matches!(c, '0' | '1' | ' ' | '\n' | '\r' | '{' | '}')) {
        return None;
    }
    // only binary (owner: "Only binary?"): a plain brace in an otherwise concise file is the likeliest slip
    if src.contains(['{', '}']) {
        return Some(Err("a concise file is only `0`, `1`, spaces and line breaks: write `{` as `01111011` and `}` as `01111101`".into()));
    }
    let mut bytes = Vec::new();
    for (n, line) in src.split('\n').enumerate() {
        if n > 0 { bytes.push(b'\n'); }
        for group in line.trim_end_matches('\r').split(' ').filter(|g| !g.is_empty()) {
            if group.len() != 8 {
                return Some(Err(format!("line {} of this concise file: `{group}` is {} digits; each group is 8, one byte", n + 1, group.len())));
            }
            bytes.push(u8::from_str_radix(group, 2).expect("eight binary digits"));
        }
    }
    Some(String::from_utf8(bytes).map_err(|_| "this concise file's bytes are not UTF-8 text".to_string()))
}

/// A message about line `line` of decoded text, with that line shown: in the binary file the line and column
/// mean nothing.
pub fn shown(text: &str, line: usize) -> String {
    match text.lines().nth(line.saturating_sub(1)) {
        Some(l) => format!(" (concise file; decoded line {line}: `{}`)", l.trim()),
        None => " (concise file)".to_string(),
    }
}

/// Text spelled as a concise file: each byte eight digits, a space between, a line break kept
pub fn encode(text: &str) -> String {
    text.split('\n').map(|l| l.bytes().map(|b| format!("{b:08b}")).collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join("\n")
}
