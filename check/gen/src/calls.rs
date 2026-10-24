//! Calls take `( )`, definitions keep `[ ]`, variables may be `$name`
//! (owner, 2026-10-22; oracle c16dc2d7). A rewrite of the rendered text,
//! outside strings, comments and character literals, after `paths`:
//!
//! - a `[` right after a name (no space) opens a call, written `( … )`,
//!   unless the name is a type (`i32[min: 1]`, `std::list[u8]`), describes
//!   (`size.of[T]`, `align.of[T]`, `ptr.cast[T]`, `simd[…]`, `range[…]`,
//!   `incl.…`, `optional[T]`, a result's `or fail[…]`), builds a struct (a capitalised name, `P[x = 1]`), or follows
//!   `fail` (`fail r[$v]`);
//! - after a quoted or `$` name, a `[` reads a position (`$xs[pos: 2]`) or
//!   a slice (`from: pos 2`), otherwise it calls a function value;
//! - a `to.T` or `ptr.cast` whose group is followed at once by another `[`
//!   is described by the first (`to.i32[min: 1, max: 99]`, `to.percent[d64]`,
//!   `ptr.cast[T]`), and the second is the call;
//! - everything after a space (`fn f [$n: i64]`, a preset `['k' = 1]`,
//!   `labels [ … ]`, `for … [from: …]`), a list, and a group after `]` or `)`
//!   that reads no position stay as they are.
//!
//! Lines stay where they were; a column moves where `'x'` becomes `$x`.

const TYPES: [&str; 27] = [
    "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f16", "f32", "f64", "d32", "d64", "d128", "bool", "brain16", "str.utf8", "list", "map", "simd", "ptr", "ptr.mut", "percent", "channel", "fn",
];
const DESCRIBE: [&str; 5] = ["size.of", "align.of", "ptr.cast", "range", "simd"];

#[derive(Clone, Copy, PartialEq)]
enum Open {
    Call,
    Keep,
    /// keeps `[ ]` and makes the group right after it a call
    Describes,
}

fn is_type(path: &str) -> bool {
    let p = path.strip_prefix("std::").unwrap_or(path);
    TYPES.contains(&p)
}

/// What a `[` at `i` opens, given the text before it.
fn opens(text: &str, i: usize, last_closed: Option<Open>) -> Open {
    let b = text.as_bytes();
    let path_char = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b':';
    let inside_starts = |w: &str| text[i + 1..].trim_start().starts_with(w);
    let position = inside_starts("pos:") || inside_starts("from:");
    if i == 0 {
        return Open::Keep;
    }
    let p = b[i - 1];
    if p == b']' || p == b')' {
        return match last_closed {
            Some(Open::Describes) if !position => Open::Call,
            _ => Open::Keep,
        };
    }
    if p == b'\'' {
        return if position { Open::Keep } else { Open::Call };
    }
    if !path_char(p) {
        return Open::Keep;
    }
    let start = (0..i).rev().take_while(|&k| path_char(b[k])).last().unwrap_or(i);
    let path = &text[start..i];
    if start > 0 && b[start - 1] == b'$' {
        return if position { Open::Keep } else { Open::Call };
    }
    if path.is_empty() || path.as_bytes()[0].is_ascii_digit() {
        return Open::Keep;
    }
    let last = path.rsplit(|c| c == '.' || c == ':').next().unwrap_or(path);
    // `to.T[…]` is a call (`to.E1(85)` too), unless its group is followed at
    // once by another, which it then describes
    let p2 = path.strip_prefix("std::").unwrap_or(path);
    if p2.starts_with("to.") {
        let close = matching(text, i);
        return if close.is_some_and(|c| b.get(c + 1) == Some(&b'[')) { Open::Describes } else { Open::Call };
    }
    // a capitalised last part builds a struct (`P[x = 1]`); a type's own
    // function (`Counter.next`) is called like any other
    if last.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return Open::Keep;
    }
    if text[..start].ends_with("fail ") {
        return Open::Keep;
    }
    // `optional[i32]` and `or fail[odd, negative: i32]` describe a result
    if is_type(path) || path.starts_with("incl.") || path == "optional" || path == "fail" {
        return Open::Keep;
    }
    if DESCRIBE.contains(&p2) {
        return if p2 == "ptr.cast" { Open::Describes } else { Open::Keep };
    }
    Open::Call
}

/// The `]` that closes the `[` at `i`, skipping strings and nested groups.
fn matching(text: &str, i: usize) -> Option<usize> {
    let b = text.as_bytes();
    let (mut depth, mut k, mut in_str) = (0usize, i, false);
    while k < b.len() {
        let c = b[k];
        if in_str {
            if c == b'\\' {
                k += 2;
                continue;
            }
            in_str = c != b'"';
        } else if c == b'"' {
            in_str = true;
        } else if c == b'[' {
            depth += 1;
        } else if c == b']' {
            depth -= 1;
            if depth == 0 {
                return Some(k);
            }
        }
        k += 1;
    }
    None
}

pub fn paren_calls(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut stack: Vec<Open> = vec![];
    let mut last_closed: Option<Open> = None;
    let (mut i, mut in_str, mut in_name, mut in_char) = (0, false, false, false);
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
            b'[' => {
                let o = opens(text, i, last_closed);
                stack.push(o);
                out.push(if o == Open::Call { '(' } else { '[' });
                last_closed = None;
                i += 1;
                continue;
            }
            b']' => {
                let o = stack.pop().unwrap_or(Open::Keep);
                out.push(if o == Open::Call { ')' } else { ']' });
                last_closed = Some(o);
                i += 1;
                continue;
            }
            _ => {}
        }
        if !c.is_ascii_whitespace() {
            last_closed = None;
        }
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// `'name'` written `$name` every other time where the name needs no
/// quotes (letters, digits, `_`): `$x` and `'x'` are one variable.
pub fn dollars(text: &str) -> String {
    dollars_with(text).0
}

/// The same, with the edits made: each `'x'` written `$x` drops its closing
/// quote.
pub fn dollars_with(text: &str) -> (String, crate::paths::Edits) {
    let mut edits = vec![];
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let (mut i, mut in_str, mut in_char, mut n) = (0, false, false, 0);
    while i < b.len() {
        let c = b[i];
        let ch = text[i..].chars().next().unwrap();
        if in_str || in_char {
            out.push(ch);
            if c == b'\\' && i + 1 < b.len() {
                let x = text[i + 1..].chars().next().unwrap();
                out.push(x);
                i += 1 + x.len_utf8();
                continue;
            }
            if (in_str && c == b'"') || (in_char && c == b'`') {
                in_str = false;
                in_char = false;
            }
            i += ch.len_utf8();
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'`' => in_char = true,
            b'#' => {
                let end = text[i..].find('\n').map(|k| i + k).unwrap_or(b.len());
                out.push_str(&text[i..end]);
                i = end;
                continue;
            }
            b'\'' => {
                let end = text[i + 1..].find('\'').map(|k| i + 1 + k);
                if let Some(e) = end {
                    let name = &text[i + 1..e];
                    let plain = !name.is_empty() && name.bytes().all(|x| x.is_ascii_alphanumeric() || x == b'_') && !name.as_bytes()[0].is_ascii_digit();
                    if plain {
                        n += 1;
                        if n % 2 == 0 {
                            out.push('$');
                            out.push_str(name);
                            edits.push((e as isize, -1));
                        } else {
                            out.push_str(&text[i..=e]);
                        }
                    } else {
                        out.push_str(&text[i..=e]);
                    }
                    i = e + 1;
                    continue;
                }
            }
            _ => {}
        }
        out.push(ch);
        i += ch.len_utf8();
    }
    (out, edits)
}

#[cfg(test)]
mod tests {
    use super::{dollars, paren_calls};

    #[test]
    fn calls_and_what_stays() {
        let s = "fn f ['n': i64] -> i64 { return 'n'; }\nMAIN {\n    let mut 'b': std::list[u8] = [];\n    std::list.append[lend.mut 'b', value: to.u8[48 plus f['n']]];\n    let 'x': i32[min: 0, max: 9] = to.i32[min: 0, max: 9]['t'] otherwise 0;\n    std::print.useful['b'[pos: 1], size.of[L], L[a = 1], to.percent[d64][0.07], newln];\n    fail r['v'];\n}";
        let t = paren_calls(s);
        assert_eq!(t, "fn f ['n': i64] -> i64 { return 'n'; }\nMAIN {\n    let mut 'b': std::list[u8] = [];\n    std::list.append(lend.mut 'b', value: to.u8(48 plus f('n')));\n    let 'x': i32[min: 0, max: 9] = to.i32[min: 0, max: 9]('t') otherwise 0;\n    std::print.useful('b'[pos: 1], size.of[L], L[a = 1], to.percent[d64](0.07), newln);\n    fail r['v'];\n}");
    }

    #[test]
    fn dollars_every_other_plain_name() {
        assert_eq!(dollars("'a' 'b' 'r3-' 'c' \"'d'\""), "'a' $b 'r3-' 'c' \"'d'\"");
    }
}
