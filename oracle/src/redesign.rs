//! The redesign of 2026-10-26 and 2026-10-27 (DECISIONS.md, every "Redesign: …" entry): `oracle --redesign` rewrites
//! a program in the spellings before it into the new ones, in place.
//!
//! What it changes: blocks (`{ }` → an opening line ending `:` and `end of` its kind and name), `!` where something
//! needs attention (`!MAIN`, `!no_check`), `fn` → `function` (pure) or `procedure` (effects), `let` → `let` (never
//! changes) or `variable`, the operator symbols → words, a quoted name → `$name`, the number types → `integer.signed.32_bit`
//! and the like, and every renamed word. It never touches comments or text inside quotes, except a format's `\n` / `\t`
//! and its holes' words, which are code. Design claude's provisional picks, where the rulings say nothing, are marked
//! PROVISIONAL here and listed in oracle/README.md as open questions.

use crate::ast::{self, Expr, Stmt, Handler};
use crate::lexer::{Tok, Token};
use std::collections::{HashMap, HashSet};

/// PROVISIONAL (asked 2026-10-27): `lend.mut`, `lent.mut`, `ptr.mut[T]` become `.mutable`
pub const MUTABLE: &str = "mutable";

/// What the converter needs to know that the tokens do not say: which functions have effects, and which `let`s change.
#[derive(Default)]
pub struct Info {
    /// the lines of each `fn` that is a procedure
    pub procedures: HashSet<usize>,
    /// the lines of each `fn` known to be pure
    pub functions: HashSet<usize>,
    /// (line, name) of each declaration something changes
    pub changed: HashSet<(usize, String)>,
    /// (line, name) of each declaration seen, changed or not
    pub declared: HashSet<(usize, String)>,
}

/// The effects a body has of its own, and the names of the functions it calls: a function is pure when neither it nor
/// anything it calls prints, reads, waits, exits, starts a task, uses a channel, reaches C or a pointer, or runs
/// another language's code; changing its own local variables is not an effect (design claude's reading).
fn effects_of(f: &ast::Func) -> (bool, Vec<String>) {
    let text = format!("{:?}", f.body);
    let mut effect = f.is_extern || f.params.iter().any(|p| p.kind == ast::ParamKind::LentMut) || f.ret_lent.as_ref().is_some_and(|(m, _)| *m);
    for marker in ["Asm {", "Msl {", "Concur {", "Whichever {", "ChannelNew(", "PtrReadChecked(", "PtrCast(", "MustStop"] {
        if text.contains(marker) { effect = true; }
    }
    // std's calls with effects
    for (kit, path) in calls_in(&text) {
        if kit.as_deref() == Some("std") {
            let head = path.first().map(String::as_str).unwrap_or("");
            if matches!(head, "print" | "input" | "exit" | "time" | "channel" | "printu" | "println" | "printf") { effect = true; }
        }
        if kit.is_none() && matches!(path.first().map(String::as_str), Some("ptr") | Some("channel")) { effect = true; }
    }
    if text.contains("Builtin(\"ptr\"") || text.contains("head: \"ptr\"") { effect = true; }
    let called = calls_in(&text).into_iter().filter(|(k, _)| k.is_none()).filter_map(|(_, p)| p.last().cloned()).collect();
    (effect, called)
}

/// `(kit, path)` of every call written in a body's debug text
fn calls_in(text: &str) -> Vec<(Option<String>, Vec<String>)> {
    let mut out = Vec::new();
    for rest in text.split("kit: ").skip(1) {
        let (kit, after) = if let Some(r) = rest.strip_prefix("None, path: [") { (None, r) }
            else if let Some(r) = rest.strip_prefix("Some(\"") { let Some(e) = r.find('"') else { continue }; let kit = r[..e].to_string(); let Some(r) = r[e..].strip_prefix("\"), path: [") else { continue }; (Some(kit), r) }
            else { continue };
        let Some(end) = after.find(']') else { continue };
        let path = after[..end].split(", ").map(|s| s.trim_matches('"').to_string()).filter(|s| !s.is_empty()).collect();
        out.push((kit, path));
    }
    // a call through a variable, `$f($x)`: the value's type says; a parameter typed `fn[…]` is a function by default
    out
}

/// Which functions are procedures: those with an effect, and those that call one, to a fixed point. A call is matched
/// by its last name, so two functions of one name anywhere count as one (design claude's reading, for the converter).
pub fn analyse(prog: &ast::Program) -> Info {
    let mut info = Info::default();
    let all: Vec<(&ast::Func, bool, Vec<String>)> = prog.funcs.iter().map(|f| { let (e, c) = effects_of(f); (f, e, c) }).collect();
    let mut impure: HashSet<String> = HashSet::new();
    let name_of = |f: &ast::Func| f.name.rsplit('.').next().unwrap_or(&f.name).to_string();
    loop {
        let before = impure.len();
        for (f, e, called) in &all {
            if *e || called.iter().any(|c| impure.contains(c)) { impure.insert(name_of(f)); }
        }
        if impure.len() == before { break; }
    }
    for (f, e, called) in &all {
        if *e || called.iter().any(|c| impure.contains(c)) || f.is_test { info.procedures.insert(f.line); } else { info.functions.insert(f.line); }
    }
    crate::CHANGED.with(|c| for (line, name, changed) in c.borrow().iter() {
        info.declared.insert((*line, name.clone()));
        if *changed { info.changed.insert((*line, name.clone())); }
    });
    info
}

/// One edit to the source: the characters `at..end` become `text`.
struct Edit { at: usize, end: usize, text: String }

/// What a block is, for its `end of` line
#[derive(Clone)]
struct Open {
    /// what `end of` says after it: `function add`, `if`, `case hot`
    close: String,
    /// the `!` on the closing line
    bang: bool,
    /// the indentation of the line the block opens on, in the new text
    indent: String,
    /// arms inside: each `name $v { }` becomes `case name $v:`
    arms: bool,
    /// fields or cases inside, ended `;` (`,` between them before)
    fields: bool,
    /// the member lines of a collection
    members: Option<String>,
    /// its arms are a handler's (`otherwise`, a `coll.handler` member): one that jumps wears `!`
    handler: bool,
    /// this block is an arm: `case` goes in front of its line
    arm: bool,
    /// inside a `precedence` block: `level N { … }` is a list, not a block
    levels: bool,
    /// a bare `{ … }`, which gets the word `block` on its opening line
    bare: bool,
    /// a `{ … }` that is a list (a precedence level's operators): kept, its braces become brackets
    list: bool,
}

/// The number types by their new names (owner, 2026-10-27)
pub fn number_type(w: &str) -> Option<String> {
    let (kind, rest) = if let Some(r) = w.strip_prefix('i') { ("integer.signed", r) } else if let Some(r) = w.strip_prefix('u') { ("integer.unsigned", r) }
        else if let Some(r) = w.strip_prefix('f') { ("float.binary", r) } else if let Some(r) = w.strip_prefix('d') { ("float.decimal", r) }
        else if w == "brain16" { return Some("float.brain.16_bit".into()) } else { return None };
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) { return None; }
    let n: u32 = rest.parse().ok()?;
    let ok = match kind { "integer.signed" | "integer.unsigned" => matches!(n, 8 | 16 | 32 | 64 | 128), "float.binary" => matches!(n, 16 | 32 | 64 | 128), _ => matches!(n, 32 | 64 | 128) };
    if !ok { return None; }
    Some(format!("{kind}.{n}_bit"))
}

/// PROVISIONAL (asked 2026-10-27): a limb's number type wears its mark on the part the limb adds
fn marked_number_type(w: &str) -> Option<String> {
    let t = number_type(w)?;
    Some(match w {
        "d32" | "d64" | "d128" => t.replacen("float.decimal", "float.%decimal", 1),
        "brain16" => t.replacen("float.brain", "float.%brain", 1),
        _ if w.ends_with("128") && (w.starts_with('i') || w.starts_with('u') || w.starts_with('f')) => t.replacen(".128_bit", ".%128_bit", 1),
        _ => t,
    })
}

/// A word renamed wherever it stands as a keyword or a word of the language (not a name in a path)
fn keyword(w: &str) -> Option<&'static str> {
    Some(match w {
        "struct" => "record",
        "const" => "constant",
        "pub" => "public",
        "coll" => "collection",
        "ptr" => "pointer",

        "op" => "operator",
        "extern" => "external",
        "seq" => "sequential",
        "para" => "parallel",
        "temp" => "temporary",
        "perm" => "permanent",
        "no_cheat" => "no_skip",
        "bool" => "boolean",
        "char" => "character",
        "str" => "string",
        "concat" => "concatenate",
        "div" => "divide",
        "mod" => "modulo",
        "shl" => "shift_left",
        "shr" => "shift_right",
        "not_equal" => "not_equal_to",
        "newln" => "newline",
        "secs" => "seconds",
        "mins" => "minutes",
        "cols" => "columns",
        "bin" => "binary",
        "hex" => "hexadecimal",
        "oct" => "octal",
        "pos" => "position",
        "concur" => "spawn.task",
        "xx" => "power",
        "x" => "times",
        _ => return None,
    })
}

/// A word in a path, after `.`, renamed by the segment before it (std's names, the narrowings)
fn segment(before: &str, w: &str) -> Option<&'static str> {
    Some(match (before, w) {
        ("print", "useful") | ("weird", "useful") | ("to", "useful") => "shortest",
        ("print", "weird") => "arrows",
        (_, "auto_newln") => "auto_newline",
        ("str", "upper") => "uppercase",
        ("str", "lower") => "lowercase",
        ("str" | "char" | "slice" | "at" | "each" | "find" | "length" | "count" | "to", "point") => "code_point",
        ("str" | "char" | "slice" | "at" | "each" | "find" | "length" | "count", "points") => "code_points",
        ("str" | "char" | "slice" | "at" | "each" | "find" | "length" | "count" | "to", "grapheme") => "grapheme_cluster",
        ("str" | "char" | "slice" | "at" | "each" | "find" | "length" | "count", "graphemes") => "grapheme_clusters",
        ("map", "length") => "count",
        ("div" | "mod" | "round", "near") => "nearest",
        ("div" | "mod" | "round", "trunc") => "truncate",
        ("div" | "mod" | "round", "ceil") => "ceiling",
        ("lend" | "lent" | "ptr", "mut") => MUTABLE,
        ("coll", "fn") => "function",
        ("coll", "bound") => "bounded",
        (_, "str") => "string",
        (_, "char") => "character",
        (_, "bool") => "boolean",
        (_, "pow") => "power",
        (_, "xx") => "power",
        (_, "shl") => "shift_left",
        (_, "shr") => "shift_right",
        ("pub", "kit") => "kit",
        _ => return None,
    })
}

/// The labels, a word before `:` inside `[ ]` or `( )`
fn label(w: &str) -> Option<&'static str> {
    Some(match w { "min" => "minimum", "max" => "maximum", "pos" => "position", "min_places" => "minimum_places", "max_places" => "maximum_places", _ => return None })
}

/// The symbol operators and their words (owner, 2026-10-27)
fn operator_word(o: &str) -> Option<&'static str> {
    Some(match o { "+" => "plus", "-" => "minus", "<" => "less_than", ">" => "greater_than", "x~" => "times.wrap", "xx~" => "power.wrap", _ => return None })
}

/// Rewrites one file's text. `info` is what the analysis found, or `None` when the file did not parse on its own.
pub fn convert(src: &str, toks: &[Token], info: Option<&Info>) -> Result<String, String> {
    let chars: Vec<char> = src.chars().collect();
    let text = |t: &Token| -> String { if t.at == usize::MAX || t.end == usize::MAX { String::new() } else { chars[t.at..t.end].iter().collect() } };
    let mut edits: Vec<Edit> = Vec::new();
    let line_indent = |at: usize| -> String {
        let mut s = at;
        while s > 0 && chars[s - 1] != '\n' { s -= 1; }
        chars[s..].iter().take_while(|c| **c == ' ' || **c == '\t').collect()
    };
    // the first token of each line
    let first_on_line = |k: usize| -> bool { k == 0 || toks[k - 1].line != toks[k].line };
    let mut stack: Vec<Open> = Vec::new();
    // indentation, in the new text, of the line each token now sits on: the line's own, or the one a split gave it
    let mut cur_indent = String::new();
    // when a fn's purity is unknown (the file did not parse), a body with an effect word is a procedure
    let n = toks.len();
    // the start of the statement each `{` belongs to
    let mut stmt_start = 0usize;
    let mut depth_brackets = 0i32;
    let mut k = 0;
    // the `{`s that open no block: a `precedence` level's operator list
    let mut list_braces: Vec<bool> = Vec::new();
    while k < n {
        let t = &toks[k];
        if t.at != usize::MAX && first_on_line(k) { cur_indent = line_indent(t.at); }
        match &t.tok {
            Tok::LBracket | Tok::LParen => depth_brackets += 1,
            Tok::RBracket | Tok::RParen => depth_brackets -= 1,
            _ => {}
        }
        match &t.tok {
            Tok::LBrace => {
                let open = describe(toks, stmt_start, k, &text, info, stack.last());
                if open.list {
                    let inner = toks.get(k + 1).map_or(t.end, |nx| if nx.line == t.line { nx.at } else { t.end });
                    edits.push(Edit { at: t.at, end: inner, text: "[".into() });
                    list_braces.push(true);
                    stack.push(open);
                    k += 1;
                    continue;
                }
                list_braces.push(false);
                // the `{` and the space before it become `:`
                let prev_end = if k > 0 { toks[k - 1].end } else { t.at };
                let from = if open.bare && first_on_line(k) { t.at } else if prev_end != usize::MAX && prev_end <= t.at { prev_end } else { t.at };
                let body_indent = format!("{}    ", cur_indent);
                let next_same_line = k + 1 < n && toks[k + 1].line == t.line && !matches!(toks[k + 1].tok, Tok::Eof | Tok::RBrace);
                let mut colon = if open.bare { format!("\n{cur_indent}block:") } else { ":".to_string() };
                if open.bare && first_on_line(k) { colon = "block:".into(); }
                if next_same_line {
                    colon.push('\n');
                    colon.push_str(&body_indent);
                    // the text up to the next token goes too
                    edits.push(Edit { at: from, end: toks[k + 1].at.min(chars.len()), text: colon });
                } else {
                    edits.push(Edit { at: from, end: t.end, text: colon });
                }
                let mut open = open;
                if open.arm && stmt_start < k && toks[stmt_start].at != usize::MAX {
                    let a = toks[stmt_start].at;
                    edits.push(Edit { at: a, end: a, text: if open.bang { "!case ".into() } else { "case ".into() } });
                }
                open.indent = cur_indent.clone();
                if next_same_line { cur_indent = body_indent; }
                stack.push(open);
                stmt_start = k + 1;
            }
            Tok::RBrace => {
                let Some(open) = stack.pop() else { return Err(format!("line {}: a `}}` with no block open", t.line)); };
                let was_list = list_braces.pop().unwrap_or(false);
                if was_list {
                    let from = if k > 0 && toks[k - 1].line == t.line && toks[k - 1].end != usize::MAX { toks[k - 1].end } else { t.at };
                    edits.push(Edit { at: from, end: t.end, text: "]".into() });
                    k += 1;
                    continue;
                }
                // a last field or case without its `,` gets its `;`
                if open.fields && k > 0 && !matches!(toks[k - 1].tok, Tok::Comma | Tok::Semi | Tok::LBrace) {
                    let e = toks[k - 1].end;
                    if e != usize::MAX { edits.push(Edit { at: e, end: e, text: ";".into() }); }
                }
                let bang = if open.bang { "!" } else { "" };
                let closing = format!("{bang}end of {}", open.close);
                let alone_before = first_on_line(k);
                let mut out = String::new();
                let from;
                if alone_before {
                    from = t.at;
                } else {
                    // the line goes on before the `}`: the closing line is its own
                    let pe = toks[k - 1].end;
                    from = if pe != usize::MAX && pe <= t.at { pe } else { t.at };
                    out.push('\n');
                    out.push_str(&open.indent);
                }
                out.push_str(&closing);
                // what follows on the same line: `;`, `)`, `,` stay on it; anything else starts a line
                let next = toks.get(k + 1);
                let mut end = t.end;
                if let Some(nx) = next {
                    if nx.line == t.line && nx.tok != Tok::Eof && !matches!(nx.tok, Tok::Semi | Tok::RParen | Tok::RBracket | Tok::Comma | Tok::RBrace) {
                        out.push('\n');
                        out.push_str(&open.indent);
                        end = nx.at;
                        cur_indent = open.indent.clone();
                    } else if alone_before { cur_indent = open.indent.clone(); }
                    else { cur_indent = open.indent.clone(); }
                }
                edits.push(Edit { at: from, end, text: out });
                stmt_start = k + 1;
            }
            Tok::Semi => { if depth_brackets == 0 { stmt_start = k + 1; } }
            _ => {}
        }
        // an interface's types, one per line, each ends `;` (PROVISIONAL: as fields do)
        if stack.last().is_some_and(|o| o.close.starts_with("interface ")) && depth_brackets == 0 && k + 1 < n && toks[k + 1].line != t.line
            && !matches!(t.tok, Tok::Semi | Tok::LBrace | Tok::Comma) && toks[k + 1].tok != Tok::RBrace && t.end != usize::MAX {
            edits.push(Edit { at: t.end, end: t.end, text: ";".into() });
        }
        // inside fields, a `,` between fields is `;`
        if t.tok == Tok::Comma && depth_brackets == 0 && stack.last().is_some_and(|o| o.fields) {
            edits.push(Edit { at: t.at, end: t.end, text: ";".into() });
        }
        k += 1;
    }
    inline_blocks(toks, &chars, &mut edits, &line_indent);
    words(toks, &text, info, &mut edits, &chars)?;
    apply(&chars, edits)
}

/// `inline.ASM { code } with [ … ];` → `inline.ASM with [ … ]:`, the code on its own lines, `end of inline.ASM` (owner,
/// 2026-10-26: the bindings on the start line, the code up to the exact `end of` line)
fn inline_blocks(toks: &[Token], chars: &[char], edits: &mut Vec<Edit>, line_indent: &dyn Fn(usize) -> String) {
    for i in 2..toks.len() {
        let Tok::Word(lang) = &toks[i].tok else { continue };
        if !matches!(lang.as_str(), "ASM" | "MSL" | "S" | "Assembly" | "Metal_Shading_Language") { continue; }
        let marked = toks[i - 1].tok == Tok::Mark;
        let d = if marked { i - 2 } else { i - 1 };
        if d < 1 || toks[d].tok != Tok::Dot || !matches!(&toks[d - 1].tok, Tok::Word(w) if w == "inline") { continue; }
        // the code's braces, in the text: the first `{` outside brackets after the language
        let mut p = toks[i].end;
        let mut depth = 0;
        while p < chars.len() {
            match chars[p] { '(' | '[' => depth += 1, ')' | ']' => depth -= 1, '{' if depth == 0 => break, ';' if depth == 0 => { p = chars.len(); break; } _ => {} }
            p += 1;
        }
        if p >= chars.len() { continue; }
        let open = p;
        let mut q = p + 1;
        let mut dd = 1;
        while q < chars.len() { match chars[q] { '{' => dd += 1, '}' => { dd -= 1; if dd == 0 { break; } } _ => {} } q += 1; }
        if q >= chars.len() { continue; }
        let close = q;
        // the gap before `{` goes with it
        let mut g = open;
        while g > 0 && (chars[g - 1] == ' ' || chars[g - 1] == '\t') { g -= 1; }
        edits.push(Edit { at: g, end: close + 1, text: String::new() });
        // the statement's `;`, after the bindings if there are any
        let Some(semi) = toks.iter().position(|t| t.at != usize::MAX && t.at > close && t.tok == Tok::Semi) else { continue };
        let indent = line_indent(toks[i].at);
        let code: String = chars[open + 1..close].iter().collect();
        let body = if code.contains('\n') {
            let mut lines: Vec<&str> = code.split('\n').collect();
            if lines.first().is_some_and(|l| l.trim().is_empty()) { lines.remove(0); }
            if lines.last().is_some_and(|l| l.trim().is_empty()) { lines.pop(); }
            lines.join("\n")
        } else { format!("{indent}    {}", code.trim()) };
        let end_line = format!("{indent}end of inline.{}{lang}", if marked { "%" } else { "" });
        // a comment after the `;` stays on the opening line, before the code
        edits.push(Edit { at: toks[semi].at, end: toks[semi].end, text: format!(":{}", "") });
        let line_end = { let mut e = toks[semi].end; while e < chars.len() && chars[e] != '\n' { e += 1; } e };
        edits.push(Edit { at: line_end, end: line_end, text: format!("\n{body}\n{end_line}") });
    }
}

/// The kind and name of the block a `{` opens, from its statement's tokens
fn describe(toks: &[Token], start: usize, brace: usize, text: &dyn Fn(&Token) -> String, info: Option<&Info>, outer: Option<&Open>) -> Open {
    let mut o = Open { close: String::new(), bang: false, indent: String::new(), arms: false, fields: false, members: None, handler: false, arm: false, levels: false, bare: false, list: false };
    let s: Vec<&Token> = toks[start..brace].iter().collect();
    let word = |i: usize| -> Option<&str> { s.get(i).and_then(|t| if let Tok::Word(w) = &t.tok { Some(w.as_str()) } else { None }) };
    // inside a match or an otherwise: an arm
    if let Some(out) = outer {
        if out.list { o.list = true; return o; }
        if out.levels && word(0) == Some("level") { o.list = true; return o; }
        if out.arms {
            // `name $v {` or `$msg from lend $ch {` (a `%whichever` arm)
            let name = s.iter().find_map(|t| if let Tok::Word(w) = &t.tok { Some(w.clone()) } else { None }).unwrap_or_default();
            o.close = format!("case {name}");
            if matches!(s.first().map(|t| &t.tok), Some(Tok::Var(_))) {
                // PROVISIONAL: a `%whichever` arm closes on its variable's case
                let v = if let Tok::Var(v) = &s[0].tok { v.clone() } else { String::new() };
                o.close = format!("case ${v}");
            }
            o.bang = out.handler && arm_jumps(toks, brace);
            o.arm = true;
            return o;
        }
        if let Some(coll) = &out.members {
            // a collection's member: `greet [ … ] -> T {`, `greet.loud [ … ] {`, `half_failed.as_zero {`, `versions area {`
            if word(0) == Some("versions") {
                o.close = format!("versions {}", word(1).unwrap_or(""));
                o.members = Some(coll.clone());
                return o;
            }
            if matches!(s.first().map(|t| &t.tok), Some(Tok::LBracket)) {
                // PROVISIONAL: one version of `versions`, closed `end of version`
                o.close = "version".into();
                return o;
            }
            if word(0) == Some("fn") {
                let n = match s.get(1).map(|t| &t.tok) { Some(Tok::Word(w)) => w.clone(), _ => String::new() };
                o.close = format!("{} {n}", fn_kind(info, s[0].line, toks, brace)).trim().to_string();
                return o;
            }
            // the member's path, each segment as the rewrite spells it (`round.near` is `round.nearest`)
            let mut segs: Vec<String> = Vec::new();
            let mut last_word = false;
            for t in &s { match &t.tok { Tok::Word(w) if !last_word => { let n = segs.last().and_then(|b| segment(b, w)).map(str::to_string).unwrap_or(w.clone()); segs.push(n); last_word = true; } Tok::Dot => { last_word = false; } _ => break } }
            o.close = segs.join(".");
            if coll.starts_with("handler") { o.arms = true; o.handler = true; }
            return o;
        }
    }
    // the statement's tokens at bracket depth 0
    let mut depth = 0;
    let mut top: Vec<usize> = Vec::new();
    for (i, t) in s.iter().enumerate() {
        match t.tok { Tok::LBracket | Tok::LParen => { depth += 1; continue; } Tok::RBracket | Tok::RParen => { depth -= 1; continue; } _ => {} }
        if depth == 0 { top.push(i); }
    }
    let is_w = |i: usize, w: &str| matches!(&s[i].tok, Tok::Word(x) if x == w);
    // a lambda inside a call's brackets: the nearest `fn [` back from the `{` at the brackets' own depth
    if depth > 0 {
        let mut d = 0;
        for i in (0..s.len()).rev() {
            match s[i].tok { Tok::RBracket | Tok::RParen => d += 1, Tok::LBracket | Tok::LParen => { if d == 0 { break; } d -= 1; } _ => {} }
            if d == 0 && is_w(i, "fn") { o.close = fn_kind(info, s[i].line, toks, brace).to_string(); return o; }
        }
    }
    // the last `match` or `otherwise` at depth 0 makes it one of those
    if let Some(&i) = top.iter().rev().find(|&&i| is_w(i, "match") || is_w(i, "otherwise")) {
        o.close = if is_w(i, "match") { "match".into() } else { "otherwise".into() };
        o.arms = true;
        o.handler = is_w(i, "otherwise");
        return o;
    }
    // a long lambda, `keep(lend $xs, fn [$s] -> bool { … })`: the last `fn` in the statement, before its parameters
    if let Some(&fi) = top.iter().rev().find(|&&i| is_w(i, "fn")) {
        if fi > 0 && s.get(fi + 1).is_some_and(|t| t.tok == Tok::LBracket) {
            o.close = fn_kind(info, s[fi].line, toks, brace).to_string();
            return o;
        }
    }
    // skip what comes in front: `pub`, `pub.kit`, `bare.…` marks, the check words, `%layout.C`, `#strict`
    let mut i = 0;
    loop {
        match s.get(i).map(|t| &t.tok) {
            Some(Tok::Word(w)) if w == "pub" => { i += 1; if matches!(s.get(i).map(|t| &t.tok), Some(Tok::Dot)) { i += 2; } }
            Some(Tok::Word(w)) if w == "bare" => { i += 1; while matches!(s.get(i).map(|t| &t.tok), Some(Tok::Dot)) { i += 2; } }
            Some(Tok::Word(w)) if (w == "no_check" || w == "NO_FUCKING_CHECKS" || w == "check" || w == "no_cheat") && s.len() > i + 1 => { i += 1; }
            Some(Tok::Mark) => { i += 1; }
            Some(Tok::Strict) => { i += 1; }
            // `%layout.C record Timeval`: the layout goes in front, as `public` does
            Some(Tok::Word(w)) if w == "layout" && matches!(s.get(i + 1).map(|t| &t.tok), Some(Tok::Dot)) => { i += 3; }
            // `table_bytes 1_024 fn square …` (owner, 2026-10-19)
            Some(Tok::Word(w)) if w == "table_bytes" && matches!(s.get(i + 1).map(|t| &t.tok), Some(Tok::Int(_))) => { i += 2; }
            _ => break,
        }
    }
    let rest: Vec<&Token> = s[i..].to_vec();
    let w0 = rest.first().and_then(|t| if let Tok::Word(w) = &t.tok { Some(w.clone()) } else { None }).unwrap_or_default();
    let path_after = |from: usize| -> String {
        let mut name = String::new();
        let mut j = from;
        if let Some(Tok::Text(..)) = rest.get(from).map(|t| &t.tok) { return shown(&rest[from].tok); }
        while j < rest.len() {
            match &rest[j].tok {
                Tok::Word(w) => { name.push_str(w); }
                Tok::Op(op) if name.is_empty() => { name.push_str(operator_word(op).filter(|_| op.len() == 1).unwrap_or(op)); break; }
                _ => break,
            }
            if matches!(rest.get(j + 1).map(|t| &t.tok), Some(Tok::Dot)) && matches!(rest.get(j + 2).map(|t| &t.tok), Some(Tok::Word(_))) { name.push('.'); j += 2; } else { break; }
        }
        name
    };
    let layout_c = s.iter().any(|t| matches!(&t.tok, Tok::Word(w) if w == "layout"));
    match w0.as_str() {
        "fn" => {
            let line = rest[0].line;
            let kind = fn_kind(info, line, toks, brace);
            let name = path_after(1);
            o.close = if name.is_empty() { kind.to_string() } else { format!("{kind} {name}") };
        }
        "test" => { o.close = format!("test procedure {}", path_after(2)); }
        "op" => { o.close = format!("operator {}", path_after(1)); }
        "struct" => { o.close = format!("record {}", path_after(1)); o.fields = true; }
        "union" => { o.close = format!("union {}", path_after(1)); o.fields = true; }
        "enum" => { o.close = format!("{} {}", if layout_c { "enumeration" } else { "union.tagged" }, path_after(1)); o.fields = true; }
        "interface" => { o.close = format!("interface {}", path_after(1)); o.fields = true; }
        "module" => { o.close = format!("module {}", path_after(1)); }
        "coll" => {
            let kind = match rest.get(2).map(|t| &t.tok) { Some(Tok::Word(k)) => k.clone(), _ => String::new() };
            let name = path_after(3);
            match kind.as_str() {
                "fn" => {
                    let line = rest[0].line;
                    let k = coll_kind(info, line, toks, brace);
                    o.close = format!("collection.{k} {name}");
                    o.members = Some(format!("fn {name}"));
                }
                "handler" => { o.close = format!("collection.handler {name}"); o.members = Some(format!("handler {name}")); }
                "bound" => { o.close = format!("collection.bounded {name}"); }
                other => { o.close = format!("collection.{other} {name}"); }
            }
        }
        "if" => o.close = "if".into(),
        "else" => {
            if rest.get(1).is_some_and(|t| matches!(&t.tok, Tok::Word(w) if w == "if")) { o.close = "else if".into(); }
            else if rest.get(1).is_some_and(|t| matches!(&t.tok, Tok::Word(w) if w == "on")) { o.close = format!("else {}", path_after(1)); }
            else { o.close = "else".into(); }
        }
        "while" | "loop" | "for" => o.close = w0.clone(),
        "MAIN" => { o.close = "MAIN".into(); o.bang = true; }
        "concur" => {
            let detached = rest.get(1).is_some_and(|t| t.tok == Tok::Dot);
            o.close = if detached { "%spawn.task.detached".into() } else { "%spawn.task".into() };
        }
        "whichever" => { o.close = "%whichever".into(); o.arms = true; }
        "on" => { o.close = path_after(0); }
        "precedence" => { o.close = path_after(0); o.levels = true; }
        "extern" => { o.close = "%external.C".into(); }
        "no_check" => { o.close = "no_check".into(); o.bang = true; }
        "NO_FUCKING_CHECKS" => { o.close = "NO_FUCKING_CHECKS".into(); o.bang = true; }
        "check" => { o.close = "check".into(); }
        "no_cheat" => { o.close = "no_skip".into(); }
        _ => {
            // a template's member, `[] -> $T {` under its `named` line: PROVISIONAL, `end of named`, which the generator writes
            // as the member's full name
            if matches!(rest.first().map(|t| &t.tok), Some(Tok::LBracket)) && start >= 1 && toks[..start].iter().rev().take_while(|t| t.tok != Tok::LBrace).any(|t| matches!(&t.tok, Tok::Word(w) if w == "named")) {
                o.close = "named".into();
                return o;
            }
            // a bare `{ … }`: PROVISIONAL, `block:` … `end of block`
            o.close = if rest.is_empty() || matches!(&rest[0].tok, Tok::Word(b) if b == "block") { "block".into() } else { text(rest[0]) };
            o.bare = rest.is_empty();
        }
    }
    o.close = o.close.trim().to_string();
    o
}

/// `function` or `procedure` for the `fn` on `line`; unknown, a body that says an effect word is a procedure
fn fn_kind(info: Option<&Info>, line: usize, toks: &[Token], brace: usize) -> &'static str {
    if let Some(i) = info {
        if i.procedures.contains(&line) { return "procedure"; }
        if i.functions.contains(&line) { return "function"; }
    }
    if body_has_effect_word(toks, brace) { "procedure" } else { "function" }
}

/// A collection of functions is a `collection.procedure` when any member has an effect
fn coll_kind(info: Option<&Info>, _line: usize, toks: &[Token], brace: usize) -> &'static str {
    let _ = info;
    if body_has_effect_word(toks, brace) { "procedure" } else { "function" }
}

/// A body, from its `{` to the matching `}`, naming a print, an input, an exit, a wait, a task, a channel, C, a pointer
/// or another language: the fallback when the file could not be analysed
fn body_has_effect_word(toks: &[Token], brace: usize) -> bool {
    let mut d = 0;
    let mut j = brace;
    while j < toks.len() {
        match &toks[j].tok {
            Tok::LBrace => d += 1,
            Tok::RBrace => { d -= 1; if d == 0 { break; } }
            Tok::Word(w) if matches!(w.as_str(), "print" | "input" | "exit" | "time" | "concur" | "channel" | "whichever" | "inline" | "ptr" | "extern") => return true,
            Tok::Raw(_) => return true,
            Tok::Op(o) if o == "!" => {}
            _ => {}
        }
        if let Tok::Word(w) = &toks[j].tok { if w == "lent" && toks.get(j + 1).is_some_and(|t| t.tok == Tok::Dot) && toks.get(j + 2).is_some_and(|t| matches!(&t.tok, Tok::Word(m) if m == "mut")) { return true; } }
        j += 1;
    }
    // a `lent.mut` parameter, before the `{`
    let mut b = brace;
    while b > 0 && !matches!(toks[b - 1].tok, Tok::Semi | Tok::RBrace | Tok::LBrace) {
        b -= 1;
        if matches!(&toks[b].tok, Tok::Word(w) if w == "lent") && toks.get(b + 2).is_some_and(|t| matches!(&t.tok, Tok::Word(m) if m == "mut")) { return true; }
    }
    false
}

/// A handler arm that jumps, holding a `continue`, a `break` or a `return` outside any loop of its own (owner,
/// 2026-10-26: "`!` means 'needs your attention', it would also apply to … a handler holding a `continue`")
fn arm_jumps(toks: &[Token], brace: usize) -> bool {
    let mut d = 0;
    let mut loops: Vec<i32> = Vec::new();
    let mut j = brace;
    let mut pending_loop = false;
    while j < toks.len() {
        match &toks[j].tok {
            Tok::LBrace => { d += 1; if pending_loop { loops.push(d); pending_loop = false; } }
            Tok::RBrace => { if loops.last() == Some(&d) { loops.pop(); } d -= 1; if d == 0 { break; } }
            Tok::Word(w) if w == "for" || w == "while" || w == "loop" => pending_loop = true,
            Tok::Word(w) if w == "return" => return true,
            Tok::Word(w) if (w == "break" || w == "continue") && loops.is_empty() => return true,
            _ => {}
        }
        j += 1;
    }
    false
}

/// The handlers of this file that jump: `half_failed.as_zero` for a `coll.handler` member, `half_failed` for its own
fn jumping_handlers(toks: &[Token]) -> HashSet<String> {
    let mut out = HashSet::new();
    let w = |i: usize| -> Option<&str> { toks.get(i).and_then(|t| if let Tok::Word(x) = &t.tok { Some(x.as_str()) } else { None }) };
    let mut i = 0;
    while i + 4 < toks.len() {
        if w(i) == Some("coll") && toks[i + 1].tok == Tok::Dot && w(i + 2) == Some("handler") && toks[i + 4].tok == Tok::LBrace {
            let coll = w(i + 3).unwrap_or("").to_string();
            let mut j = i + 5;
            let mut d = 1;
            // each member: its path, then its `{`, then its arms
            let mut name = String::new();
            while j < toks.len() && d > 0 {
                match &toks[j].tok {
                    Tok::Word(x) if d == 1 && name.is_empty() => name = x.clone(),
                    Tok::Dot if d == 1 && !name.is_empty() => { if let Some(x) = w(j + 1) { name.push('.'); name.push_str(x); j += 1; } }
                    Tok::LBracket if d == 1 => { let mut b = 0; while j < toks.len() { match toks[j].tok { Tok::LBracket => b += 1, Tok::RBracket => { b -= 1; if b == 0 { break; } } _ => {} } j += 1; } }
                    Tok::LBrace if d == 1 => {
                        // the member's arms: any that jumps makes it a jumping handler
                        let mut k = j + 1;
                        let mut dd = 1;
                        let mut jumps = false;
                        while k < toks.len() && dd > 0 {
                            match toks[k].tok { Tok::LBrace => { if dd == 1 && arm_jumps(toks, k) { jumps = true; } dd += 1; } Tok::RBrace => dd -= 1, _ => {} }
                            k += 1;
                        }
                        if jumps { out.insert(if name == coll { coll.clone() } else { name.clone() }); }
                        name.clear();
                        j = k;
                        continue;
                    }
                    Tok::LBrace => d += 1,
                    Tok::RBrace => d -= 1,
                    _ => {}
                }
                j += 1;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

/// The word-by-word renames, everywhere outside comments and texts
fn words(toks: &[Token], text: &dyn Fn(&Token) -> String, info: Option<&Info>, edits: &mut Vec<Edit>, chars: &[char]) -> Result<(), String> {
    let n = toks.len();
    let tok = |i: usize| -> &Tok { toks.get(i).map(|t| &t.tok).unwrap_or(&Tok::Eof) };
    let is_word = |i: usize, w: &str| matches!(tok(i), Tok::Word(x) if x == w);
    let mut depth: i32 = 0;
    let jumping = jumping_handlers(toks);
    for i in 0..n {
        let t = &toks[i];
        // a handler that jumps, where it is used: `otherwise half_failed.!as_zero`, `otherwise !half_failed(…)` (owner, 2026-10-26)
        if i > 0 && is_word(i - 1, "otherwise") {
            if let Tok::Word(c) = &t.tok {
                if *tok(i + 1) == Tok::Dot {
                    if let Tok::Word(m) = tok(i + 2) { if jumping.contains(&format!("{c}.{m}")) { let a = toks[i + 2].at; edits.push(Edit { at: a, end: a, text: "!".into() }); } }
                } else if jumping.contains(c) { edits.push(Edit { at: t.at, end: t.at, text: "!".into() }); }
            }
        }
        match t.tok { Tok::LBracket | Tok::LParen => depth += 1, Tok::RBracket | Tok::RParen => depth -= 1, _ => {} }
        if t.at == usize::MAX || t.end == usize::MAX { continue; }
        // an old spelling the lexer names stays as written: a test may hold it on purpose
        if t.mtm.is_some() { continue; }
        let written = text(t);
        // the token before, a limb's mark passed over: `to.%d64` is `d64` after `to.`
        let marked = i > 0 && *tok(i - 1) == Tok::Mark;
        let pi = if marked { i.saturating_sub(2) } else { i.saturating_sub(1) };
        let prev = if i > usize::from(marked) { tok(pi) } else { &Tok::Eof };
        let after_dot = *prev == Tok::Dot;
        let after_kit = *prev == Tok::ColonColon;
        let before = if after_dot && pi >= 1 { if let Tok::Word(b) = tok(pi - 1) { b.clone() } else { String::new() } } else { String::new() };
        let put = |edits: &mut Vec<Edit>, s: String| edits.push(Edit { at: t.at, end: t.end, text: s });
        match &t.tok {
            Tok::Var(v) => {
                // a quoted name becomes `$name`, or `$'odd name'` when it needs its quotes (owner, 2026-10-27)
                if written.starts_with('\'') {
                    let plain = v.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') && v.chars().all(|c| c.is_alphanumeric() || c == '_');
                    put(edits, if plain { format!("${v}") } else { format!("${written}") });
                }
            }
            Tok::Newline => put(edits, "newline".into()),
            Tok::Op(o) => {
                // `<T>` is a type's blank, not a comparison: no space after `<`
                let glued_after = toks.get(i + 1).is_some_and(|nx| nx.at == t.end);
                let glued_before = i > 0 && toks[i - 1].end == t.at;
                if (o == "<" && glued_after && written == "<") || (o == ">" && glued_before && written == ">") { continue; }
                // `on.target aarch64-macos`: a target's name, not a subtraction
                if o == "-" && statement_has(toks, i, "target") { continue; }
                if o == "-" {
                    // a leading minus: `negative 5` on a number, `negate $x` on a value
                    let unary = i == 0 || !value_end(toks, pi);
                    if unary {
                        let number = matches!(tok(i + 1), Tok::Int(_) | Tok::Bin(..) | Tok::BigInt(_) | Tok::PercentLit(..));
                        let word = if number { "negative" } else { "negate" };
                        let glued = toks.get(i + 1).is_some_and(|nx| nx.at == t.end);
                        put(edits, if glued { format!("{word} ") } else { word.to_string() });
                        continue;
                    }
                }
                if let Some(w) = operator_word(o) {
                    if written == *o {
                        // spaced, whatever the symbol's spacing was
                        let before_gap = if i > 0 && toks[i - 1].end == t.at { " " } else { "" };
                        let after_gap = if toks.get(i + 1).is_some_and(|nx| nx.at == t.end) { " " } else { "" };
                        put(edits, format!("{before_gap}{w}{after_gap}"));
                    }
                    continue;
                }
                if o == "not_equal" { put(edits, "not_equal_to".into()); }
            }
            Tok::Word(w) => {
                // the word after an old symbol, `*const`, `*mut`, `&mut`, stays, so the old spelling is still refused by name
                if i > 0 && matches!(&toks[i - 1].tok, Tok::Op(o) if o == "*" || o == "&") && toks[i - 1].end == t.at { continue; }
                // a label inside brackets: `min:` → `minimum:`
                if depth > 0 && *tok(i + 1) == Tok::Colon && !after_dot {
                    if let Some(l) = label(w) { put(edits, l.into()); continue; }
                }
                let std_kit = pi >= 1 && matches!(tok(pi - 1), Tok::Word(k) if k == "std");
                if after_kit && !std_kit { continue; }
                if after_kit {
                    // std's path: its own names, `std::str.split`, `std::pow`
                    let r = match w.as_str() { "str" => Some("string".to_string()), "pow" => Some("power".into()), "xx" => Some("power".into()), "shl" => Some("shift_left".into()), "shr" => Some("shift_right".into()), "div" => Some("divide".into()), "mod" => Some("modulo".into()), "printu" | "println" | "printf" => None, _ => None };
                    if let Some(r) = r { put(edits, r); }
                    continue;
                }
                if after_dot {
                    // a number type in a path, `to.%d64`, `to.stored.d64`, `not_std.i24`: its mark goes inside it
                    if let Some(nt) = number_type(w) {
                        let s = if marked { marked_number_type(w).unwrap_or(nt.clone()) } else { nt.clone() };
                        if marked && s.contains('%') { let m = &toks[i - 1]; edits.push(Edit { at: m.at, end: m.end, text: String::new() }); }
                        put(edits, s);
                        continue;
                    }
                    if before == "coll" && w == "fn" { put(edits, coll_kind(info, t.line, toks, next_brace(toks, i)).into()); continue; }
                    if let Some(r) = segment(&before, w) { put(edits, r.into()); }
                    continue;
                }
                // the first word of a path or a bare word
                if let Some(nt) = number_type(w) {
                    let s = if marked { marked_number_type(w).unwrap_or(nt) } else { nt };
                    // a mark before the old word stays before the new one's first word: drop it there, it is in `s`
                    if marked && s.contains('%') {
                        let m = &toks[i - 1];
                        edits.push(Edit { at: m.at, end: m.end, text: String::new() });
                    }
                    put(edits, s);
                    continue;
                }
                match w.as_str() {
                    "fn" => {
                        // a declaration, a lambda or a function type
                        let kind = if *tok(i + 1) == Tok::LBracket && (matches!(prev, Tok::Colon | Tok::Arrow | Tok::Comma | Tok::LBracket) || fn_is_type(toks, i)) { "function" }
                            else if i > 0 && is_word(i - 1, "test") { "procedure" }
                            else if statement_has(toks, i, "extern") { "procedure" }
                            else {
                                let b = next_brace(toks, i);
                                if b == i { if short_has_effect(toks, i) { "procedure" } else { "function" } } else { fn_kind(info, t.line, toks, b) }
                            };
                        put(edits, kind.into());
                    }
                    "enum" => {
                        let layout = (i.saturating_sub(4)..i).any(|j| is_word(j, "layout"));
                        put(edits, if layout { "enumeration".into() } else { "union.tagged".into() });
                    }
                    "let" => {
                        // `let mut $x` → `variable $x`; `let immut $x` → `let $x`; a plain `let` by what changes it
                        let ns = *tok(i + 1) == Tok::NoShadow;
                        let m_at = if ns { i + 2 } else { i + 1 };
                        let flagged = toks.get(m_at).is_some_and(|x| x.mtm.is_some());
                        let (skip, name_at) = if (is_word(m_at, "mut") || is_word(m_at, "immut")) && !flagged { (true, m_at + 1) } else { (false, m_at) };
                        let name = if let Tok::Var(v) = tok(name_at) { v.clone() } else { String::new() };
                        let mutable = is_word(m_at, "mut") || info.map_or_else(|| changed_in_text(toks, i, &name), |inf| inf.changed.contains(&(t.line, name.clone())) || (!inf.declared.contains(&(t.line, name.clone())) && changed_in_text(toks, i, &name)));
                        let word = if mutable && !is_word(m_at, "immut") { "variable" } else { "let" };
                        if skip {
                            // the word, then `no_shadow` if written, then the name
                            edits.push(Edit { at: t.at, end: t.end, text: word.into() });
                            let m = &toks[m_at];
                            let gap_end = toks.get(m_at + 1).map_or(m.end, |x| x.at);
                            edits.push(Edit { at: m.at, end: gap_end, text: String::new() });
                        } else {
                            put(edits, word.into());
                        }
                    }
                    // a parameter: `[mut $n: i32]` → `[variable $n: …]`, `[immut $n: i32]` → `[$n: …]` (PROVISIONAL)
                    "mut" if matches!(prev, Tok::LBracket | Tok::Comma) && matches!(tok(i + 1), Tok::Var(_)) => put(edits, "variable".into()),
                    "immut" if matches!(prev, Tok::LBracket | Tok::Comma) && matches!(tok(i + 1), Tok::Var(_)) => {
                        let gap = toks[i + 1].at;
                        edits.push(Edit { at: t.at, end: gap, text: String::new() });
                    }
                    // `shadow mut $x` → `shadow variable $x`, `shadow immut $x` → `shadow $x`, a plain one by what changes it
                    // (PROVISIONAL: a shadow never changes unless it says `variable`, as `let` / `variable`)
                    "shadow" => {
                        let ns = *tok(i + 1) == Tok::NoShadow;
                        let m_at = if ns { i + 2 } else { i + 1 };
                        let flagged = toks.get(m_at).is_some_and(|x| x.mtm.is_some());
                        if is_word(m_at, "mut") && !flagged {
                            let m = &toks[m_at];
                            edits.push(Edit { at: m.at, end: m.end, text: "variable".into() });
                        } else if is_word(m_at, "immut") && !flagged {
                            let m = &toks[m_at];
                            let gap_end = toks.get(m_at + 1).map_or(m.end, |x| x.at);
                            edits.push(Edit { at: m.at, end: gap_end, text: String::new() });
                        } else if let Tok::Var(name) = tok(m_at) {
                            let changed = info.map_or_else(|| changed_in_text(toks, i, name), |inf| inf.changed.contains(&(t.line, name.clone())) || (!inf.declared.contains(&(t.line, name.clone())) && changed_in_text(toks, i, name)));
                            if changed { let a = toks[m_at].at; edits.push(Edit { at: a, end: a, text: "variable ".into() }); }
                        }
                    }
                    "immut" if matches!(tok(i + 1), Tok::Word(_)) => {
                        // PROVISIONAL: a field that never changes, `immut number: i32`, is `let number: …`
                        put(edits, "let".into());
                    }
                    "mut" if statement_has(toks, i, "extern") && matches!(tok(i + 1), Tok::Var(_)) => put(edits, "variable".into()),
                    _ if in_kind_list(toks, i) => {
                        let k = match w.as_str() { "fn" => "function", "struct" => "record", "enum" => "union.tagged", "const" => "constant", "var" => "variable", "op" => "operator", _ => continue };
                        put(edits, k.into());
                    }
                    "MAIN" if !after_dot && *prev != Tok::ColonColon && !matches!(prev, Tok::Word(f) if f == "fn") && next_is_brace_or_args(toks, i) => put(edits, "!MAIN".into()),
                    "no_check" | "NO_FUCKING_CHECKS" => put(edits, format!("!{w}")),
                    "coll" if *tok(i + 1) == Tok::Dot => put(edits, "collection".into()),
                    "incl" if *tok(i + 1) == Tok::Dot => put(edits, "include".into()),
                    "concur" => {
                        if *tok(i + 1) == Tok::Dot && is_word(i + 2, "detached") { put(edits, "spawn.task".into()); } else { put(edits, "spawn.task".into()); }
                    }
                    _ => {
                        // a unit after a number: `40 graphemes`, `1500 millis` (PROVISIONAL: `millis` and the like are
                        // abbreviations too)
                        if matches!(prev, Tok::Int(_)) {
                            let u = match w.as_str() { "graphemes" => Some("grapheme_clusters"), "points" => Some("code_points"), "millis" => Some("milliseconds"), "micros" => Some("microseconds"), "nanos" => Some("nanoseconds"), "secs" => Some("seconds"), "mins" => Some("minutes"), "cols" => Some("columns"),
                                "sec" => Some("second"), "min" => Some("minute"), "milli" => Some("millisecond"), "micro" => Some("microsecond"), "nano" => Some("nanosecond"), _ => None };
                            if let Some(u) = u { put(edits, u.into()); continue; }
                        }
                        if matches!(w.as_str(), "secs" | "mins" | "cols") { continue; }
                        if w == "pos" && !value_start(toks.get(i + 1).map(|x| &x.tok)) { continue; }
                        // a `for`'s modifiers: `for seq perm $i …`, `for %para $i …`; elsewhere a name
                        if matches!(w.as_str(), "seq" | "para" | "temp" | "perm") && !for_modifier(toks, i) { continue; }
                        if let Some(r) = keyword(w) {
                            // `x` and `xx` only as operators, between two values
                            if (w == "x" || w == "xx") && !between_values(toks, i) { continue; }
                            // `pos 2` in a slice, `secs` after a number
                            put(edits, r.into());
                        }
                    }
                }
            }
            Tok::Int(_) if written.starts_with("bin") || written.starts_with("hex") || written.starts_with("oct") => {
                let word = match &written[..3] { "bin" => "binary", "hex" => "hexadecimal", _ => "octal" };
                put(edits, format!("{word}{}", &written[3..]));
            }
            Tok::Text(_, raw) => {
                // a format's text: `\n` → `{newline}`, `\t` → `{tab}`, `{newln}` → `{newline}`, the hole words spelled out
                if is_format_text(toks, i) {
                    let s = format_text(raw);
                    if s != *raw { put(edits, format!("\"{s}\"")); }
                }
            }
            _ => {}
        }
    }
    let _ = chars;
    Ok(())
}

/// A word among a `for`'s modifiers, between `for` and its variable
fn for_modifier(toks: &[Token], i: usize) -> bool {
    let mut j = i;
    while j > 0 {
        j -= 1;
        match &toks[j].tok {
            Tok::Word(w) if w == "for" => return true,
            Tok::Word(w) if matches!(w.as_str(), "seq" | "para" | "temp" | "perm" | "sequential" | "parallel" | "temporary" | "permanent") => continue,
            Tok::Mark => continue,
            _ => return false,
        }
    }
    false
}

/// `fn [ … ] -> R` with neither a body (`{`) nor a short body (`=`, `with`) after it: a function's type
fn fn_is_type(toks: &[Token], i: usize) -> bool {
    let mut d = 0;
    let mut j = i + 1;
    while j < toks.len() {
        match toks[j].tok { Tok::LBracket => d += 1, Tok::RBracket => { d -= 1; if d == 0 { break; } } _ => {} }
        j += 1;
    }
    // past the return type: up to what follows it at this depth
    let mut k = j + 1;
    let mut dd = 0;
    while k < toks.len() {
        match &toks[k].tok {
            Tok::LBracket | Tok::LParen => dd += 1,
            Tok::RBracket | Tok::RParen if dd == 0 => return true,
            Tok::RBracket | Tok::RParen => dd -= 1,
            Tok::LBrace if dd == 0 => return false,
            Tok::Eq if dd == 0 => return false,
            Tok::Word(w) if dd == 0 && w == "with" => return false,
            Tok::Semi | Tok::Comma if dd == 0 => return true,
            Tok::Word(w) if dd == 0 && w == "fn" => return true,
            _ => {}
        }
        if toks[k].line > toks[i].line && dd == 0 && !matches!(toks[k - 1].tok, Tok::Arrow | Tok::Comma | Tok::LBracket) { return true; }
        k += 1;
    }
    true
}

/// A word inside the kinds of a `case` or `bare` mark, `case.any [fn, var];`
fn in_kind_list(toks: &[Token], i: usize) -> bool {
    let mut j = i;
    while j > 0 && matches!(toks[j - 1].tok, Tok::Word(_) | Tok::Comma | Tok::Dot) { j -= 1; }
    if j == 0 || toks[j - 1].tok != Tok::LBracket { return false; }
    // the path before `[`
    let mut h = j - 1;
    while h >= 2 && toks[h - 1].tok != Tok::Dot && false { h -= 1; }
    let mut words = Vec::new();
    let mut k = j - 1;
    while k >= 1 && matches!(toks[k - 1].tok, Tok::Word(_) | Tok::Dot) { k -= 1; if let Tok::Word(w) = &toks[k].tok { words.push(w.clone()); } }
    let _ = h;
    words.last().is_some_and(|w| w == "case" || w == "bare")
}

/// A word earlier in the statement around token `i`, or before the block it sits in (`%extern.C { fn f […]; }`)
fn statement_has(toks: &[Token], i: usize, w: &str) -> bool {
    let mut j = i;
    while j > 0 && !matches!(toks[j - 1].tok, Tok::Semi | Tok::LBrace | Tok::RBrace) {
        j -= 1;
        if matches!(&toks[j].tok, Tok::Word(x) if x == w) { return true; }
    }
    // the block around: its opening `{`, then the words before it on its line
    let mut d = 0;
    let mut k = j;
    while k > 0 {
        k -= 1;
        match toks[k].tok { Tok::RBrace => d += 1, Tok::LBrace => { if d == 0 { break; } d -= 1; } _ => {} }
    }
    if k < j && toks[k].tok == Tok::LBrace {
        let line = toks[k].line;
        let mut b = k;
        while b > 0 && toks[b - 1].line == line { b -= 1; if matches!(&toks[b].tok, Tok::Word(x) if x == w) { return true; } }
    }
    false
}

/// A short `fn` (a lambda's `= …`, or a signature): an effect word before its `;` or its end
fn short_has_effect(toks: &[Token], i: usize) -> bool {
    let mut d = 0;
    for t in &toks[i + 1..] {
        match &t.tok {
            Tok::LBracket | Tok::LParen => d += 1,
            Tok::RBracket | Tok::RParen => { d -= 1; if d < 0 { return false; } }
            Tok::Semi | Tok::Comma if d == 0 => return false,
            Tok::Word(w) if matches!(w.as_str(), "print" | "input" | "exit" | "time" | "channel" | "ptr") => return true,
            _ => {}
        }
    }
    false
}

/// The `{` after token `i` at depth 0 (a `fn`'s body), or `i` when there is none before a `;`
fn next_brace(toks: &[Token], i: usize) -> usize {
    let mut d = 0;
    for j in i..toks.len() {
        match toks[j].tok { Tok::LBracket | Tok::LParen => d += 1, Tok::RBracket | Tok::RParen => d -= 1, Tok::LBrace if d == 0 => return j, Tok::Semi if d == 0 => return i, _ => {} }
    }
    i
}

fn next_is_brace_or_args(toks: &[Token], i: usize) -> bool {
    matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::LBrace) | Some(Tok::LBracket))
}

/// The token at `p` ends a value, so a `-` after it is a subtraction: a variable, a number, a text, a closing bracket, or a
/// word that is a name (not an operator word, a keyword, nor a narrowing of one: `div.floor`)
fn value_end(toks: &[Token], p: usize) -> bool {
    match &toks[p].tok {
        Tok::Var(_) | Tok::Int(_) | Tok::BigInt(_) | Tok::Bin(..) | Tok::Text(..) | Tok::Char(_) | Tok::RParen | Tok::RBracket | Tok::PercentLit(..) | Tok::Newline => true,
        Tok::Word(_) => {
            // the path's first word
            let mut h = p;
            while h >= 2 && toks[h - 1].tok == Tok::Dot && matches!(toks[h - 2].tok, Tok::Word(_)) { h -= 2; }
            let Tok::Word(head) = &toks[h].tok else { return true };
            !matches!(head.as_str(), "div" | "mod" | "plus" | "minus" | "times" | "x" | "xx" | "pow" | "shl" | "shr" | "and" | "or" | "xor" | "not" | "concat" | "return" | "is" | "otherwise" | "in" | "to" | "from" | "by" | "before" | "exactly" | "negate" | "equal_to" | "at_most" | "at_least" | "if" | "while" | "let" | "change" | "give" | "copy" | "lend" | "fail")
        }
        Tok::Op(o) => matches!(o.as_str(), "?"),
        _ => false,
    }
}

/// `x` / `xx` between two values: after a value and before one
fn between_values(toks: &[Token], i: usize) -> bool {
    // after a value: a variable, a number, a closing bracket, `true` / `false`, or a field (a word after `.`)
    let after_value = i > 0 && (matches!(toks[i - 1].tok, Tok::Var(_) | Tok::Int(_) | Tok::BigInt(_) | Tok::Bin(..) | Tok::RParen | Tok::RBracket | Tok::PercentLit(..))
        || matches!(&toks[i - 1].tok, Tok::Word(w) if w == "true" || w == "false")
        || (matches!(toks[i - 1].tok, Tok::Word(_)) && i >= 2 && toks[i - 2].tok == Tok::Dot));
    after_value && toks.get(i + 1).is_some_and(|t| !matches!(t.tok, Tok::Semi | Tok::Comma | Tok::RBracket | Tok::RParen | Tok::Colon | Tok::Eq | Tok::Walrus))
}

/// Something in the file changes `$name` after its `let` (the fallback when the file could not be analysed)
fn changed_in_text(toks: &[Token], from: usize, name: &str) -> bool {
    for j in from + 1..toks.len() {
        if let Tok::Var(v) = &toks[j].tok {
            if v != name { continue; }
            let p = &toks[j - 1].tok;
            if matches!(p, Tok::Word(w) if w == "change") { return true; }
            if j >= 3 && matches!(&toks[j - 1].tok, Tok::Word(m) if m == "mut") && toks[j - 2].tok == Tok::Dot && matches!(&toks[j - 3].tok, Tok::Word(l) if l == "lend") { return true; }
        }
    }
    false
}

/// The first argument of `print.format` / `std::format` / `str.format`
fn is_format_text(toks: &[Token], i: usize) -> bool {
    // back to the bracket that opens this argument list, passing a writer before the text (`lend.mut $out,`)
    let mut d = 0;
    let mut j = i;
    while j > 0 {
        j -= 1;
        match &toks[j].tok {
            Tok::RParen | Tok::RBracket => d += 1,
            Tok::LParen | Tok::LBracket if d > 0 => d -= 1,
            // `print.format(`, `std::format(`, `print.format.stderr(`
            Tok::LParen | Tok::LBracket => return j >= 1 && (matches!(&toks[j - 1].tok, Tok::Word(w) if w == "format")
                || (j >= 3 && matches!(&toks[j - 1].tok, Tok::Word(w) if w == "stderr" || w == "stdout") && toks[j - 2].tok == Tok::Dot && matches!(&toks[j - 3].tok, Tok::Word(f) if f == "format"))),
            Tok::Text(..) if d == 0 => return false,
            Tok::Semi | Tok::LBrace | Tok::RBrace => return false,
            _ => {}
        }
    }
    false
}

/// A format's text in the new spellings: the escapes `\n` / `\t` and the `{newln}` hole become `{newline}` / `{tab}`
/// (owner, 2026-10-26), and the shapes' words are spelled out (owner, 2026-10-27)
fn format_text(raw: &str) -> String {
    let mut out = String::new();
    let cs: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '\\' && i + 1 < cs.len() {
            match cs[i + 1] {
                'n' => { out.push_str("{newline}"); i += 2; continue; }
                't' => { out.push_str("{tab}"); i += 2; continue; }
                d => { out.push(c); out.push(d); i += 2; continue; }
            }
        }
        if c == '{' {
            if let Some(e) = cs[i..].iter().position(|x| *x == '}') {
                let inner: String = cs[i + 1..i + e].iter().collect();
                out.push('{');
                out.push_str(&hole_words(&inner));
                out.push('}');
                i += e + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

fn hole_words(inner: &str) -> String {
    let mut s = String::new();
    let mut word = String::new();
    let flush = |word: &mut String, s: &mut String| {
        let r = match word.as_str() { "newln" => "newline", "cols" => "columns", "hex" => "hexadecimal", "bin" => "binary", "oct" => "octal", "pos" => "position", "graphemes" => "grapheme_clusters", "points" => "code_points", "min" => "minimum", "max" => "maximum", w => w };
        s.push_str(r);
        word.clear();
    };
    for c in inner.chars() {
        if c.is_alphanumeric() || c == '_' { word.push(c); } else { flush(&mut word, &mut s); s.push(c); }
    }
    flush(&mut word, &mut s);
    s
}

/// The edits applied, from the end of the file back; two that overlap are an error
fn apply(chars: &[char], mut edits: Vec<Edit>) -> Result<String, String> {
    edits.sort_by(|a, b| a.at.cmp(&b.at).then(a.end.cmp(&b.end)));
    // an insertion and a replacement at one place: both kept, the insertion first
    let mut out = String::new();
    let mut pos = 0;
    for e in &edits {
        if e.at < pos { return Err(format!("two changes overlap at character {}: `{}`", e.at, e.text.escape_debug())); }
        out.extend(&chars[pos..e.at]);
        out.push_str(&e.text);
        pos = e.end;
    }
    out.extend(&chars[pos..]);
    Ok(out)
}

/// Unused helpers kept for the front pass
#[allow(dead_code)]
fn _unused(_: &Handler, _: &Stmt, _: &Expr, _: &HashMap<String, String>) {}

// ------------------------------------------------------------------------------------------------------------------
// The front: the new spellings read into the tokens the parser has always read
// ------------------------------------------------------------------------------------------------------------------

thread_local! {
    /// the kind each `function` / `procedure` was written as, by its line: true for a procedure (for the `end of` checks
    /// of the file being read)
    pub static WRITTEN_KINDS: std::cell::RefCell<HashMap<usize, bool>> = std::cell::RefCell::new(HashMap::new());
    /// the same, by module and line, for every file read: what the check that a function calls only functions reads
    pub static KINDS_BY_MODULE: std::cell::RefCell<HashMap<(Vec<String>, usize), bool>> = std::cell::RefCell::new(HashMap::new());
    /// the module of the file being read
    pub static LEX_MODULE: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
}

/// A `function` has no effects and calls only functions (owner, 2026-10-27: "`function` is pure, possibly partial;
/// `procedure` may have effects"); a function value may stand where a procedure is asked, never the reverse.
pub fn check_purity(prog: &ast::Program) -> Result<(), String> {
    let kinds = KINDS_BY_MODULE.with(|k| k.borrow().clone());
    let kind_of = |f: &ast::Func| -> Option<bool> {
        let mut m = f.module.clone();
        if f.owned { m.pop(); }
        kinds.get(&(m, f.line)).copied().or_else(|| kinds.get(&(f.module.clone(), f.line)).copied())
    };
    let name_of = |f: &ast::Func| f.name.rsplit('.').next().unwrap_or(&f.name).to_string();
    // the names written as procedures, and C's
    let procedures: HashSet<String> = prog.funcs.iter().filter(|f| f.is_extern || kind_of(f) == Some(true)).map(name_of).collect();
    let functions: HashSet<String> = prog.funcs.iter().filter(|f| !f.is_extern && kind_of(f) == Some(false)).map(name_of).collect();
    for f in &prog.funcs {
        if kind_of(f) != Some(false) { continue; }
        let shown = f.name.clone();
        if f.is_extern { return Err(format!("line {}: `{shown}` is C's, reached by a `procedure`: `%external.C procedure {shown} […]` (owner, 2026-10-27)", f.line)); }
        let (effect, called) = effects_of(f);
        if effect {
            return Err(format!("line {}: `{shown}` is a `function`, which has no effects, but it prints, reads, waits, starts a task, changes what it was lent, or reaches C or another language: make it a `procedure` (owner, 2026-10-27)", f.line));
        }
        if let Some(c) = called.iter().find(|c| procedures.contains(*c) && !functions.contains(*c)) {
            return Err(format!("line {}: `{shown}` is a `function`, so it calls only functions, but `{c}` is a `procedure`: make `{shown}` a `procedure`, or `{c}` a `function` (owner, 2026-10-27)", f.line));
        }
    }
    Ok(())
}

type Fault = (usize, usize, String);

fn word_tok(w: &str, like: &Token) -> Token {
    Token { tok: Tok::Word(w.to_string()), line: like.line, col: like.col, mtm: None, paren: false, at: like.at, end: like.end }
}

fn tok_like(tok: Tok, like: &Token) -> Token {
    Token { tok, line: like.line, col: like.col, mtm: None, paren: false, at: like.at, end: like.end }
}

/// The words before 2026-10-27 that a keyword position refuses, with the word to write
fn old_word(w: &str) -> Option<String> {
    if let Some(t) = number_type(w) { return Some(t); }
    Some(match w {
        "fn" => "function` (no effects) or `procedure".into(),
        "struct" => "record".into(),
        "enum" => "union.tagged`, or for C `enumeration".into(),
        "mut" => "variable`, or for a borrow `lend.mutable".into(),
        "immut" => "let".into(),
        "var" => "variable".into(),
        "newln" => "newline".into(),
        "not_equal" => "not_equal_to".into(),
        "x" | "xx" | "concur" | "str" | "char" | "bool" => keyword(w)?.into(),
        _ => match keyword(w) { Some(k) if !matches!(w, "x" | "xx" | "pos" | "graphemes" | "points" | "secs" | "mins" | "cols") => k.into(), _ => return None },
    })
}

/// The new keywords read as the parser's (keyword position: not after `.` or `::`)
fn new_word(w: &str) -> Option<&'static str> {
    Some(match w {
        "function" | "procedure" => "fn",
        "operator" => "op",
        "record" => "struct",
        "enumeration" => "enum",
        "constant" => "const",
        "public" => "pub",
        "pointer" => "ptr",

        "external" => "extern",
        "sequential" => "seq",
        "parallel" => "para",
        "temporary" => "temp",
        "permanent" => "perm",
        "no_skip" => "no_cheat",
        "boolean" => "bool",
        "character" => "char",
        "string" => "str",
        "concatenate" => "concat",
        "divide" => "div",
        "modulo" => "mod",
        "shift_left" => "shl",
        "shift_right" => "shr",
        _ => return None,
    })
}

/// A token a value starts with, for `position 3` / `pos 3`
fn value_start(t: Option<&Tok>) -> bool {
    matches!(t, Some(Tok::Int(_) | Tok::BigInt(_) | Tok::Var(_) | Tok::LParen)) || matches!(t, Some(Tok::Word(w)) if w == "negative" || w == "my")
}

/// A word after `.`, read by the segment before it (the parser's spelling of that segment)
fn new_segment(before: &str, w: &str) -> Option<&'static str> {
    Some(match (before, w) {
        ("coll", "function" | "procedure") => "fn",
        ("coll", "bounded") => "bound",
        ("print", "shortest") | ("weird", "shortest") | ("to", "shortest") => "useful",
        ("print", "arrows") => "weird",
        (_, "auto_newline") => "auto_newln",
        ("str", "uppercase") => "upper",
        ("str", "lowercase") => "lower",
        (_, "code_point") => "point",
        (_, "code_points") => "points",
        (_, "grapheme_cluster") => "grapheme",
        (_, "grapheme_clusters") => "graphemes",
        ("map", "count") => "length",
        ("div" | "mod" | "round", "nearest") => "near",
        ("div" | "mod" | "round", "truncate") => "trunc",
        ("div" | "mod" | "round", "ceiling") => "ceil",
        ("lend" | "lent" | "ptr", m) if m == MUTABLE => "mut",
        (_, "string") => "str",
        (_, "character") => "char",
        (_, "boolean") => "bool",
        (_, "power") => "pow",
        (_, "shift_left") => "shl",
        (_, "shift_right") => "shr",
        ("to", "function" | "procedure") => return None,
        _ => return None,
    })
}

/// The old number type a new name spells: `integer.signed.32_bit` → `i32`
fn old_number(kind: &str, sub: &str, width: &str) -> Option<String> {
    let n = width.strip_suffix("_bit")?;
    let w = match (kind, sub) { ("integer", "signed") => format!("i{n}"), ("integer", "unsigned") => format!("u{n}"), ("float", "binary") => format!("f{n}"), ("float", "decimal") => format!("d{n}"), ("float", "brain") if n == "16" => "brain16".into(), _ => return None };
    number_type(&w).map(|_| w)
}

/// The labels in the new spellings, read as the parser's
fn new_label(w: &str) -> Option<&'static str> {
    Some(match w { "minimum" => "min", "maximum" => "max", "position" => "pos", "minimum_places" => "min_places", "maximum_places" => "max_places", _ => return None })
}

/// A format's text, from the new spellings to the oracle's: `{newline}` → `\n`, `{tab}` → `\t`, the holes' words
fn format_text_back(raw: &str) -> Result<String, String> {
    let cs: Vec<char> = raw.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '\\' && i + 1 < cs.len() {
            match cs[i + 1] {
                'n' => return Err("in a format, a newline is `{newline}` since 2026-10-26 (owner); `\\n` is an old spelling".into()),
                't' => return Err("in a format, a tab is `{tab}` since 2026-10-26 (owner); `\\t` is an old spelling".into()),
                d => { out.push(c); out.push(d); i += 2; continue; }
            }
        }
        if c == '{' {
            if let Some(e) = cs[i..].iter().position(|x| *x == '}') {
                let inner: String = cs[i + 1..i + e].iter().collect();
                match inner.as_str() {
                    "newline" => out.push_str("\\n"),
                    "tab" => out.push_str("\\t"),
                    "newln" => return Err("`{newln}` is an old spelling: write `{newline}` (owner, 2026-10-27)".into()),
                    _ => {
                        let mut s = String::new();
                        let mut word = String::new();
                        let flush = |word: &mut String, s: &mut String| -> Result<(), String> {
                            let r = match word.as_str() {
                                "columns" => "cols", "hexadecimal" => "hex", "binary" => "bin", "octal" => "oct", "position" => "pos",
                                "grapheme_clusters" => "graphemes", "code_points" => "points", "minimum" => "min", "maximum" => "max",
                                "cols" | "hex" | "bin" | "oct" | "graphemes" | "points" if !word.is_empty() => return Err(format!("in a format hole, `{word}` is an old spelling: write `{}` (owner, 2026-10-27)", match word.as_str() { "cols" => "columns", "hex" => "hexadecimal", "bin" => "binary", "oct" => "octal", "graphemes" => "grapheme_clusters", _ => "code_points" })),
                                w => w,
                            };
                            s.push_str(r);
                            word.clear();
                            Ok(())
                        };
                        for ch in inner.chars() { if ch.is_alphanumeric() || ch == '_' { word.push(ch); } else { flush(&mut word, &mut s)?; s.push(ch); } }
                        flush(&mut word, &mut s)?;
                        out.push('{'); out.push_str(&s); out.push('}');
                    }
                }
                i += e + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    Ok(out)
}

/// The text a token stands for in an `end of` line
fn shown(t: &Tok) -> String {
    match t { Tok::Word(w) => w.clone(), Tok::Var(v) => format!("${v}"), Tok::Dot => ".".into(), Tok::Mark => "%".into(), Tok::Op(o) => o.clone(), Tok::Attention => "!".into(), other => format!("{other}").trim_matches('`').to_string() }
}

/// The new spellings into the parser's tokens. Refuses each old spelling by name, then reads the blocks and checks every
/// `end of` line and every `!` against the block it closes.
pub fn front(toks: Vec<Token>) -> Result<Vec<Token>, Fault> {
    WRITTEN_KINDS.with(|k| k.borrow_mut().clear());
    let toks = words_back(toks)?;
    blocks_back(toks)
}

/// Step one: the words. Each old spelling is refused, each new one read as the parser's.
fn words_back(toks: Vec<Token>) -> Result<Vec<Token>, Fault> {
    let mut out: Vec<Token> = Vec::with_capacity(toks.len());
    let n = toks.len();
    let mut i = 0;
    let is_w = |i: usize, w: &str| matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Word(x)) if x == w);
    let mut depth = 0i32;
    while i < n {
        let t = &toks[i];
        match t.tok { Tok::LBracket | Tok::LParen => depth += 1, Tok::RBracket | Tok::RParen => depth -= 1, _ => {} }
        let fault = |m: String| -> Result<Vec<Token>, Fault> { Err((t.line, t.col, m)) };
        // an `end of` line is read as written: it names the block in the new spellings, checked in step two
        let starts_line = i == 0 || toks[i - 1].line < t.line;
        let end_at = if t.tok == Tok::Attention { i + 1 } else { i };
        if (starts_line || t.tok == Tok::Attention) && matches!(toks.get(end_at).map(|x| &x.tok), Some(Tok::Word(e)) if e == "end") && matches!(toks.get(end_at + 1).map(|x| &x.tok), Some(Tok::Word(o)) if o == "of") {
            let line = toks[end_at].line;
            while i < n && toks[i].line == line && !matches!(toks[i].tok, Tok::Semi | Tok::RParen | Tok::Comma | Tok::RBracket | Tok::Eof) { out.push(toks[i].clone()); i += 1; }
            continue;
        }
        // the token before in what is read so far, a mark passed over
        let last = out.iter().rev().find(|x| x.tok != Tok::Mark).map(|x| x.tok.clone()).unwrap_or(Tok::Eof);
        let marked = out.last().is_some_and(|x| x.tok == Tok::Mark);
        let after_dot = last == Tok::Dot;
        let after_kit = last == Tok::ColonColon;
        let before: String = if after_dot { out.iter().rev().filter(|x| x.tok != Tok::Mark).nth(1).and_then(|x| if let Tok::Word(b) = &x.tok { Some(b.clone()) } else { None }).unwrap_or_default() } else { String::new() };
        // an old spelling the lexer already named is refused by the parser, naming what replaced it
        if t.mtm.is_some() { out.push(t.clone()); i += 1; continue; }
        // and the word glued to an old symbol, `*const`, `&mut`: the symbol names the whole old spelling
        if i > 0 && toks[i - 1].mtm.is_some() && matches!(&toks[i - 1].tok, Tok::Op(o) if o == "*" || o == "&") && toks[i - 1].end == t.at { out.push(t.clone()); i += 1; continue; }
        match &t.tok {
            Tok::LBrace | Tok::RBrace => return fault("`{ }` is an old spelling since 2026-10-26: a block's opening line ends in `:` and its last line is `end of` its kind and name (`if $n greater_than 0:` … `end of if`) (owner)".into()),
            // a symbol, not the word the lexer reads as it (`less_than` is `<` inside)
            Tok::Op(o) if (o == "+" || o == "<" || o == ">" || o == "-") && t.end != usize::MAX && t.end - t.at == 1 => {
                // `<T>` is a type's blank; `-` ending a line before a block's opening line joins the two (owner, 2026-10-26)
                let glued_after = toks.get(i + 1).is_some_and(|nx| nx.at == t.end && t.end != usize::MAX);
                let glued_before = i > 0 && toks[i - 1].end == t.at && t.at != usize::MAX;
                if (o == "<" && glued_after) || (o == ">" && glued_before) { out.push(t.clone()); i += 1; continue; }
                if o == "-" && toks.get(i + 1).is_some_and(|nx| nx.line > t.line) { out.push(t.clone()); i += 1; continue; }
                if o == "-" && (statement_has(&toks, i, "target") || out.iter().rev().take(6).any(|x| matches!(&x.tok, Tok::Word(w) if w == "target"))) { out.push(t.clone()); i += 1; continue; }
                let w = match o.as_str() { "+" => "plus", "<" => "less_than", ">" => "greater_than", _ => "minus` (`negative 5` for a number, `negate $x` for a value" };
                return fault(format!("`{o}` is an old spelling since 2026-10-27: write `{w}` (owner)"));
            }
            Tok::Op(o) if o == "x~" || o == "xx~" => return fault(format!("`{o}` is an old spelling: write `{}` (owner, 2026-10-27)", if o == "x~" { "times.wrap" } else { "power.wrap" })),
            Tok::Text(stored, raw) => {
                if is_format_text(&toks, i) {
                    let back = format_text_back(raw).map_err(|m| (t.line, t.col, m))?;
                    if back != *raw {
                        let stored_back = format_text_back(stored).unwrap_or_else(|_| stored.clone()).replace("\\n", "\n").replace("\\t", "\t");
                        let _ = stored_back;
                        // the stored text is the raw one with only `\"` and `\\` read: the oracle's format reads the escapes itself
                        let mut st = String::new();
                        let cs: Vec<char> = back.chars().collect();
                        let mut k = 0;
                        while k < cs.len() { if cs[k] == '\\' && k + 1 < cs.len() && (cs[k + 1] == '"' || cs[k + 1] == '\\') { st.push(cs[k + 1]); k += 2; } else { st.push(cs[k]); k += 1; } }
                        out.push(tok_like(Tok::Text(st, back), t));
                        i += 1;
                        continue;
                    }
                }
                out.push(t.clone());
            }
            Tok::Word(w) => {
                let w = w.as_str();
                // a label: `minimum:` → `min:`; the old `min:` refused
                if depth > 0 && !after_dot && toks.get(i + 1).is_some_and(|x| x.tok == Tok::Colon) {
                    if let Some(l) = new_label(w) { out.push(word_tok(l, t)); i += 1; continue; }
                    if let Some(l) = label(w) { return fault(format!("`{w}:` is an old spelling: write `{l}:` (owner, 2026-10-27)")); }
                }
                let std_kit = { let mut k = out.len(); while k > 0 && out[k - 1].tok == Tok::Mark { k -= 1; } k >= 2 && matches!(&out[k - 2].tok, Tok::Word(s) if s == "std") };
                if after_kit && !std_kit { out.push(t.clone()); i += 1; continue; }
                if after_kit {
                    let r = match w { "string" => Some("str"), "power" => Some("pow"), "divide" => Some("div"), "modulo" => Some("mod"), "shift_left" => Some("shl"), "shift_right" => Some("shr"), _ => None };
                    if let Some(r) = r { out.push(word_tok(r, t)); i += 1; continue; }
                    let old = match w { "str" => Some("string"), "pow" | "xx" => Some("power"), "div" => Some("divide"), "mod" => Some("modulo"), "shl" => Some("shift_left"), "shr" => Some("shift_right"), _ => None };
                    if let Some(o) = old { return fault(format!("`{w}` is an old spelling: write `{o}` (owner, 2026-10-27)")); }
                    out.push(t.clone()); i += 1; continue;
                }
                if after_dot {
                    // `to.integer.signed.32_bit`: a number type inside a path
                    if (w == "integer" || w == "float") && toks.get(i + 1).is_some_and(|x| x.tok == Tok::Dot) {
                        if let Some(k) = number_back(&toks, i, &mut out)? { i = k; continue; }
                    }
                    if number_type(w).is_some() { return fault(format!("`{w}` is an old spelling: write `{}` (owner, 2026-10-27)", number_type(w).unwrap())); }
                    if let Some(r) = new_segment(&before, w) { out.push(word_tok(r, t)); i += 1; continue; }
                    if let Some(r) = segment(&before, w) {
                        // the old word in a path, unless it is the new one too; the message shows the word before as written
                        let raw_before = { let mut k = i; while k > 0 && toks[k - 1].tok == Tok::Mark { k -= 1; } if k >= 2 { shown(&toks[k - 2].tok) } else { before.clone() } };
                        if r != w { return fault(format!("`{raw_before}.{w}` is an old spelling: write `{raw_before}.{r}` (owner, 2026-10-27)")); }
                    }
                    out.push(t.clone()); i += 1; continue;
                }
                // the first word of a path, or a bare word
                if (w == "integer" || w == "float") && toks.get(i + 1).is_some_and(|x| x.tok == Tok::Dot) {
                    if let Some(k) = number_back(&toks, i, &mut out)? { i = k; continue; }
                }
                // a unit after a number
                if matches!(last, Tok::Int(_)) {
                    let u = match w { "grapheme_clusters" => Some("graphemes"), "code_points" => Some("points"), "milliseconds" => Some("millis"), "microseconds" => Some("micros"), "nanoseconds" => Some("nanos"), "seconds" => Some("secs"), "minutes" => Some("mins"), "columns" => Some("cols"),
                        "second" => Some("sec"), "minute" => Some("min"), "millisecond" => Some("milli"), "microsecond" => Some("micro"), "nanosecond" => Some("nano"), _ => None };
                    if let Some(u) = u { out.push(word_tok(u, t)); i += 1; continue; }
                    let old = match w { "graphemes" => Some("grapheme_clusters"), "points" => Some("code_points"), "millis" => Some("milliseconds"), "micros" => Some("microseconds"), "nanos" => Some("nanoseconds"),
                        "secs" => Some("seconds"), "mins" => Some("minutes"), "cols" => Some("columns"), "sec" => Some("second"), "min" => Some("minute"), "milli" => Some("millisecond"), "micro" => Some("microsecond"), "nano" => Some("nanosecond"), _ => None };
                    if let Some(o) = old { return fault(format!("`{w}` is an old spelling: write `{o}` (owner, 2026-10-27)")); }
                }
                if in_kind_list(&toks, i) {
                    let k = match w { "function" | "procedure" => Some("fn"), "record" => Some("struct"), "constant" => Some("const"), "variable" => Some("var"), "operator" => Some("op"), "union" => Some("enum"), _ => None };
                    if let Some(k) = k { out.push(word_tok(k, t)); if w == "union" && is_w(i + 2, "tagged") { i += 3; } else { i += 1; } continue; }
                }
                match w {
                    "function" | "procedure" => {
                        // a declaration's kind, by its line (a function type, `function[…]`, or a lambda names none)
                        let declares = matches!(toks.get(i + 1).map(|x| &x.tok), Some(Tok::Word(_)));
                        let lambda = matches!(toks.get(i + 1).map(|x| &x.tok), Some(Tok::LBracket)) && !matches!(last, Tok::Colon | Tok::Arrow | Tok::Comma | Tok::LBracket);
                        if declares || lambda { WRITTEN_KINDS.with(|k| { k.borrow_mut().insert(t.line, w == "procedure"); }); }
                        if declares {
                            let module = LEX_MODULE.with(|m| m.borrow().clone());
                            KINDS_BY_MODULE.with(|k| { k.borrow_mut().insert((module, t.line), w == "procedure"); });
                        }
                        out.push(word_tok("fn", t));
                    }
                    // a writable C variable, `%external.C variable $x: C.int;`; a parameter that changes, `[variable $n: i32]`
                    "variable" if out.iter().rev().take(4).any(|x| matches!(&x.tok, Tok::Word(e) if e == "extern")) || matches!(out.last().map(|x| &x.tok), Some(Tok::LBracket | Tok::Comma)) => out.push(word_tok("mut", t)),
                    // `shadow $x` never changes, `shadow variable $x` does (PROVISIONAL)
                    "shadow" => {
                        out.push(t.clone());
                        let ns = toks.get(i + 1).is_some_and(|x| x.tok == Tok::NoShadow);
                        if ns { out.push(toks[i + 1].clone()); }
                        let m = if ns { i + 2 } else { i + 1 };
                        if is_w(m, "variable") { out.push(word_tok("mut", &toks[m])); i = m + 1; continue; }
                        if matches!(toks.get(m).map(|x| &x.tok), Some(Tok::Var(_))) { out.push(word_tok("immut", t)); }
                        i = m;
                        continue;
                    }
                    "variable" | "let" => {
                        let ns = toks.get(i + 1).is_some_and(|x| x.tok == Tok::NoShadow);
                        let m = if ns { i + 2 } else { i + 1 };
                        if (is_w(m, "mut") || is_w(m, "immut")) && toks[m].mtm.is_none() { let x = &toks[m]; return Err((x.line, x.col, format!("`let {}` is an old spelling: a name that changes is a `variable`, one that never does a `let` (owner, 2026-10-27)", shown(&x.tok)))); }
                        out.push(word_tok("let", t));
                        if ns { out.push(toks[i + 1].clone()); }
                        out.push(word_tok(if w == "variable" { "mut" } else { "immut" }, t));
                        i = m;
                        continue;
                    }
                    "union" if toks.get(i + 1).is_some_and(|x| x.tok == Tok::Dot) && is_w(i + 2, "tagged") => { out.push(word_tok("enum", t)); i += 3; continue; }
                    "spawn" if toks.get(i + 1).is_some_and(|x| x.tok == Tok::Dot) && is_w(i + 2, "task") => { out.push(word_tok("concur", t)); i += 3; continue; }
                    "collection" if toks.get(i + 1).is_some_and(|x| x.tok == Tok::Dot) => out.push(word_tok("coll", t)),
                    "include" if toks.get(i + 1).is_some_and(|x| x.tok == Tok::Dot) => out.push(word_tok("incl", t)),
                    "incl" if toks.get(i + 1).is_some_and(|x| x.tok == Tok::Dot) => return fault("`incl.` is an old spelling: write `include.` (owner, 2026-10-27)".into()),
                    // a bare block's word, `block:` (PROVISIONAL)
                    "block" if toks.get(i + 1).is_some_and(|x| x.tok == Tok::Colon) && toks.get(i + 2).map_or(true, |x| x.line > toks[i + 1].line) && (i == 0 || toks[i - 1].line < t.line) => {}
                    "mut" if out.iter().rev().take(4).any(|x| matches!(&x.tok, Tok::Word(e) if e == "extern")) => return fault("`mut` is an old spelling: a C variable written to is `%external.C variable $x: C.int;` (owner, 2026-10-27)".into()),
                    "negative" if matches!(toks.get(i + 1).map(|x| &x.tok), Some(Tok::Int(_) | Tok::Bin(..) | Tok::BigInt(_) | Tok::PercentLit(..))) => out.push(tok_like(Tok::Op("-".into()), t)),
                    "power" => out.push(word_tok(if toks.get(i + 1).is_some_and(|x| x.tok == Tok::Dot) { "pow" } else { "xx" }, t)),
                    "position" if value_start(toks.get(i + 1).map(|x| &x.tok)) => out.push(word_tok("pos", t)),
                    "pos" if value_start(toks.get(i + 1).map(|x| &x.tok)) => return fault("`pos` is an old spelling: write `position` (owner, 2026-10-27)".into()),
                    "MAIN" if toks.get(i + 1).is_some_and(|x| matches!(x.tok, Tok::Colon | Tok::LBracket)) && !matches!(last, Tok::Word(ref f) if f == "fn") && !matches!(out.last().map(|x| &x.tok), Some(Tok::Attention)) =>
                        return fault("`MAIN:` opens with `!`, the program's start being what needs attention: `!MAIN:` … `!end of MAIN` (owner, 2026-10-26)".into()),
                    "no_check" | "NO_FUCKING_CHECKS" if !matches!(out.last().map(|x| &x.tok), Some(Tok::Attention)) =>
                        return fault(format!("`{w}` wears `!`, unchecked code being what a reviewer looks at first: `!{w}` (owner, 2026-10-27)")),
                    _ => {
                        if matches!(w, "sequential" | "parallel" | "temporary" | "permanent" | "seq" | "para" | "temp" | "perm") && !for_modifier(&toks, i) { out.push(t.clone()); i += 1; continue; }
                        if let Some(r) = new_word(w) { out.push(word_tok(r, t)); i += 1; continue; }
                        // `x`, `xx`, `pos` are old only where they are an operator or a word of the language
                        let operator = matches!(w, "x" | "xx") && between_values(&toks, i);
                        if matches!(w, "x" | "xx") && !operator { out.push(t.clone()); i += 1; continue; }
                        if let Some(o) = old_word(w) {
                            if marked && number_type(w).is_some() {
                                return fault(format!("`%{w}` is an old spelling: write `{}` (owner, 2026-10-27)", marked_number_type(w).unwrap()));
                            }
                            return fault(format!("`{w}` is an old spelling: write `{o}` (owner, 2026-10-27)"));
                        }
                        out.push(t.clone());
                    }
                }
            }
            _ => out.push(t.clone()),
        }
        i += 1;
    }
    Ok(out)
}

/// `integer.signed.32_bit` at `i` (its first word), into `i32`, a mark where the limb's part is; the index after it
fn number_back(toks: &[Token], i: usize, out: &mut Vec<Token>) -> Result<Option<usize>, Fault> {
    // a limb's type is marked where the marks table looks for it: first in a path, or right after `to.` (`to.useful.d64`
    // was never marked; design claude's reading, kept as it was)
    let before: Option<String> = {
        let mut k = out.len();
        while k > 0 && out[k - 1].tok == Tok::Mark { k -= 1; }
        if k >= 2 && out[k - 1].tok == Tok::Dot { if let Tok::Word(b) = &out[k - 2].tok { Some(b.clone()) } else { None } } else { None }
    };
    let markable = before.as_deref().map_or(true, |b| b == "to");
    // kind . [%] sub . [%] width
    let mut j = i + 2;
    let m1 = toks.get(j).is_some_and(|t| t.tok == Tok::Mark);
    if m1 { j += 1; }
    let Some(Tok::Word(sub)) = toks.get(j).map(|t| &t.tok) else { return Ok(None) };
    let sub = sub.clone();
    if !toks.get(j + 1).is_some_and(|t| t.tok == Tok::Dot) { return Ok(None); }
    j += 2;
    let m2 = toks.get(j).is_some_and(|t| t.tok == Tok::Mark);
    if m2 { j += 1; }
    let Some(Tok::Word(width)) = toks.get(j).map(|t| &t.tok) else { return Ok(None) };
    let Tok::Word(kind) = &toks[i].tok else { return Ok(None) };
    let Some(old) = old_number(kind, &sub, width) else { return Ok(None) };
    let t = &toks[i];
    let marked_before = out.last().is_some_and(|x| x.tok == Tok::Mark);
    // where the mark belongs (PROVISIONAL, asked 2026-10-27): on the part the limb adds
    let want = marked_number_type(&old).unwrap_or_default();
    let has_mark = m1 || m2 || marked_before;
    let written = format!("{}{kind}.{}{sub}.{}{width}", if marked_before { "%" } else { "" }, if m1 { "%" } else { "" }, if m2 { "%" } else { "" });
    let needs = want.contains('%') && markable;
    let want = if markable { want } else { want.replace('%', "") };
    if has_mark && !want.contains('%') && markable { return Err((t.line, t.col, format!("`{written}`: `{want}` is the base's, so it takes no mark (owner, 2026-10-26)"))); }
    if has_mark && written != want { return Err((t.line, t.col, format!("`{written}`: the limb's mark goes on the part it adds, `{want}` (owner, 2026-10-26)"))); }
    if needs && !has_mark { return Err((t.line, t.col, format!("`{written}` is a limb's, so it is marked: `{want}` (owner, 2026-10-26)"))); }
    if marked_before { out.pop(); }
    if needs { out.push(tok_like(Tok::Mark, t)); }
    let mut w = word_tok(&old, t);
    w.end = toks[j].end;
    out.push(w);
    Ok(Some(j + 1))
}

/// Step two: the blocks. An opening line's `:` is a `{`, an `end of …` line a `}`, `case` goes from an arm's line, and
/// an inline block's code goes where it always went. Then every `end of` and every `!` is checked.
fn blocks_back(toks: Vec<Token>) -> Result<Vec<Token>, Fault> {
    let n = toks.len();
    let mut out: Vec<Token> = Vec::with_capacity(n);
    // what each `}` was written as: the `end of` text and whether it wore `!`; each arm's `!case`; each `!` at a use
    let mut closes: Vec<(String, bool, usize, usize)> = Vec::new();
    let mut arm_bangs: HashMap<usize, (bool, usize, usize)> = HashMap::new();
    let mut use_bangs: Vec<(usize, usize, usize)> = Vec::new();   // (index in out of the word after `!`, line, col)
    let mut i = 0;
    let mut depth = 0i32;
    while i < n {
        let t = &toks[i];
        match t.tok { Tok::LBracket | Tok::LParen => depth += 1, Tok::RBracket | Tok::RParen => depth -= 1, _ => {} }
        // `end of …` / `!end of …`: up to the end of the line, or a `;`, `)`, `,`
        let bang = t.tok == Tok::Attention && matches!(toks.get(i + 1).map(|x| &x.tok), Some(Tok::Word(e)) if e == "end");
        let at_end = if bang { i + 1 } else { i };
        if matches!(toks.get(at_end).map(|x| &x.tok), Some(Tok::Word(e)) if e == "end") && matches!(toks.get(at_end + 1).map(|x| &x.tok), Some(Tok::Word(o)) if o == "of") && (at_end == 0 || toks[at_end - 1].line < toks[at_end].line || bang) {
            let line = toks[at_end].line;
            let mut j = at_end + 2;
            let mut text = String::new();
            while j < n && toks[j].line == line && !matches!(toks[j].tok, Tok::Semi | Tok::RParen | Tok::Comma | Tok::RBracket | Tok::Eof) {
                // an operator as it was written: `less_than` is `<` inside
                let s = match &toks[j].tok {
                    Tok::Op(o) if toks[j].end != usize::MAX && toks[j].end - toks[j].at > o.chars().count() => match o.as_str() { "<" => "less_than".to_string(), ">" => "greater_than".into(), "not_equal" => "not_equal_to".into(), _ => o.clone() },
                    other => shown(other),
                };
                let glue = matches!(toks[j].tok, Tok::Dot) || text.ends_with('.') || text.ends_with('%') || text.ends_with('!') || text.is_empty();
                if !glue { text.push(' '); }
                text.push_str(&s);
                j += 1;
            }
            closes.push((text, bang, t.line, t.col));
            out.push(tok_like(Tok::RBrace, t));
            i = j;
            continue;
        }
        // the hyphen ending a line, before a block's opening line, joins them (owner, 2026-10-26: "See that `-`?")
        if matches!(&t.tok, Tok::Op(o) if o == "-") && toks.get(i + 1).is_some_and(|x| x.line > t.line) {
            i += 1;
            continue;
        }
        // an opening line's `:`: the last thing on its line, outside brackets
        if t.tok == Tok::Colon && toks.get(i + 1).map_or(true, |x| x.line > t.line || x.tok == Tok::Eof || matches!(x.tok, Tok::Raw(_))) {
            // an inline block: its bindings go after its code, as before, and the statement ends `;`
            if let Some(Tok::Raw(_)) = toks.get(i + 1).map(|x| &x.tok) {
                // the `with [ … ]` on the opening line, already in `out`: taken back out, put after the code
                let mut w_at = None;
                let mut d = 0;
                for (k, x) in out.iter().enumerate().rev() {
                    match x.tok { Tok::RBracket => d += 1, Tok::LBracket => d -= 1, _ => {} }
                    if d == 0 && matches!(&x.tok, Tok::Word(w) if w == "with") { w_at = Some(k); break; }
                    if d == 0 && matches!(&x.tok, Tok::Word(w) if w == "ASM" || w == "MSL" || w == "S" || w == "Assembly" || w == "Metal_Shading_Language") { break; }
                }
                let binds: Vec<Token> = match w_at { Some(k) => out.drain(k..).collect(), None => Vec::new() };
                out.push(toks[i + 1].clone());
                out.extend(binds);
                // the `end of inline.…` line
                let mut j = i + 2;
                let (bang2, e) = if toks.get(j).is_some_and(|x| x.tok == Tok::Attention) { (true, j + 1) } else { (false, j) };
                if !matches!(toks.get(e).map(|x| &x.tok), Some(Tok::Word(w)) if w == "end") { return Err((t.line, t.col, "an inline block ends with its `end of` line".into())); }
                let line = toks[e].line;
                j = e;
                while j < n && toks[j].line == line && !matches!(toks[j].tok, Tok::Semi | Tok::Eof) { j += 1; }
                if bang2 { return Err((toks[e].line, toks[e].col, "an inline block's `end of` line takes no `!`: the `!` is on `!no_check` (owner, 2026-10-27)".into())); }
                out.push(tok_like(Tok::Semi, &toks[e]));
                i = j;
                if toks.get(i).is_some_and(|x| x.tok == Tok::Semi) { i += 1; }
                continue;
            }
            out.push(tok_like(Tok::LBrace, t));
            i += 1;
            continue;
        }
        // an arm's `case` / `!case`, at the start of its line
        let starts_line = i == 0 || toks[i - 1].line < t.line;
        let after_case = |k: usize| matches!(toks.get(k).map(|x| &x.tok), Some(Tok::Word(_) | Tok::Var(_)));
        if starts_line && ((matches!(&t.tok, Tok::Word(c) if c == "case") && after_case(i + 1)) || (t.tok == Tok::Attention && matches!(toks.get(i + 1).map(|x| &x.tok), Some(Tok::Word(c)) if c == "case") && after_case(i + 2))) {
            let b = t.tok == Tok::Attention;
            arm_bangs.insert(out.len(), (b, t.line, t.col));
            i += if b { 2 } else { 1 };
            continue;
        }
        // `!` before a handler's name where it is used, or before `MAIN`, `no_check`, `NO_FUCKING_CHECKS`
        if t.tok == Tok::Attention {
            match toks.get(i + 1).map(|x| &x.tok) {
                Some(Tok::Word(w)) if w == "MAIN" || w == "no_check" || w == "NO_FUCKING_CHECKS" => { i += 1; continue; }
                Some(Tok::Word(_)) => { use_bangs.push((out.len(), t.line, t.col)); i += 1; continue; }
                _ => return Err((t.line, t.col, "`!` goes on a word: `!MAIN`, `!no_check`, `!case`, `!end of`, or a handler that jumps".into())),
            }
        }
        out.push(t.clone());
        i += 1;
    }
    // a precedence level's operators, `level 2 [plus, minus] left_first` (PROVISIONAL), in the braces the parser reads
    let mut k = 0;
    while k + 2 < out.len() {
        if matches!(&out[k].tok, Tok::Word(l) if l == "level") && matches!(out[k + 1].tok, Tok::Int(_)) && out[k + 2].tok == Tok::LBracket {
            let mut d = 0;
            for j in k + 2..out.len() {
                match out[j].tok { Tok::LBracket => d += 1, Tok::RBracket => { d -= 1; if d == 0 { out[j].tok = Tok::RBrace; break; } } _ => {} }
            }
            out[k + 2].tok = Tok::LBrace;
        }
        k += 1;
    }
    check_blocks(&out, &closes, &arm_bangs, &use_bangs)?;
    fields_back(out)
}

/// Inside a `record`, a `union.tagged`, an `enumeration`: each field or case ends `;`, which the parser reads as the `,`
/// between them; an interface's types end `;` too, which it reads as nothing
fn fields_back(toks: Vec<Token>) -> Result<Vec<Token>, Fault> {
    let text = |t: &Token| shown(&t.tok);
    let mut kinds: Vec<u8> = Vec::new();   // 0 a block, 1 fields, 2 an interface
    let mut stmt_start = 0;
    let mut depth = 0;
    let mut drop: HashSet<usize> = HashSet::new();
    let mut comma: HashSet<usize> = HashSet::new();
    let mut stack: Vec<Open> = Vec::new();
    for k in 0..toks.len() {
        match toks[k].tok { Tok::LBracket | Tok::LParen => depth += 1, Tok::RBracket | Tok::RParen => depth -= 1, _ => {} }
        match &toks[k].tok {
            Tok::LBrace => {
                let o = describe(&toks, stmt_start, k, &text, None, stack.last());
                kinds.push(if o.close.starts_with("interface ") { 2 } else if o.fields { 1 } else { 0 });
                stack.push(o);
                stmt_start = k + 1;
            }
            Tok::RBrace => {
                // a field or case before the end with no `;` of its own
                if kinds.last() == Some(&1) && stmt_start < k {
                    let t = &toks[k - 1];
                    return Err((t.line, t.col, "each field and case ends with `;` (owner, 2026-10-26: commas inside brackets, semicolons inside blocks)".into()));
                }
                kinds.pop(); stack.pop(); stmt_start = k + 1;
            }
            Tok::Comma if depth == 0 && kinds.last() == Some(&1) => {
                let t = &toks[k];
                return Err((t.line, t.col, "fields and cases end with `;`, not `,` (owner, 2026-10-26: commas inside brackets, semicolons inside blocks)".into()));
            }
            Tok::Word(l) if l == "let" && kinds.last() == Some(&1) && k == stmt_start && matches!(toks.get(k + 1).map(|t| &t.tok), Some(Tok::Word(m)) if m == "immut") => { drop.insert(k); }
            Tok::Semi if depth == 0 => {
                match kinds.last() {
                    Some(1) => { if toks.get(k + 1).is_some_and(|t| t.tok == Tok::RBrace) { drop.insert(k); } else { comma.insert(k); } }
                    // a type entry (no `fn` / `op` in it) loses its `;`
                    Some(2) => { if !toks[stmt_start..k].iter().any(|t| matches!(&t.tok, Tok::Word(w) if w == "fn" || w == "op")) { drop.insert(k); } }
                    _ => {}
                }
                stmt_start = k + 1;
            }
            _ => {}
        }
    }
    Ok(toks.into_iter().enumerate().filter(|(k, _)| !drop.contains(k)).map(|(k, t)| if comma.contains(&k) { tok_like(Tok::Comma, &t) } else { t }).collect())
}

/// Every `end of` line says the kind and name its block opened with, and every `!` is where something needs attention
fn check_blocks(toks: &[Token], closes: &[(String, bool, usize, usize)], arm_bangs: &HashMap<usize, (bool, usize, usize)>, use_bangs: &[(usize, usize, usize)]) -> Result<(), Fault> {
    let kinds: HashMap<usize, bool> = WRITTEN_KINDS.with(|k| k.borrow().clone());
    let mut info = Info::default();
    for (l, p) in &kinds { if *p { info.procedures.insert(*l); } else { info.functions.insert(*l); } }
    let text = |t: &Token| shown(&t.tok);
    let mut stack: Vec<(Open, usize)> = Vec::new();
    let mut stmt_start = 0;
    let mut depth = 0;
    let mut close_at = 0;
    for (k, t) in toks.iter().enumerate() {
        match t.tok { Tok::LBracket | Tok::LParen => depth += 1, Tok::RBracket | Tok::RParen => depth -= 1, _ => {} }
        match t.tok {
            Tok::LBrace => {
                let open = describe(toks, stmt_start, k, &text, Some(&info), stack.last().map(|(o, _)| o));
                if open.list { stack.push((open, k)); continue; }
                if open.arm {
                    let (written, line, col) = arm_bangs.get(&stmt_start).cloned().unwrap_or((false, t.line, t.col));
                    if !arm_bangs.contains_key(&stmt_start) { return Err((toks[stmt_start].line, toks[stmt_start].col, format!("an arm opens with `case`: `case {}:` (owner, 2026-10-26)", open.close.trim_start_matches("case ")))); }
                    if written && !open.bang { return Err((line, col, format!("`!{}` jumps nowhere: `!` marks an arm holding a `return`, `break` or `continue` (owner, 2026-10-26)", open.close))); }
                    if !written && open.bang { return Err((line, col, format!("this arm jumps, so it wears `!`: `!{}:` … `!end of {}` (owner, 2026-10-26)", open.close, open.close))); }
                }
                stack.push((open, k));
                stmt_start = k + 1;
            }
            Tok::RBrace => {
                let Some((open, _)) = stack.pop() else { return Err((t.line, t.col, "an `end of` line with no block open".into())); };
                if open.list { continue; }
                let Some((written, bang, line, col)) = closes.get(close_at).cloned() else { return Err((t.line, t.col, "a block's end".into())); };
                close_at += 1;
                let want = open.close.clone();
                if written != want { return Err((line, col, format!("`end of {written}` closes `{want}`: write `{}end of {want}` (owner, 2026-10-26)", if open.bang { "!" } else { "" }))); }
                if bang != open.bang {
                    return Err((line, col, if open.bang { format!("this block's last line wears `!`, as its first does: `!end of {want}` (owner, 2026-10-26)") } else { format!("`!end of {want}`: `!` goes only where something needs attention (owner, 2026-10-26)") }));
                }
                stmt_start = k + 1;
            }
            Tok::Semi if depth == 0 => stmt_start = k + 1,
            _ => {}
        }
    }
    // a handler that jumps, where it is used (owner, 2026-10-26: "and also where that handler is used")
    let jumping = jumping_handlers(toks);
    let banged: HashSet<usize> = use_bangs.iter().map(|(k, _, _)| *k).collect();
    for k in 1..toks.len() {
        if !matches!(&toks[k - 1].tok, Tok::Word(o) if o == "otherwise") { continue; }
        let Tok::Word(c) = &toks[k].tok else { continue };
        let (word_at, name) = if toks.get(k + 1).is_some_and(|t| t.tok == Tok::Dot) {
            match toks.get(k + 2).map(|t| &t.tok) { Some(Tok::Word(m)) => (k + 2, format!("{c}.{m}")), _ => continue }
        } else { (k, c.clone()) };
        if !jumping_known(toks, c) { continue; }
        let jumps = jumping.contains(&name);
        let has = banged.contains(&word_at);
        let t = &toks[word_at];
        if jumps && !has { return Err((t.line, t.col, format!("`{name}` can jump from here, so its use wears `!`: `otherwise {}` (owner, 2026-10-26)", if name.contains('.') { name.replacen('.', ".!", 1) } else { format!("!{name}") }))); }
        if has && !jumps { return Err((t.line, t.col, format!("`{name}` never jumps: `!` goes only on a handler that can (owner, 2026-10-26)"))); }
    }
    // a `!` before a word that is no handler's use
    for (k, line, col) in use_bangs {
        let used = *k >= 1 && (matches!(&toks[k - 1].tok, Tok::Word(o) if o == "otherwise") || (*k >= 3 && toks[k - 1].tok == Tok::Dot && matches!(&toks[k - 3].tok, Tok::Word(o) if o == "otherwise")));
        if !used { return Err((*line, *col, "`!` goes on `MAIN`, `no_check`, an arm that jumps, its `end of` line, or a handler that jumps where it is used (owner, 2026-10-26)".into())); }
    }
    Ok(())
}

/// The handlers this file declares, by collection: only those are checked where they are used (another kit's are its own)
fn jumping_known(toks: &[Token], coll: &str) -> bool {
    (0..toks.len().saturating_sub(3)).any(|i| matches!(&toks[i].tok, Tok::Word(c) if c == "coll") && toks[i + 1].tok == Tok::Dot && matches!(&toks[i + 2].tok, Tok::Word(h) if h == "handler") && matches!(&toks[i + 3].tok, Tok::Word(n) if n == coll))
}
