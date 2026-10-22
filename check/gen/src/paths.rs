//! std's names with their path (owner, 2026-10-22; oracle 82d88048):
//! another kit's names always take it, std's too, so `list[i64]` is written
//! `std::list[i64]` and `round.near[…]` `std::round.near[…]`. A rewrite of
//! the rendered text: outside strings, comments and quoted names, a root of
//! std's standing alone (not after `.`, `::` or a letter) and followed by `[`
//! or `.`. Lines stay where they were; columns after a rewrite move.

/// std's roots the generator writes bare. `str` only for the text
/// operations: `str.utf8` and the empty text `str.empty` are the base's.
const ROOTS: [&str; 7] = ["list", "map", "round", "format", "duration", "casing", "str"];

pub fn std_paths(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len() + text.len() / 8);
    let (mut i, mut in_str, mut in_name) = (0, false, false);
    let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    while i < b.len() {
        let c = b[i];
        if in_str {
            out.push(c as char);
            if c == b'\\' && i + 1 < b.len() {
                out.push(b[i + 1] as char);
                i += 2;
                continue;
            }
            in_str = c != b'"';
            i += 1;
            continue;
        }
        if in_name {
            out.push(c as char);
            if c == b'\\' && i + 1 < b.len() {
                out.push(b[i + 1] as char);
                i += 2;
                continue;
            }
            in_name = c != b'\'';
            i += 1;
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'\'' => in_name = true,
            b'#' => {
                let end = text[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
                out.push_str(&text[i..end]);
                i = end;
                continue;
            }
            _ => {}
        }
        let alone = i == 0 || !(word(b[i - 1]) || b[i - 1] == b'.' || b[i - 1] == b':');
        if alone && c.is_ascii_alphabetic() {
            let end = (i..b.len()).find(|&k| !word(b[k])).unwrap_or(b.len());
            let w = &text[i..end];
            let next = b.get(end).copied();
            let rest = &text[end..];
            let hit = ROOTS.contains(&w) && matches!(next, Some(b'[') | Some(b'.')) && !(w == "str" && (rest.starts_with(".utf8") || rest.starts_with(".empty")));
            if hit {
                out.push_str("std::");
            }
            out.push_str(w);
            i = end;
            continue;
        }
        // text is ASCII outside strings and names, but keep any other byte whole
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::std_paths;

    #[test]
    fn rewrites_roots_only() {
        let s = "let mut 'ds': list[u8] = [];\n    list.append[lend.mut 'ds', value: 1]; # list[x]\n    std::print.useful[\"list[1]\", 'list', round.near[2.5, places: 0], to.stored.str.utf8[3], str.utf8[max: 4]];";
        let t = std_paths(s);
        assert_eq!(t, "let mut 'ds': std::list[u8] = [];\n    std::list.append[lend.mut 'ds', value: 1]; # list[x]\n    std::print.useful[\"list[1]\", 'list', std::round.near[2.5, places: 0], to.stored.str.utf8[3], str.utf8[max: 4]];");
    }
}
