//! Collection templates (owner, 2026-10-13): `kayo generate` runs each template in a collection and
//! writes what it makes into the source, under `#generated_N_lines_below`; a build only reads, and
//! refuses a generated part that no longer matches its template. The oracle does both: it checks every
//! program it loads, and `oracle --generate <path>` rewrites the generated parts.
//!
//! A template is `for 'v' in [ … ] { named <text>; [ … ] -> T { … } }`, or a counting `for 'v' from A to B by
//! C { … }` (`before` too), over what is known before building: number and text literals, consts, and
//! types. The `named` line is CyborgPL, run with `'v'` set (a type is its name, as text); the member under it
//! is copied with `'v'` written as the value itself.

use std::path::PathBuf;

/// A file with templates, kept from parsing for the check: its path, its text and its module.
pub struct Pending { pub path: PathBuf, pub src: String, pub module: Vec<String>, pub kit_root: Vec<String> }

thread_local! {
    pub static PENDING: std::cell::RefCell<Vec<Pending>> = std::cell::RefCell::new(Vec::new());
}

/// One template, read from the text: where it is, its loop, its `named` text, and its member.
struct Template { for_line: usize, close_line: usize, indent: String, var: String, items: Vec<String>, counting: Option<(String, bool, String, String)>, named: String, member: Vec<String> }

fn indent_of(l: &str) -> String { l.chars().take_while(|c| *c == ' ' || *c == '\t').collect() }

/// Braces opened minus closed on a line, outside text and comments.
fn brace_delta(l: &str) -> i32 {
    let (mut d, mut in_text, mut prev) = (0, false, ' ');
    for c in l.chars() {
        if in_text { if c == '"' && prev != '\\' { in_text = false; } prev = c; continue; }
        match c { '"' => in_text = true, '#' => break, '{' => d += 1, '}' => d -= 1, _ => {} }
        prev = c;
    }
    d
}

fn split_top(s: &str) -> Vec<String> {
    let (mut out, mut d, mut cur, mut in_text) = (Vec::new(), 0i32, String::new(), false);
    for c in s.chars() {
        if in_text { cur.push(c); if c == '"' { in_text = false; } continue; }
        match c { '"' => in_text = true, '[' | '(' => d += 1, ']' | ')' => d -= 1, ',' if d == 0 => { out.push(cur.trim().to_string()); cur.clear(); continue; } _ => {} }
        cur.push(c);
    }
    if !cur.trim().is_empty() { out.push(cur.trim().to_string()); }
    out
}

fn read_templates(src: &str, lines_at: &[usize]) -> Result<Vec<Template>, String> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for &at in lines_at {
        let i = at - 1;
        let head = lines.get(i).ok_or("a template's line is past the file's end")?;
        let indent = indent_of(head);
        let h = head.trim();
        let after_for = h.strip_prefix("for ").ok_or(format!("line {at}: a template starts with `for`"))?;
        let q = after_for.strip_prefix('\'').ok_or(format!("line {at}: a template's variable is quoted: `for 'n' in [ … ]`"))?;
        let e = q.find('\'').ok_or(format!("line {at}: a template's variable is quoted"))?;
        let var = q[..e].to_string();
        let rest = q[e + 1..].trim();
        let rest = rest.strip_suffix('{').ok_or(format!("line {at}: a template's `{{` ends its first line"))?.trim();
        let (items, counting) = if let Some(list) = rest.strip_prefix("in ") {
            let list = list.trim().strip_prefix('[').and_then(|l| l.strip_suffix(']')).ok_or(format!("line {at}: a template runs over a list written out, `in [2, 4, 8]`"))?;
            (split_top(list), None)
        } else if let Some(c) = rest.strip_prefix("from ") {
            let (a, r) = c.split_once(' ').ok_or(format!("line {at}: `from A to B by C`"))?;
            let (word, r) = r.trim().split_once(' ').ok_or(format!("line {at}: `from A to B by C`"))?;
            if word != "to" && word != "before" { return Err(format!("line {at}: after the start comes `to` or `before`")); }
            let (b, r) = r.trim().split_once(" by ").ok_or(format!("line {at}: a counting template writes its step: `by 1`"))?;
            (Vec::new(), Some((a.trim().to_string(), word == "before", b.trim().to_string(), r.trim().to_string())))
        } else { return Err(format!("line {at}: a template runs `in [ … ]` or `from A to B by C`")); };
        // its body: up to the `}` closing the `for`
        let mut depth = 1;
        let mut j = i + 1;
        let mut body = Vec::new();
        while j < lines.len() {
            depth += brace_delta(lines[j]);
            if depth == 0 { break; }
            body.push(lines[j].to_string());
            j += 1;
        }
        if depth != 0 { return Err(format!("line {at}: the template's `{{` is never closed")); }
        let text = body.join("\n");
        let n = text.find("named ").ok_or(format!("line {at}: a template names what it makes first: `named \"times_\" ++ …;`"))?;
        let semi = text[n..].find(';').ok_or(format!("line {at}: the `named` line ends with `;`"))? + n;
        let named = text[n + 6..semi].trim().to_string();
        let member: Vec<String> = text[semi + 1..].lines().map(str::to_string).skip_while(|l| l.trim().is_empty()).collect();
        let member: Vec<String> = { let mut m = member; while m.last().is_some_and(|l| l.trim().is_empty()) { m.pop(); } m };
        if member.is_empty() { return Err(format!("line {at}: a template writes a member under its `named` line: `[ … ] -> T {{ … }}`")); }
        out.push(Template { for_line: at, close_line: j + 1, indent, var, items, counting, named, member });
    }
    Ok(out)
}

fn is_type(item: &str) -> bool {
    let w = item.split(['[', '.']).next().unwrap_or("");
    ["int", "uint", "bin", "deci", "bool", "str", "char", "C", "brain16", "list", "array", "map", "ptr"].iter().any(|p| w.starts_with(p) && w[p.len()..].chars().all(|c| c.is_ascii_digit()))
        || w.starts_with(|c: char| c.is_ascii_uppercase())
}

/// `'v'` written as the value, outside text.
fn put(text: &str, var: &str, value: &str) -> String {
    let pat = format!("'{var}'");
    let (mut out, mut in_text, mut i) = (String::new(), false, 0);
    let b = text.as_bytes();
    while i < text.len() {
        if !in_text && text[i..].starts_with(&pat) { out.push_str(value); i += pat.len(); continue; }
        if b[i] == b'"' { in_text = !in_text; }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// What every template in the program makes: per file, (template, the lines it makes). Each `named` line
/// runs as a small function added to the program, all of them in one go.
fn expected(prog: &mut crate::ast::Program) -> Result<Vec<(usize, Vec<(Template, Vec<String>)>)>, String> {
    let files: Vec<(PathBuf, String, Vec<String>, Vec<String>, Vec<usize>)> = PENDING.with(|p| p.borrow().iter().map(|f| {
        let lines = crate::parser::TEMPLATES.with(|t| t.borrow().iter().filter(|(path, _, _)| *path == f.path).map(|(_, _, l)| *l).collect());
        (f.path.clone(), f.src.clone(), f.module.clone(), f.kit_root.clone(), lines)
    }).collect());
    let mut plans = Vec::new();
    let mut calls: Vec<(Vec<String>, String)> = Vec::new();
    let mut k = 0;
    for (fi, (path, src, module, root, lines_at)) in files.iter().enumerate() {
        let ts = read_templates(src, lines_at).map_err(|m| format!("{}: {m}", path.display()))?;
        let mut per = Vec::new();
        for t in ts {
            // the values: written out, or counted
            let values: Vec<String> = match &t.counting {
                None => t.items.clone(),
                Some((a, before, b, c)) => {
                    let num = |x: &str| x.replace('_', "").parse::<i128>().map_err(|_| format!("{}: line {}: a counting template's bounds are whole numbers written out", path.display(), t.for_line));
                    let (a, b, c) = (num(a)?, num(b)?, num(c)?);
                    if c == 0 { return Err(format!("{}: line {}: a step of 0 never ends", path.display(), t.for_line)); }
                    let mut v = Vec::new();
                    let mut i = a;
                    while if *before { (c > 0 && i < b) || (c < 0 && i > b) } else { (c > 0 && i <= b) || (c < 0 && i >= b) } { v.push(i.to_string()); i += c; }
                    v
                }
            };
            let mut made = Vec::new();
            for v in &values {
                let fname = format!("generated_name_{k}");
                k += 1;
                let src_fn = if is_type(v) {
                    format!("fn {fname} [] -> str.utf8 {{ return {}; }}", put(&t.named, &t.var, &format!("\"{v}\"")))
                } else {
                    format!("fn {fname} [] -> str.utf8 {{ let '{}' := {v}; return {}; }}", t.var, t.named)
                };
                let toks = crate::lexer::lex(&src_fn).map_err(|e| format!("{}: line {}: the `named` line: {}", path.display(), t.for_line, e.msg))?;
                let part = crate::parser::Parser::new_in_kit(toks, module.clone(), root.clone()).program().map_err(|e| format!("{}: line {}: the `named` line: {}", path.display(), t.for_line, e.msg))?;
                prog.funcs.extend(part.funcs);
                calls.push((module.clone(), fname));
                made.push(v.clone());
            }
            per.push((t, made));
        }
        plans.push((fi, per));
    }
    // run every `named` line
    let names = {
        let mut it = crate::interp::Interp::new(prog, Box::new(std::io::sink()));
        it.call_texts(&calls).map_err(|e| format!("a `named` line: {}", e.msg))?
    };
    prog.funcs.retain(|f| !f.name.starts_with("generated_name_"));
    let mut n = names.into_iter();
    let mut out = Vec::new();
    for (fi, per) in plans {
        let mut ts = Vec::new();
        for (t, values) in per {
            let mut lines = Vec::new();
            let base = indent_of(t.member.first().map(String::as_str).unwrap_or(""));
            for v in &values {
                let name = n.next().unwrap_or_default();
                for (li, l) in t.member.iter().enumerate() {
                    let body = put(l.strip_prefix(base.as_str()).unwrap_or(l.trim_start()), &t.var, v);
                    lines.push(if li == 0 { format!("{}{name} {body}", t.indent) } else { format!("{}{body}", t.indent) });
                }
            }
            ts.push((t, lines));
        }
        out.push((fi, ts));
    }
    Ok(out)
}

/// The check every load makes: each template's generated part is there, under its marker, as made.
pub fn check(prog: &mut crate::ast::Program) -> Result<(), String> {
    if PENDING.with(|p| p.borrow().is_empty()) { return Ok(()); }
    let plans = expected(prog)?;
    for (fi, ts) in plans {
        let (path, src) = PENDING.with(|p| { let p = p.borrow(); (p[fi].path.clone(), p[fi].src.clone()) });
        let lines: Vec<&str> = src.lines().collect();
        for (t, made) in ts {
            let marker = lines.get(t.close_line).map(|l| l.trim()).unwrap_or("");
            let want = format!("#generated_{}_lines_below", made.len());
            let have: Vec<&str> = lines.iter().skip(t.close_line + 1).take(made.len()).copied().collect();
            if marker != want || have.iter().map(|l| l.trim_end()).ne(made.iter().map(|l| l.trim_end())) {
                return Err(format!("{}: line {}: the generated part under the template at line {} does not match it: run `kayo generate` (in the oracle, `oracle --generate {}`) (owner, 2026-10-13)", path.display(), t.close_line + 1, t.for_line, path.display()));
            }
        }
    }
    Ok(())
}

/// `oracle --generate`: write each template's generated part under it, replacing the old one.
pub fn write(prog: &mut crate::ast::Program) -> Result<usize, String> {
    let plans = expected(prog)?;
    let mut written = 0;
    for (fi, ts) in plans {
        let (path, src) = PENDING.with(|p| { let p = p.borrow(); (p[fi].path.clone(), p[fi].src.clone()) });
        let mut lines: Vec<String> = src.lines().map(str::to_string).collect();
        // from the last template up, so the earlier lines keep their numbers
        let mut ts = ts;
        ts.sort_by_key(|(t, _)| std::cmp::Reverse(t.close_line));
        for (t, made) in ts {
            let at = t.close_line;
            // an old part: the marker and its lines
            if let Some(old) = lines.get(at).and_then(|l| l.trim().strip_prefix("#generated_").and_then(|r| r.strip_suffix("_lines_below")).and_then(|d| d.parse::<usize>().ok())) {
                let end = (at + 1 + old).min(lines.len());
                lines.drain(at..end);
            }
            let mut part = vec![format!("{}#generated_{}_lines_below", t.indent, made.len())];
            part.extend(made);
            lines.splice(at..at, part);
            written += 1;
        }
        let mut text = lines.join("\n");
        if src.ends_with('\n') { text.push('\n'); }
        std::fs::write(&path, text).map_err(|e| format!("{}: cannot write: {e}", path.display()))?;
    }
    Ok(written)
}
