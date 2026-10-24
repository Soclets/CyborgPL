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
    rewrite(text, false, false, &mut vec![])
}

/// An edit a rewrite made: the characters after offset `.0` of the text it
/// was given (all of them for -1) moved by `.1`. Spans are carried through.
pub type Edits = Vec<(isize, isize)>;

/// Where offset `p` of the text before a rewrite stands after it.
pub fn map_offset(edits: &Edits, p: usize) -> usize {
    (p as isize + edits.iter().filter(|(at, _)| *at < p as isize).map(|(_, d)| d).sum::<isize>()) as usize
}

/// A rendered program's text with std's paths in the form the program was
/// written for (`use_list`, `glyph`), and the edits made.
pub fn std_paths_with(text: &str, use_list: bool, glyph: bool) -> (String, Edits) {
    let mut edits = vec![];
    let out = rewrite(text, use_list, glyph, &mut edits);
    if use_list {
        let prefix = "use std::list as list; ";
        edits.insert(0, (-1, prefix.len() as isize));
        return (format!("{prefix}{out}"), edits);
    }
    (out, edits)
}

/// The same, with std's glyph `xx` written `std::xx` every other time too
/// (design claude, 2026-10-22; oracle edcbf2cb).
pub fn std_paths_glyph(text: &str) -> String {
    rewrite(text, false, true, &mut vec![])
}

/// The same, with `use std::list as list;` (owner, 2026-10-22; oracle
/// 82d88048, kayo a42b37c4) put in front of the first line, a comment, so no
/// line moves, and `list` then written bare every other time.
pub fn std_paths_using_list(text: &str) -> String {
    format!("use std::list as list; {}", rewrite(text, true, false, &mut vec![]))
}

fn rewrite(text: &str, use_list: bool, glyph: bool, edits: &mut Edits) -> String {
    let (mut lists, mut pows, mut glyphs) = (0, 0, 0);
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
            let bare_list = use_list && w == "list" && {
                lists += 1;
                lists % 2 == 0
            };
            // std's operator `pow` (and `pow.wrap`) between values: its path is
            // optional (owner, 2026-10-22), so written every other time
            let pow_path = w == "pow" && i > 0 && b[i - 1] == b' ' && matches!(next, Some(b' ') | Some(b'.')) && {
                pows += 1;
                pows % 2 == 0
            };
            // the glyph `xx` between values, with its path every other time
            let glyph_path = glyph && w == "xx" && i > 0 && b[i - 1] == b' ' && next == Some(b' ') && {
                glyphs += 1;
                glyphs % 2 == 0
            };
            if (hit && !bare_list) || pow_path || glyph_path {
                out.push_str("std::");
                edits.push((i as isize, 5));
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
    use super::{std_paths, std_paths_using_list};

    #[test]
    fn pow_every_other_time() {
        let s = "('a' pow 2) + ('b' pow.wrap 3) + ('c' pow 4)";
        assert_eq!(std_paths(s), "('a' pow 2) + ('b' std::pow.wrap 3) + ('c' pow 4)");
    }

    #[test]
    fn glyph_every_other_time() {
        let s = "('a' xx 2) + ('b' xx 3) + ('c' x 4)";
        assert_eq!(super::std_paths_glyph(s), "('a' xx 2) + ('b' std::xx 3) + ('c' x 4)");
        assert_eq!(std_paths(s), s);
    }

    #[test]
    fn use_keeps_lines() {
        let s = "# seed 1\nlet 'a': list[u8] = [];\nlist.append[lend.mut 'a', value: 1];\n";
        assert_eq!(std_paths_using_list(s), "use std::list as list; # seed 1\nlet 'a': std::list[u8] = [];\nlist.append[lend.mut 'a', value: 1];\n");
    }

    #[test]
    fn rewrites_roots_only() {
        let s = "let mut 'ds': list[u8] = [];\n    list.append[lend.mut 'ds', value: 1]; # list[x]\n    std::print.useful[\"list[1]\", 'list', round.near[2.5, places: 0], to.stored.str.utf8[3], str.utf8[max: 4]];";
        let t = std_paths(s);
        assert_eq!(t, "let mut 'ds': std::list[u8] = [];\n    std::list.append[lend.mut 'ds', value: 1]; # list[x]\n    std::print.useful[\"list[1]\", 'list', std::round.near[2.5, places: 0], to.stored.str.utf8[3], str.utf8[max: 4]];");
    }
}
