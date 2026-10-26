//! The limb mark, `%` (owner, 2026-10-26; design/cores.md "The limb mark";
//! oracle 8bc2967c): every piece a limb adds is marked on its first word, and
//! `%` no longer means percent. A rewrite of the rendered text, outside
//! strings, comments, character literals and names, after `calls`:
//!
//! - a limb's word standing alone (not after a letter, `.`, `::`, `'` or `$`):
//!   Money's `d32` `d64` `d128`, WSMW's `i128` `u128` `f128` `not_std`, AI's
//!   `brain16`, MCPS's `concur` `channel` `whichever` and `para` (after `for`),
//!   CNC's `extern` and `layout` (before `.C`) and `C` (before `.`);
//! - after `to.`, the limb's type (`to.%d64(…)`, `to.%brain16(…)`), and the
//!   unicode limb's units: `char.%point`, `char.%grapheme`, `str.%utf16`,
//!   `str.%utf32` (also after `to.`); after `incl.` and `inline.`, `C` and `MSL`;
//! - a percent is a word: `82.25%` is `82.25 percent`, `%[d32]` is
//!   `percent[%d32]`.
//!
//! Names, fields, labels and anything after a kit's `::` are never marked.
//! Cross-checked against `oracle --mark-limbs` (2026-10-26).

use crate::paths::Edits;

/// Limb words marked wherever they stand alone.
const ALONE: [&str; 12] = ["d32", "d64", "d128", "i128", "u128", "f128", "not_std", "brain16", "concur", "channel", "whichever", "C"];
/// The unicode limb's words after `char.` or `str.`.
const UNITS: [&str; 4] = ["point", "grapheme", "utf16", "utf32"];

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether the harness's own programs (`check clayout`, `check metal`) are
/// written marked, as the generator's are under `--limb-marks`.
pub static MARKED: AtomicBool = AtomicBool::new(false);

/// A program the harness wrote, marked when `MARKED` says so.
pub fn as_marked(text: String) -> String {
    if MARKED.load(Ordering::Relaxed) { mark_limbs(&text) } else { text }
}

pub fn mark_limbs(text: &str) -> String {
    mark_limbs_with(text).0
}

pub fn mark_limbs_with(text: &str) -> (String, Edits) {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len() + text.len() / 16);
    let mut edits: Edits = vec![];
    let (mut i, mut in_str, mut in_name, mut in_char) = (0, false, false, false);
    let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    while i < b.len() {
        let c = b[i];
        let ch = text[i..].chars().next().unwrap();
        if in_str || in_name || in_char {
            out.push(ch);
            if c == b'\\' && i + 1 < b.len() {
                let n = text[i + 1..].chars().next().unwrap();
                out.push(n);
                i += 1 + n.len_utf8();
                continue;
            }
            if (in_str && c == b'"') || (in_name && c == b'\'') || (in_char && c == b'`') {
                in_str = false;
                in_name = false;
                in_char = false;
            }
            i += ch.len_utf8();
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'\'' => in_name = true,
            b'`' => in_char = true,
            b'#' => {
                let end = text[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
                out.push_str(&text[i..end]);
                i = end;
                continue;
            }
            b'%' => {
                // a percent: `82.25%` is `82.25 percent`, `%[d32]` `percent[%d32]`
                let after_number = i > 0 && (b[i - 1].is_ascii_digit() || b[i - 1] == b')');
                if after_number {
                    out.push_str(" percent");
                    edits.push((i as isize, 7));
                } else if b.get(i + 1) == Some(&b'[') {
                    out.push_str("percent");
                    edits.push((i as isize, 6));
                } else {
                    out.push('%');
                }
                i += 1;
                continue;
            }
            _ => {}
        }
        if c.is_ascii_alphabetic() {
            let end = (i..b.len()).find(|&k| !word(b[k])).unwrap_or(b.len());
            let w = &text[i..end];
            let next = b.get(end).copied();
            let prev = if i > 0 { b[i - 1] } else { b' ' };
            let before = &text[..i];
            let alone = !(word(prev) || prev == b'.' || prev == b':' || prev == b'$');
            let mark = if alone {
                match w {
                    "extern" | "layout" => text[end..].starts_with(".C"),
                    "C" => next == Some(b'.'),
                    "para" => before.ends_with("for "),
                    _ => ALONE.contains(&w),
                }
            } else if prev == b'.' {
                // the word after `to.`, `incl.`, `inline.`, `char.` or `str.`
                let head = before.trim_end_matches('.');
                let last = head.rsplit(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')).next().unwrap_or("");
                match last {
                    "to" => ALONE.contains(&w) && w != "C",
                    "incl" | "inline" => w == "C" || w == "MSL",
                    l if l == "char" || l == "str" || l == "to.char" || l == "to.str" => UNITS.contains(&w),
                    _ => false,
                }
            } else {
                false
            };
            if mark {
                out.push('%');
                edits.push((i as isize, 1));
            }
            out.push_str(w);
            i = end;
            continue;
        }
        out.push(ch);
        i += ch.len_utf8();
    }
    (out, edits)
}

#[cfg(test)]
mod tests {
    use super::mark_limbs;

    #[test]
    fn marks() {
        let s = "extern.C fn w = C.printf [$f: ptr[u8]] -> C.int;\nlayout.C struct P { a: u32 }\nlet $d: d64 = to.d64($x) otherwise 1.5;\nlet $p: %[d32] = exactly 7%;\nlet $l: std::list[d64] = [];\nconcur with [lend $c] { channel.send(lend $c, value: 1); }\nfor para $i: u64 [from: 1, to: 3, by: 1] { }\nlet $t := to.str.utf32($s);\nlet $c := char.point.from_code(65);\nno_check inline.MSL threads 4 { } with [];\nlet $n: str.utf8 = \"d64 50%\";\nlet $q: i64 = $d64 + 'C';";
        let t = mark_limbs(s);
        assert_eq!(t, "%extern.C fn w = %C.printf [$f: ptr[u8]] -> %C.int;\n%layout.C struct P { a: u32 }\nlet $d: %d64 = to.%d64($x) otherwise 1.5;\nlet $p: percent[%d32] = exactly 7 percent;\nlet $l: std::list[%d64] = [];\n%concur with [lend $c] { %channel.send(lend $c, value: 1); }\nfor %para $i: u64 [from: 1, to: 3, by: 1] { }\nlet $t := to.str.%utf32($s);\nlet $c := char.%point.from_code(65);\nno_check inline.%MSL threads 4 { } with [];\nlet $n: str.utf8 = \"d64 50%\";\nlet $q: i64 = $d64 + 'C';");
    }
}
