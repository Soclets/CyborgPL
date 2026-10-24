//! `std::print.format`'s format text (owner, 2026-09-22; `{}` holes since
//! 2026-10-01; `examples/print_format.cy`): `{}` takes the next value in
//! its readable form, `{1}`, `{2}`… a numbered one (from 1, never mixed with
//! `{}`); after `:` a shape: fill and alignment `< ^ >`, `+`, `0` padding, a
//! width, `.places`, and a kind — `x` `X` `o` `b` (bases), `e` (scientific),
//! `?` (the stored form). The text has its own escapes, read in one pass:
//! `\n` `\t` `\\` `\"` `\{` `\}`; `%` is plain text. Read when building, by
//! the checker and again by the code it makes.

/// One part of a format text.
#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    Text(String),
    /// Which value (from 0), and its shape.
    Hole(usize, Spec),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Spec {
    /// The fill character, `' '` unless written.
    pub fill: Option<char>,
    /// `<`, `^` or `>`.
    pub align: Option<char>,
    pub plus: bool,
    pub zero: bool,
    pub width: Option<u32>,
    pub places: Option<u32>,
    /// `x` `X` `o` `b` `e` `?`.
    pub kind: Option<char>,
}

/// The pieces of a format text, or why it is refused (the oracle's words).
pub fn parse(text: &str) -> Result<Vec<Piece>, String> {
    let mut out: Vec<Piece> = Vec::new();
    let mut lit = String::new();
    let mut next = 0usize;
    let (mut plain, mut numbered) = (false, false);
    let mut cs = text.chars().peekable();
    while let Some(c) = cs.next() {
        match c {
            '\\' => match cs.next() {
                Some('n') => lit.push('\n'),
                Some('t') => lit.push('\t'),
                Some('\\') => lit.push('\\'),
                Some('"') => lit.push('"'),
                Some('{') => lit.push('{'),
                Some('}') => lit.push('}'),
                Some(o) => return Err(format!("`\\{}` is not an escape in a format text: `\\n`, `\\t`, `\\\\`, `\\\"`, `\\{{` and `\\}}` are", o)),
                None => return Err("a format text ends in a lone `\\`".into()),
            },
            '}' => return Err("a lone `}` in a format text: a literal brace is `\\}`".into()),
            '{' => {
                let mut inside = String::new();
                loop {
                    match cs.next() {
                        Some('}') => break,
                        Some(ch) => inside.push(ch),
                        None => return Err("a `{` in a format text is never closed: a literal brace is `\\{`".into()),
                    }
                }
                let (index, shape) = match inside.split_once(':') {
                    Some((i, s)) => (i, Some(s)),
                    None => (inside.as_str(), None),
                };
                let which = if index.is_empty() {
                    plain = true;
                    next += 1;
                    next - 1
                } else {
                    let n: usize = index.parse().map_err(|_| format!("`{{{}}}` is not a hole: `{{}}`, `{{1}}`, or `{{:shape}}`", inside))?;
                    if n == 0 {
                        return Err("holes count from `{1}`, the first value".into());
                    }
                    numbered = true;
                    n - 1
                };
                if plain && numbered {
                    return Err("this format text mixes `{}` with numbered holes: use one or the other".into());
                }
                if !lit.is_empty() {
                    out.push(Piece::Text(std::mem::take(&mut lit)));
                }
                let spec = match shape {
                    Some(s) => spec(s)?,
                    None => Spec::default(),
                };
                out.push(Piece::Hole(which, spec));
            }
            other => lit.push(other),
        }
    }
    if !lit.is_empty() {
        out.push(Piece::Text(lit));
    }
    Ok(out)
}

fn spec(s: &str) -> Result<Spec, String> {
    let mut sp = Spec::default();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    let is_align = |c: char| matches!(c, '<' | '^' | '>');
    if cs.len() >= 2 && is_align(cs[1]) {
        sp.fill = Some(cs[0]);
        sp.align = Some(cs[1]);
        i = 2;
    } else if !cs.is_empty() && is_align(cs[0]) {
        sp.align = Some(cs[0]);
        i = 1;
    }
    if cs.get(i) == Some(&'+') {
        sp.plus = true;
        i += 1;
    }
    if cs.get(i) == Some(&'#') {
        return Err("`#` is not a shape: type the prefix in the text, `0x{:x}`".into());
    }
    if cs.get(i) == Some(&'0') {
        sp.zero = true;
        i += 1;
    }
    let digits = |i: &mut usize| -> Option<u32> {
        let start = *i;
        while cs.get(*i).is_some_and(|c| c.is_ascii_digit()) {
            *i += 1;
        }
        (start != *i).then(|| cs[start..*i].iter().collect::<String>().parse().ok()).flatten()
    };
    sp.width = digits(&mut i);
    if cs.get(i) == Some(&'.') {
        i += 1;
        sp.places = Some(digits(&mut i).ok_or_else(|| format!("`{{:{}}}`: `.` is followed by the places, `{{:.2}}`", s))?);
    }
    if let Some(&k) = cs.get(i) {
        if !matches!(k, 'x' | 'X' | 'o' | 'b' | 'e' | '?') {
            return Err(format!("`{}` is not a kind in a format's shape: `x`, `X`, `o`, `b`, `e` or `?`", k));
        }
        sp.kind = Some(k);
        i += 1;
    }
    if i != cs.len() {
        return Err(format!("`{{:{}}}` is not a shape: fill and `< ^ >`, `+`, `0`, a width, `.places`, then a kind", s));
    }
    Ok(sp)
}

/// How many values the pieces ask for, and whether a numbered one is never
/// used: the count of holes against the values given.
pub fn wanted(pieces: &[Piece]) -> (usize, Vec<usize>) {
    let holes: Vec<usize> = pieces.iter().filter_map(|p| match p {
        Piece::Hole(n, _) => Some(*n),
        _ => None,
    }).collect();
    let most = holes.iter().map(|n| n + 1).max().unwrap_or(0);
    let unused = (0..most).filter(|n| !holes.contains(n)).collect();
    (most, unused)
}

/// The text as it should be written, for a refusal `parse` gives: `\%`
/// written `%`, holes counted from `{1}`, a lone brace escaped, `#` taken
/// out of a shape. `None` when no one repair makes it read (mixed holes:
/// which number each `{}` was meant to be is the writer's).
pub fn repair(text: &str) -> Option<String> {
    let mut t = text.to_string();
    for _ in 0..8 {
        let Err(why) = parse(&t) else { return (t != text).then_some(t) };
        let next = if why.starts_with("`\\%`") {
            t.replace("\\%", "%")
        } else if why.starts_with("holes count from") {
            renumber(&t)?
        } else if why.starts_with("a lone `}`") {
            escape_lone(&t, '}')
        } else if why.starts_with("a `{` in a format text is never closed") {
            escape_lone(&t, '{')
        } else if why.starts_with("`#` is not a shape") {
            drop_hash(&t)
        } else {
            return None;
        };
        if next == t {
            return None;
        }
        t = next;
    }
    None
}

/// Each numbered hole one higher: `{0}` was meant as the first value.
fn renumber(t: &str) -> Option<String> {
    let mut out = String::new();
    let mut cs = t.char_indices().peekable();
    while let Some((_, c)) = cs.next() {
        out.push(c);
        if c == '\\' {
            if let Some((_, n)) = cs.next() {
                out.push(n);
            }
            continue;
        }
        if c == '{' {
            let mut digits = String::new();
            while let Some(&(_, d)) = cs.peek() {
                if !d.is_ascii_digit() {
                    break;
                }
                digits.push(d);
                cs.next();
            }
            if !digits.is_empty() {
                out.push_str(&(digits.parse::<usize>().ok()? + 1).to_string());
            }
        }
    }
    Some(out)
}

/// The first unpaired brace of this kind written as `\{` / `\}`.
fn escape_lone(t: &str, brace: char) -> String {
    let mut depth = 0i32;
    let mut escaped = false;
    for (i, c) in t.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            '}' if brace == '}' => return format!("{}\\}}{}", &t[..i], &t[i + 1..]),
            _ => {}
        }
    }
    if brace == '{' {
        // The last `{` that never closes.
        if let Some(i) = t.rfind('{') {
            return format!("{}\\{{{}", &t[..i], &t[i + 1..]);
        }
    }
    t.to_string()
}

/// `#` out of a hole's shape: the prefix is typed in the text.
fn drop_hash(t: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in t.chars() {
        match c {
            '{' => inside = true,
            '}' => inside = false,
            '#' if inside => continue,
            _ => {}
        }
        out.push(c);
    }
    out
}

/// The text with the kind taken off hole number `which` (from 0): a kind
/// that does not fit the value it shows.
pub fn without_kind(t: &str, which: usize) -> Option<String> {
    let pieces = parse(t).ok()?;
    let mut nth = 0usize;
    let mut holes_seen = 0usize;
    let target = pieces.iter().filter(|p| matches!(p, Piece::Hole(..))).position(|p| matches!(p, Piece::Hole(i, _) if *i == which))?;
    let mut out = String::new();
    let mut cs = t.chars().peekable();
    while let Some(c) = cs.next() {
        out.push(c);
        if c == '\\' {
            if let Some(n) = cs.next() {
                out.push(n);
            }
            continue;
        }
        if c == '{' {
            let mut inside = String::new();
            for d in cs.by_ref() {
                if d == '}' {
                    break;
                }
                inside.push(d);
            }
            if holes_seen == target {
                if let Some((head, shape)) = inside.split_once(':') {
                    let shape = shape.trim_end_matches(['x', 'X', 'o', 'b', 'e']);
                    inside = if shape.is_empty() { head.to_string() } else { format!("{}:{}", head, shape) };
                }
            }
            holes_seen += 1;
            nth += 1;
            out.push_str(&inside);
            out.push('}');
        }
    }
    let _ = nth;
    Some(out)
}
