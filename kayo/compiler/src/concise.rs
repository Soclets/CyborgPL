//! Concise CyborgPL (owner, 2026-10-22, "as a joke"; `design/concise.md`): a
//! file holding only `0`, `1`, spaces and line breaks is the bytes of a
//! CyborgPL file written in binary. Each group of eight digits is one byte,
//! the spaces between groups only separate them, and a line break stays a
//! line break; the decoded text is then read as any file. `.cy` and `.cyr`
//! alike; a file `incl.bytes` brings in is data and never decoded.

/// What a file's text is.
pub enum Read {
    /// An ordinary file.
    Plain,
    /// A concise file, decoded.
    Decoded(String),
    /// A concise file that cannot be decoded: the line (1-based) and why.
    Bad(u32, String),
}

pub fn read(text: &str) -> Read {
    let only = |extra: &[char]| text.chars().all(|c| matches!(c, '0' | '1' | ' ' | '\t' | '\n' | '\r') || extra.contains(&c));
    let has_digit = text.contains(['0', '1']);
    if !has_digit {
        return Read::Plain;
    }
    // A plain brace among binary groups (design claude's reading): their
    // binary spellings.
    if !only(&[]) {
        if only(&['{', '}']) && text.split_whitespace().any(|g| g.len() == 8 && g.chars().all(|c| c == '0' || c == '1')) {
            let line = text.lines().position(|l| l.contains(['{', '}'])).unwrap_or(0) as u32 + 1;
            return Read::Bad(line, "a concise file is only `0`, `1`, spaces and line breaks: `{` is `01111011` and `}` is `01111101`".into());
        }
        return Read::Plain;
    }
    let mut bytes = Vec::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            bytes.push(b'\n');
        }
        for g in line.split_whitespace() {
            if g.len() != 8 {
                return Read::Bad(i as u32 + 1, format!("`{}` is {} digits; each group is 8, one byte", g, g.len()));
            }
            bytes.push(u8::from_str_radix(g, 2).unwrap_or(0));
        }
    }
    match String::from_utf8(bytes) {
        Ok(s) => Read::Decoded(s),
        Err(_) => Read::Bad(1, "the bytes of this concise file are not UTF-8 text".into()),
    }
}
