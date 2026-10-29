//! `std::print.format`'s format text (owner, 2026-09-22; `{}` holes since
//! 2026-10-01; shapes in words since 2026-10-25; `examples/print_format.cy`,
//! `print_format_width.cy`): `{}` takes the next value in its readable form,
//! `{1}`, `{2}`… a numbered one (from 1, never mixed with `{}`); then, in any
//! order, its shapes: `align: left|right|center`, `width: N cols` (or
//! `cols.east_asian`, `clusters`, `points`, `bytes`; `col` for one),
//! `` fill: `c` ``, `places: N`, `base: bin|oct|hex|hex.upper`,
//! `form: stored|science`, and the bare words `zeros` and `sign`. Rust's
//! `{:>8.2}` is an old spelling, refused naming its words. The text has its
//! own escapes, read in one pass: `\n` `\t` `\\` `\"` `\{` `\}`; `%` is plain
//! text. Read when building, by the checker and again by the code it makes.

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
    /// `<`, `^` or `>`: `align: left`, `center`, `right`.
    pub align: Option<char>,
    /// `sign`.
    pub plus: bool,
    /// `zeros`.
    pub zero: bool,
    pub width: Option<u32>,
    /// What the width counts: 0 cols, 1 cols.east_asian, 2 clusters, 3
    /// points, 4 bytes.
    pub unit: u32,
    pub places: Option<u32>,
    /// `x` `X` `o` `b` (`base:`), `e` `?` (`form: science`, `form: stored`).
    pub kind: Option<char>,
}

/// The runtime's width tables, one file for both (Unicode 17.0.0).
#[path = "../../runtime/src/unicode_width.rs"]
#[allow(dead_code)]
mod tables;

/// A width's units, as written.
pub const UNITS: [&str; 5] = ["cols", "cols.east_asian", "clusters", "points", "bytes"];

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
                // `{newln}` and `{tab}` are a line break and a tab (redesign two).
                match inside.trim() {
                    "newln" => {
                        lit.push('\n');
                        continue;
                    }
                    "tab" => {
                        lit.push('\t');
                        continue;
                    }
                    _ => {}
                }
                // Rust's shapes, `{:>8.2}`, `{2:x}`: an old spelling.
                if let Some((head, old)) = inside.split_once(':') {
                    if head.trim().chars().all(|c| c.is_ascii_digit()) {
                        let mut words = old_words(old).unwrap_or_default();
                        if !head.trim().is_empty() {
                            words.insert(0, head.trim().to_string());
                        }
                        let hash = if old.contains('#') { "; there is no `#`: type the prefix, `0x{base: hex}`" } else { "" };
                        return Err(format!("`{{{}}}` is an old spelling: shapes are words, `{{{}}}`{} (owner, 2026-10-25)", inside, words.join(", "), hash));
                    }
                }
                let items = split_items(&inside);
                let number = items.first().filter(|i| !i.is_empty() && i.chars().all(|c| c.is_ascii_digit())).cloned();
                let index = number.clone().unwrap_or_default();
                let shape = if number.is_some() { &items[1..] } else { &items[..] };
                let shape = if shape.len() == 1 && shape[0].is_empty() { &[][..] } else { shape };
                let index = index.as_str();
                let which = if index.is_empty() {
                    plain = true;
                    next += 1;
                    next - 1
                } else {
                    let n: usize = index.parse().map_err(|_| format!("`{{{}}}` is not a hole: `{{}}`, `{{1}}`, then its shapes, `{{1, width: 8 cols}}`", inside))?;
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
                let spec = spec(&inside, shape)?;
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

/// A hole's parts, split at its commas (not inside a fill's backticks),
/// each trimmed.
fn split_items(inside: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut tick = false;
    for c in inside.chars() {
        match c {
            '`' => {
                tick = !tick;
                out.last_mut().unwrap().push(c);
            }
            ',' if !tick => out.push(String::new()),
            _ => out.last_mut().unwrap().push(c),
        }
    }
    out.into_iter().map(|s| s.trim().to_string()).collect()
}

/// The words for Rust's shape `s` (as oracle/tools/format_words.py writes
/// them): its kind, places, sign, zeros, alignment, width, fill.
fn old_words(s: &str) -> Option<Vec<String>> {
    let cs: Vec<char> = s.chars().filter(|c| *c != '#').collect();
    let mut i = 0;
    let is_align = |c: char| matches!(c, '<' | '^' | '>');
    let (mut fill, mut align) = (None, None);
    if cs.len() >= 2 && is_align(cs[1]) {
        fill = Some(cs[0]);
        align = Some(cs[1]);
        i = 2;
    } else if !cs.is_empty() && is_align(cs[0]) {
        align = Some(cs[0]);
        i = 1;
    }
    let plus = cs.get(i) == Some(&'+');
    if plus {
        i += 1;
    }
    let mut zero = cs.get(i) == Some(&'0');
    if zero {
        i += 1;
    }
    let digits = |i: &mut usize| -> Option<String> {
        let start = *i;
        while cs.get(*i).is_some_and(|c| c.is_ascii_digit()) {
            *i += 1;
        }
        (start != *i).then(|| cs[start..*i].iter().collect())
    };
    let mut width = digits(&mut i);
    let places = if cs.get(i) == Some(&'.') {
        i += 1;
        Some(digits(&mut i)?)
    } else {
        None
    };
    let kind = cs.get(i).copied();
    if kind.is_some() {
        i += 1;
    }
    if i != cs.len() {
        return None;
    }
    if align.is_some() && zero {
        width = Some(format!("0{}", width.unwrap_or_default()));
        zero = false;
    }
    let mut w = Vec::new();
    match kind {
        Some('x') => w.push("base: hex".to_string()),
        Some('X') => w.push("base: hex.upper".to_string()),
        Some('o') => w.push("base: oct".to_string()),
        Some('b') => w.push("base: bin".to_string()),
        Some('e') => w.push("form: science".to_string()),
        Some('?') => w.push("form: stored".to_string()),
        Some(_) => return None,
        None => {}
    }
    if let Some(p) = places {
        w.push(format!("places: {}", p));
    }
    if plus {
        w.push("sign".into());
    }
    if zero {
        w.push("zeros".into());
    }
    if let Some(a) = align {
        w.push(format!("align: {}", match a { '<' => "left", '>' => "right", _ => "center" }));
    }
    if let Some(n) = width {
        w.push(format!("width: {} cols", n.trim_start_matches('0').parse::<u32>().unwrap_or(0)));
    }
    if let Some(f) = fill.filter(|f| *f != ' ') {
        w.push(format!("fill: `{}`", f));
    }
    Some(w)
}

/// A hole's shapes, in words (owner, 2026-10-25).
fn spec(inside: &str, items: &[String]) -> Result<Spec, String> {
    let mut sp = Spec::default();
    let mut said: Vec<&str> = Vec::new();
    let mut fill_written: Option<char> = None;
    let mut base_or_form: Vec<&str> = Vec::new();
    for item in items {
        let (word, value) = match item.split_once(':') {
            Some((w, v)) => (w.trim(), Some(v.trim())),
            None => (item.as_str(), None),
        };
        let first = match word {
            "align" | "width" | "fill" | "places" | "base" | "form" | "zeros" | "sign" => word,
            "left" | "right" | "center" => return Err(format!("`{}` is an alignment, one of three: `align: {}`", word, word)),
            "bin" | "oct" | "hex" | "hex.upper" => return Err(format!("`{}` is a base, one of four: `base: {}`", word, word)),
            "stored" | "science" => return Err(format!("`{}` is a form, one of two: `form: {}`", word, word)),
            "" => return Err(format!("`{{{}}}` has an empty shape between its commas", inside)),
            other => return Err(format!("`{}` is not a shape: `align:`, `width:`, `fill:`, `places:`, `base:`, `form:`, `zeros` or `sign`", other)),
        };
        if said.contains(&first) {
            return Err(format!("`{{{}}}` says `{}` twice", inside, first));
        }
        said.push(first);
        match (first, value) {
            ("zeros" | "sign", Some(_)) => return Err(format!("`{}` is a word on its own, with no `:`", first)),
            ("zeros", None) => sp.zero = true,
            ("sign", None) => sp.plus = true,
            (label, None) => return Err(format!("`{}` takes its value after `:`, `{}: …`", label, label)),
            ("align", Some(v)) => {
                sp.align = Some(match v {
                    "left" => '<',
                    "right" => '>',
                    "center" => '^',
                    other => return Err(format!("`align: {}`: an alignment is `left`, `right` or `center`", other)),
                })
            }
            ("width", Some(v)) => {
                let mut parts = v.split_whitespace();
                let n: u32 = parts.next().and_then(|n| n.parse().ok()).ok_or_else(|| format!("`width: {}`: a width is a whole number and its unit, `width: 8 cols`", v))?;
                let unit = match parts.next() {
                    None => return Err(format!("`width: {}` names its unit: `width: {} cols`", n, n)),
                    Some(u) => u,
                };
                if parts.next().is_some() {
                    return Err(format!("`width: {}`: a width is a whole number and its unit, `width: 8 cols`", v));
                }
                sp.unit = match unit {
                    "col" | "cols" => 0,
                    "cols.east_asian" => 1,
                    "clusters" => 2,
                    "points" => 3,
                    "bytes" => 4,
                    _ => return Err(format!("`width: {} {}`: a width counts `cols`, `cols.east_asian`, `clusters`, `points` or `bytes`", n, unit)),
                };
                sp.width = Some(n);
            }
            ("fill", Some(v)) => {
                let inner = v.strip_prefix('`').and_then(|r| r.strip_suffix('`')).filter(|r| r.chars().count() == 1);
                let Some(c) = inner.and_then(|r| r.chars().next()) else {
                    return Err("a fill is one character in backticks: `fill: `*``".into());
                };
                fill_written = Some(c);
            }
            ("places", Some(v)) => sp.places = Some(v.parse().map_err(|_| format!("`places: {}`: places are a whole number, `places: 2`", v))?),
            ("base", Some(v)) => {
                base_or_form.push("base");
                sp.kind = Some(match v {
                    "hex" => 'x',
                    "hex.upper" => 'X',
                    "oct" => 'o',
                    "bin" => 'b',
                    other => return Err(format!("`base: {}`: a base is `bin`, `oct`, `hex` or `hex.upper`", other)),
                });
            }
            ("form", Some(v)) => {
                base_or_form.push("form");
                sp.kind = Some(match v {
                    "stored" => '?',
                    "science" => 'e',
                    other => return Err(format!("`form: {}`: a form is `stored` or `science`", other)),
                });
            }
            _ => unreachable!(),
        }
    }
    if base_or_form.len() > 1 {
        return Err(format!("`{{{}}}` has a `base:` and a `form:`; a value is written one way", inside));
    }
    if sp.width.is_none() && (sp.align.is_some() || fill_written.is_some() || sp.zero) {
        return Err(format!("`{{{}}}`: there is nothing to pad without a `width:`", inside));
    }
    if sp.zero && sp.align.is_some() {
        return Err(format!("`{{{}}}`: `zeros` pads after the sign, so it has no alignment", inside));
    }
    if let Some(c) = fill_written {
        let wide = unit_width(c, sp.unit);
        if wide != 1 {
            return Err(format!("the fill `{}` is {} {} wide; a fill takes one", c, wide, UNITS[sp.unit as usize]));
        }
        sp.fill = Some(c);
    }
    Ok(sp)
}

/// One character's width in a width's unit (owner, 2026-10-25), as the
/// runtime measures: Unicode 17.0.0's tables, `runtime/src/unicode_width.rs`.
pub fn unit_width(c: char, unit: u32) -> u32 {
    let within = |t: &[(u32, u32)]| t.binary_search_by(|&(a, b)| if (c as u32) < a { std::cmp::Ordering::Greater } else if (c as u32) > b { std::cmp::Ordering::Less } else { std::cmp::Ordering::Equal }).is_ok();
    match unit {
        0 | 1 if within(&tables::ZERO) && c != '\u{AD}' => 0,
        0 | 1 if within(&tables::WIDE) || within(&tables::EMOJI_PRESENTATION) => 2,
        1 if within(&tables::AMBIGUOUS) => 2,
        0 | 1 | 2 | 3 => 1,
        _ => c.len_utf8() as u32,
    }
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
/// written `%`, holes counted from `{1}`, a lone brace escaped, Rust's
/// shapes in words, a bare choice given its label, `zeros:` a bare word, a
/// width given its unit, a fill's quotes made backticks. `None` when no one
/// repair makes it read (mixed holes, a shape said twice: which is meant
/// is the writer's).
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
        } else if why.contains("is an old spelling: shapes are words") {
            map_holes(&t, |inside| {
                let (head, old) = inside.split_once(':')?;
                if !head.trim().chars().all(|c| c.is_ascii_digit()) {
                    return None;
                }
                let mut w = old_words(old)?;
                if !head.trim().is_empty() {
                    w.insert(0, head.trim().to_string());
                }
                Some(w.join(", "))
            })
        } else {
            map_holes(&t, |inside| {
                let items = split_items(inside);
                let mut changed = false;
                let fixed: Vec<String> = items.iter().map(|item| {
                    let (word, value) = match item.split_once(':') {
                        Some((w, v)) => (w.trim(), Some(v.trim())),
                        None => (item.as_str(), None),
                    };
                    let new = match (word, value) {
                        ("left" | "right" | "center", None) => Some(format!("align: {}", word)),
                        ("bin" | "oct" | "hex" | "hex.upper", None) => Some(format!("base: {}", word)),
                        ("stored" | "science", None) => Some(format!("form: {}", word)),
                        ("zeros" | "sign", Some(_)) => Some(word.to_string()),
                        ("width", Some(v)) => {
                            let parts: Vec<&str> = v.split_whitespace().collect();
                            match parts.as_slice() {
                                [n] if n.parse::<u32>().is_ok() => Some(format!("width: {} cols", n)),
                                [n, u] if !UNITS.contains(u) && *u != "col" => crate::diag::closest(u, UNITS).map(|c| format!("width: {} {}", n, c)),
                                _ => None,
                            }
                        }
                        ("fill", Some(v)) => {
                            let q = v.strip_prefix("\\\"").and_then(|r| r.strip_suffix("\\\"")).or_else(|| v.strip_prefix('"').and_then(|r| r.strip_suffix('"'))).or_else(|| v.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')));
                            q.filter(|r| r.chars().count() == 1).map(|r| format!("fill: `{}`", r))
                        }
                        _ => None,
                    };
                    changed |= new.is_some();
                    new.unwrap_or_else(|| item.clone())
                }).collect();
                changed.then(|| fixed.join(", "))
            })
        };
        if next == t {
            return None;
        }
        t = next;
    }
    None
}

/// Each hole's inside rewritten by `f` where it gives a new one, escapes
/// and a fill's backticks kept as they are.
fn map_holes(t: &str, f: impl Fn(&str) -> Option<String>) -> String {
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
            let mut tick = false;
            let mut closed = false;
            for d in cs.by_ref() {
                if d == '`' {
                    tick = !tick;
                }
                if d == '}' && !tick {
                    closed = true;
                    break;
                }
                inside.push(d);
            }
            out.push_str(&f(&inside).unwrap_or(inside));
            if closed {
                out.push('}');
            }
        }
    }
    out
}

/// Each numbered hole one higher: `{0}` was meant as the first value.
fn renumber(t: &str) -> Option<String> {
    let out = map_holes(t, |inside| {
        let items = split_items(inside);
        let n = items.first().filter(|i| !i.is_empty() && i.chars().all(|c| c.is_ascii_digit()))?;
        let Ok(n) = n.parse::<usize>() else {
            return None;
        };
        let rest: Vec<String> = items[1..].to_vec();
        let mut all = vec![(n + 1).to_string()];
        all.extend(rest);
        Some(all.join(", "))
    });
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

/// The text with the shape labelled `label` (`base`, `form`, `places`,
/// `sign`, `zeros`) taken off hole number `which` (from 0): a shape that
/// does not fit the value it shows.
pub fn without_shape(t: &str, which: usize, label: &str) -> Option<String> {
    let pieces = parse(t).ok()?;
    let target = pieces.iter().filter(|p| matches!(p, Piece::Hole(..))).position(|p| matches!(p, Piece::Hole(i, _) if *i == which))?;
    let seen = std::cell::Cell::new(0usize);
    Some(map_holes(t, |inside| {
        let n = seen.get();
        seen.set(n + 1);
        if n != target {
            return None;
        }
        let items = split_items(inside);
        let kept: Vec<String> = items.into_iter().filter(|i| i.split(':').next().map(str::trim) != Some(label)).collect();
        Some(kept.join(", "))
    }))
}
