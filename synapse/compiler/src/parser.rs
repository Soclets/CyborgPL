//! The parser: tokens to `ast::File`. Every refusal here is a syntax rule
//! from `design/`; the sentence says what to write instead.

use crate::ast::*;
use crate::diag::{err, Diagnostic, Result};
use crate::lexer::{Tok, Token};
use crate::source::Span;

/// Bare words the language owns. A function may be named with one only
/// through a path (`my.to[…]`); a bare `to[…]` is the keyword.
pub const KEYWORDS: &[&str] = &[
    "fn", "let", "shadow", "change", "return", "if", "else", "loop", "while", "for", "in", "break", "continue", "give", "copy", "lend", "lent", "op",
    "check", "nocheck", "true", "false", "mut", "immut", "temp", "perm", "range", "seq", "para", "list", "array", "struct", "pub",
    "const", "module", "limb", "my", "on", "fail", "otherwise", "to", "with", "enum", "match", "interface", "satisfies", "extern", "ptr",
    "layout", "size", "align", "incl", "inline", "map", "channel", "concur", "whichever", "from", "str", "char", "bool", "var", "func",
    "int8", "int16", "int32", "int64", "int128", "uint8", "uint16", "uint32", "uint64", "uint128", "bin16", "bin32", "bin64",
    "bin128", "deci32", "deci64", "deci128",
];

/// The word operators; bare, but never reserved (functions.md).
pub const WORD_OPS: &[&str] = &["plus", "minus", "times", "pow", "mod", "div", "and", "or", "not", "xor", "shl", "shr"];

pub const DURATION_UNITS: &[&str] = &["nanos", "micros", "millis", "seconds", "minutes", "hours", "nano", "micro", "milli", "second", "minute", "hour"];

pub fn is_keyword(w: &str) -> bool {
    KEYWORDS.contains(&w)
}

/// Words that name a built-in type when they head a type path.
pub fn is_primitive_head(w: &str) -> bool {
    crate::resolve::rust_alias(w).is_some()
        || matches!(w, "str" | "char" | "bool" | "int" | "uint" | "bin" | "deci" | "brain16" | "bfloat16" | "bf16")
        || (w.starts_with("int") && w[3..].parse::<u32>().is_ok())
        || (w.starts_with("uint") && w[4..].parse::<u32>().is_ok())
        || (w.starts_with("bin") && w[3..].parse::<u32>().is_ok())
        || (w.starts_with("deci") && w[4..].parse::<u32>().is_ok())
}

struct Parser<'a> {
    toks: &'a [Token],
    pos: usize,
    /// A Limb whose `CCA.cplr` does not list std (owner, 2026-10-06):
    /// no `std::` and none of std's operators.
    no_std: bool,
    /// The cores a Limb's `CCA.cplr` lists (owner, 2026-10-06); `None`
    /// for a program without one, which has every core.
    cores: Option<Vec<String>>,
    /// What the `incl.C` lines of this scope declared so far, by name: the
    /// same declaration twice is one (owner, 2026-10-10).
    c_seen: std::collections::HashMap<String, String>,
    /// Reading what Synapse wrote for an `incl.C` line: a C struct may be
    /// opaque (`layout.C struct FILE;`) and a field may be named like a
    /// keyword (owner, 2026-10-10).
    from_c: bool,
    /// A `#strict` file (owner, 2026-09-27): `pos` on every position,
    /// every parameter's type, a return type, `mut`/`immut` on every
    /// declaration, `temp`/`perm` on every `for`.
    strict: bool,
    /// Whether the body being parsed has a `return` with a value: in a
    /// strict file such a function says its type.
    returns_value: bool,
    /// What the last `-> …` said about a returned borrow.
    ret_lent: Option<RetLent>,
    /// The levels this file's `precedence` block gives operators (owner,
    /// 2026-10-03), by name without a path: how tightly each binds, and
    /// how a chain of it reads.
    prec: std::collections::HashMap<String, (u16, Chain)>,
}

/// A binary operator at the cursor (`Parser::binop`).
struct OpTok {
    op: BinOp,
    l: u16,
    r: u16,
    span: Span,
    /// Tokens that spell it: `plus.wrap` is three, `areas::mean` three.
    len: usize,
    /// The word a retired spelling is now: `+` is `plus`.
    old: Option<&'static str>,
}

/// How a chain of one level reads (owner, 2026-10-03):
/// `level.when_chained.left_first`, `.right_first`, or `.refused`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    Left,
    Right,
    Refused,
}

/// The language's own levels, in thousands so a Limb's fit between them
/// (the oracle's "hundreds", ten times finer): loosest first. A negative
/// sits above `times` and below std's `pow`, so `-2 pow 2` is -4.
const OR_LEVEL: u16 = 1000;
const AND_LEVEL: u16 = 2000;
const CMP_LEVEL: u16 = 3000;
const JOIN_LEVEL: u16 = 4000;
const ADD_LEVEL: u16 = 5000;
const MUL_LEVEL: u16 = 6000;
const NEG_LEVEL: u16 = 7000;
const POW_LEVEL: u16 = 8000;

/// Binding powers for a level: a chain left first takes the left pair
/// first, right first the right one; a refused chain parses left first and
/// is refused where it chains.
fn powers(level: u16, chain: Chain) -> (u16, u16) {
    match chain {
        Chain::Right => (2 * level, 2 * level),
        _ => (2 * level, 2 * level + 1),
    }
}

/// A language operator's level and chain, by the name a `precedence`
/// level writes it with; `None` for one with no precedence.
fn language_level(name: &str) -> Option<(u16, Option<Chain>)> {
    let base = name.split('.').next().unwrap_or(name);
    Some(match (name, base) {
        ("or" | "or.else", _) => (OR_LEVEL, Some(Chain::Left)),
        ("and" | "and.then", _) => (AND_LEVEL, Some(Chain::Left)),
        ("==" | "!==" | "<" | ">" | "</==" | ">/==" | "equal_to" | "not_equal" | "at_most" | "at_least", _) => (CMP_LEVEL, Some(Chain::Refused)),
        ("++", _) => (JOIN_LEVEL, Some(Chain::Left)),
        ("plus" | "minus" | "plus.wrap" | "minus.wrap", _) => (ADD_LEVEL, Some(Chain::Left)),
        ("times" | "times.wrap", _) | (_, "div" | "mod") => (MUL_LEVEL, Some(Chain::Left)),
        // A negative is one value's: any chain may hold it.
        ("neg", _) => (NEG_LEVEL, None),
        ("pow" | "pow.wrap", _) => (POW_LEVEL, Some(Chain::Right)),
        _ => return None,
    })
}

/// The word an old operator spelling is now (owner, 2026-10-03).
fn retired_spelling(s: &str) -> Option<&'static str> {
    Some(match s {
        "+" => "plus",
        "-" => "minus",
        "x" => "times",
        "xx" => "pow",
        "+~" => "plus.wrap",
        "-~" => "minus.wrap",
        "x~" => "times.wrap",
        "xx~" => "pow.wrap",
        _ => return None,
    })
}

// Every Limb's operator levels, read before any file is parsed (owner,
// 2026-10-03, 9cc5e25): a Limb's `precedence` applies wherever its
// operators are used, in other Limbs too. Set by the loader.
thread_local! {
    static SHARED_PREC: std::cell::RefCell<std::collections::HashMap<String, (u16, Chain)>> = Default::default();
}

thread_local! { static NO_STD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

// The Neuron's root, where an `incl.C["…"]` path starts.
thread_local! { static NEURON_ROOT: std::cell::RefCell<std::path::PathBuf> = std::cell::RefCell::new(std::path::PathBuf::from(".")); }

/// Set while a Neuron's files are parsed; gives back the one before.
pub fn set_neuron_root(dir: &std::path::Path) -> std::path::PathBuf {
    NEURON_ROOT.with(|r| std::mem::replace(&mut *r.borrow_mut(), dir.to_path_buf()))
}

/// Whether the files parsed next belong to a Limb whose `CCA.cplr` does
/// not list std (owner, 2026-10-06); a program with no `CCA.cplr` gets the
/// full package, as the machine's install gives it.
pub fn set_no_std(on: bool) {
    NO_STD.with(|c| c.set(on));
}

thread_local! { static CORES: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) }; }

/// The cores the Limb whose files are parsed next lists (owner,
/// 2026-10-06: "Limbs should say what cores they are built for"); `None`
/// for a program with no `CCA.cplr`, which has them all.
pub fn set_cores(cores: Option<Vec<String>>) {
    CORES.with(|c| *c.borrow_mut() = cores);
}

/// Each core's own words, and the core that has them (owner, 2026-10-06).
fn core_word(toks: &[Token], i: usize) -> Option<(&'static str, &'static str)> {
    let Tok::Word(w) = &toks[i].tok else { return None };
    // A word right after `.` or `::` is a name in a path, not the keyword.
    if i > 0 && matches!(toks[i - 1].tok, Tok::Dot | Tok::ColonColon) {
        return None;
    }
    let next_is = |n: usize, t: &str| matches!(toks.get(i + n).map(|t| &t.tok), Some(Tok::Word(x)) if x == t);
    let dot_then = |t: &[&str]| matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Dot)) && matches!(toks.get(i + 2).map(|t| &t.tok), Some(Tok::Word(x)) if t.contains(&x.as_str()));
    Some(match w.as_str() {
        // The slimmer base (owner, 2026-10-07): what left it, by core.
        "test" if next_is(1, "fn") => ("`test fn`", "TAS"),
        "must" if matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Name(_) | Tok::Word(_) | Tok::Num { .. } | Tok::LParen)) => ("`must`", "TAS"),
        "brain16" => ("`brain16`", "AI"),
        // Everything about C is CNC's (owner, 2026-10-10); `ptr`, raw
        // memory and inline assembly stay the Brainstem's.
        "extern" if dot_then(&["C"]) => ("`extern.C`", "CNC"),
        "incl" if dot_then(&["C"]) => ("`incl.C`", "CNC"),
        "layout" if dot_then(&["C"]) => ("`layout.C`", "CNC"),
        "int128" | "uint128" => (if w == "int128" { "`int128`" } else { "`uint128`" }, "WSMW"),
        "not_std" => ("a `not_std.` width", "WSMW"),
        "deci32" | "deci64" | "deci128" => (match w.as_str() { "deci32" => "`deci32`", "deci64" => "`deci64`", _ => "`deci128`" }, "Money"),
        "str" if dot_then(&["utf16", "utf32"]) => (if next_is(2, "utf16") { "`str.utf16`" } else { "`str.utf32`" }, "UniTect"),
        // UTF-16 and UTF-32 included need UniTect, as their text does.
        "incl" if dot_then(&["str"]) && matches!(toks.get(i + 3).map(|t| &t.tok), Some(Tok::Dot)) && matches!(toks.get(i + 4).map(|t| &t.tok), Some(Tok::Word(x)) if x == "utf16" || x == "utf32") => (if matches!(toks.get(i + 4).map(|t| &t.tok), Some(Tok::Word(x)) if x == "utf16") { "`incl.str.utf16`" } else { "`incl.str.utf32`" }, "UniTect"),
        "str" if dot_then(&["points", "graphemes"]) => (if next_is(2, "points") { "`str.points`" } else { "`str.graphemes`" }, "UniTect"),
        "str" if dot_then(&["each", "at"]) && matches!(toks.get(i + 3).map(|t| &t.tok), Some(Tok::Dot)) && matches!(toks.get(i + 4).map(|t| &t.tok), Some(Tok::Word(x)) if x == "point" || x == "grapheme") => ("counting in points or graphemes", "UniTect"),
        "char" if dot_then(&["point", "grapheme"]) => (if next_is(2, "point") { "`char.point`" } else { "`char.grapheme`" }, "UniTect"),
        "case" if dot_then(&["any", "exact", "insensitive", "sensitive"]) => ("a `case` mark", "CAPS"),
        "nocheck" => ("`no_check`", "IEG"),
        "NO_FUCKING_CHECKS" => ("`NO_FUCKING_CHECKS`", "IEG"),
        "on" if matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Dot)) && next_is(2, "machine") => ("`on.machine`", "IEG"),
        "concur" => ("`concur`", "MCPS"),
        "whichever" => ("`whichever`", "MCPS"),
        "channel" => ("`channel`", "MCPS"),
        // `for para`, or `for !cheat para`.
        "for" if next_is(1, "para") || (next_is(1, "no_cheat") && next_is(2, "para")) => ("`for para`", "MCPS"),
        _ => return None,
    })
}

/// An old spelling at token `i` (the 2026-10-07 revision; owner: "MTM is
/// gone."), and what to say: each names the one that replaced it.
fn old_spelling(toks: &[Token], i: usize) -> Option<String> {
    let old = |w: &str, m: &str| Some(format!("`{}` is an old spelling: write `{}`", w, m));
    if let Some((w, m)) = &toks[i].alt {
        return old(w, m);
    }
    let after = |n: usize, t: &Tok| i >= n && toks[i - n].tok == *t;
    let glued = |n: usize| toks.get(i + n).is_some_and(|t| t.span.start == toks[i + n - 1].span.end);
    let next = |n: usize| toks.get(i + n).map(|t| &t.tok);
    match &toks[i].tok {
        // The `!` marks became words (owner, 2026-10-07).
        Tok::Op(o) if o == "!" => {
            if glued(1) {
                match next(1) {
                    Some(Tok::Word(w)) if w == "check" => return old("!check", "no_check"),
                    Some(Tok::Word(w)) if w == "mut" => return old("!mut", "immut"),
                    Some(Tok::Word(w)) if w == "cheat" => return old("!cheat", "no_cheat"),
                    Some(Tok::Word(w)) if w == "c" => return Some("`!c` is `no_cheat` now".into()),
                    Some(Tok::Word(w)) if (w == "std" && matches!(next(2), Some(Tok::Dot))) || w.starts_with("std_") => return old("!std.", "not_std."),
                    Some(Tok::Dollar) => return old("!$", "no_shadow"),
                    Some(Tok::Name(n)) => return Some(format!("`!` is not an operator since 2026-10-07: a value's not is `not '{}'`", n)),
                    _ => {}
                }
            }
            old("!", "not")
        }
        // `$:=` and `$=` were shadowing (owner, 2026-10-07: `shadow`).
        Tok::Dollar if !after(1, &Tok::Op("!".into())) => match toks.get(i + 1).map(|t| &t.tok) {
            Some(Tok::Walrus) => Some("`$:=` is an old spelling: write `shadow 'x' := …`".into()),
            Some(Tok::Op(o)) if o == "=" => Some("`$=` is an old spelling: write `shadow 'x': T = …`".into()),
            _ => Some("`$` is not in the language since 2026-10-07: shadowing is `shadow 'x' := …;`".into()),
        },
        Tok::Op(o) if o.starts_with('&') && !o.starts_with("&&") && !matches!(o.as_str(), "&+" | "&-" | "&*") => Some(format!("`{}` is an old spelling: write `lend` / `lent`; a bitwise and is `and.bit`", o)),
        // The inline languages' whole names (owner, 2026-10-07: "`a`").
        Tok::Word(w) if after(1, &Tok::Dot) && i >= 2 && toks[i - 2].tok == Tok::Word("inline".into()) => match w.as_str() {
            "Assembly" | "S" => old(&format!("inline.{}", w), "inline.ASM"),
            "LLVM_Intermediate_Representation" | "LLVM" => old(&format!("inline.{}", w), "inline.LL"),
            "Metal_Shading_Language" => old(&format!("inline.{}", w), "inline.MSL"),
            _ => None,
        },
        // `not_std.i24` for `not_std.int24`, `not_std.u24` for `not_std.uint24`.
        Tok::Word(w) if after(1, &Tok::Dot) && i >= 2 && toks[i - 2].tok == Tok::Word("not_std".into()) && w.len() > 1 && (w.starts_with('i') && !w.starts_with("int") || w.starts_with('u') && !w.starts_with("uint")) && w[1..].bytes().all(|c| c.is_ascii_digit()) => {
            let (kind, bits) = w.split_at(1);
            old(&format!("not_std.{}", w), &format!("not_std.{}{}", if kind == "i" { "int" } else { "uint" }, bits))
        }
        // `pub.neuron` (owner, 2026-10-09: a library is a Neuron).
        Tok::Word(w) if w == "limb" && after(1, &Tok::Dot) && i >= 2 && toks[i - 2].tok == Tok::Word("pub".into()) => old("limb", "neuron"),
        Tok::Word(_) if after(1, &Tok::Dot) || after(1, &Tok::ColonColon) => None,
        Tok::Word(w) => rust_name(w).and_then(|m| old(w, m)),
        _ => None,
    }
}

/// Rust's number names and `bf16`/`bfloat16`, old spellings since MTM went
/// (owner, 2026-10-07).
fn rust_name(w: &str) -> Option<&'static str> {
    Some(match w {
        "i8" => "int8",
        "i16" => "int16",
        "i32" => "int32",
        "i64" => "int64",
        "i128" => "int128",
        "u8" => "uint8",
        "u16" => "uint16",
        "u32" => "uint32",
        "u64" => "uint64",
        "u128" => "uint128",
        "f16" => "bin16",
        "f32" => "bin32",
        "f64" => "bin64",
        "bf16" | "bfloat16" => "brain16",
        _ => return None,
    })
}

fn core_missing<T>(what: &str, core: &str, at: Span) -> Result<T> {
    err(Diagnostic::error(format!("{} is {}'s, and this Neuron's Kinesin.nrp does not list the lobe {}: add `{}` to its `lobes`", what, core, core, core_line(core))).at(at))
}

/// std's registry ID and the one version CCA knows (owner, 2026-10-06).
pub const STD_ID: &str = "fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936";
pub const STD_VERSION: &str = "2026-10-06T12:57:14+07:00";

pub fn set_shared_precedence(table: std::collections::HashMap<String, (u16, Chain)>) {
    SHARED_PREC.with(|t| *t.borrow_mut() = table);
}

/// The `precedence` blocks in one file's tokens: each operator's level.
pub fn precedence_blocks(toks: &[Token]) -> Result<Vec<(String, (u16, Chain))>> {
    let mut p = Parser { toks, pos: 0, strict: false, returns_value: false, ret_lent: None, prec: Default::default(), no_std: false, cores: None, c_seen: Default::default(), from_c: false };
    let mut out = Vec::new();
    let mut depth = 0i32;
    for i in 0..toks.len() {
        match &toks[i].tok {
            Tok::LBrace => depth += 1,
            Tok::RBrace => depth -= 1,
            Tok::Word(w) if w == "precedence" && depth == 0 && toks[i].line_first => {
                p.pos = i;
                out.extend(p.precedence_block()?);
            }
            _ => {}
        }
    }
    Ok(out)
}

pub fn parse_file(toks: &[Token]) -> Result<File> {
    let mut p = Parser { toks, pos: 0, strict: false, returns_value: false, ret_lent: None, prec: Default::default(), no_std: false, cores: None, c_seen: Default::default(), from_c: false };
    // Every Limb's levels, and this file's own blocks, read before any
    // expression in it: an operator above its block is parsed by it too.
    p.prec = SHARED_PREC.with(|t| t.borrow().clone());
    p.prec.extend(precedence_blocks(toks)?);
    p.no_std = NO_STD.with(|c| c.get());
    p.cores = CORES.with(|c| c.borrow().clone());
    // Growing lists, maps and text operations are std's (owner,
    // 2026-10-06), as a type, a literal or an operation; fixed arrays and
    // text literals stay.
    if p.no_std {
        for i in 0..toks.len() {
            // A duration and its literals are stdkit's (owner, 2026-09-22
            // "Time: std, with duration literals"; d690ed91).
            if let (Tok::Num { .. }, Some(Tok::Word(u))) = (&toks[i].tok, toks.get(i + 1).map(|t| &t.tok)) {
                if DURATION_UNITS.contains(&u.as_str()) {
                    return err(Diagnostic::error(format!("a duration literal (`5 seconds`) is cord's, and this Neuron's Kinesin.nrp does not list cord: add `kin::Dep[id = \"{}\", version = \"{}\"]` to its `dependencies`", STD_ID, STD_VERSION)).at(toks[i].span));
                }
            }
            let Tok::Word(w) = &toks[i].tok else { continue };
            if i > 0 && matches!(toks[i - 1].tok, Tok::Dot | Tok::ColonColon) {
                continue;
            }
            let next = toks.get(i + 1).map(|t| &t.tok);
            let what = match (w.as_str(), next) {
                ("list", Some(Tok::Dot | Tok::LBracket)) => "a growing list (`list`)",
                ("map", Some(Tok::Dot | Tok::LBracket)) => "a map (`map`)",
                ("duration", _) => "`duration`",
                ("str", Some(Tok::Dot)) if matches!(toks.get(i + 2).map(|t| &t.tok), Some(Tok::Word(u)) if matches!(u.as_str(), "bytes" | "points" | "graphemes" | "each" | "split" | "find" | "contains" | "trim" | "upper" | "lower" | "replace" | "starts_with" | "ends_with" | "at" | "slice" | "join")) => "a text operation (`str.…`)",
                _ => continue,
            };
            return err(Diagnostic::error(format!("{} is cord's, and this Neuron's Kinesin.nrp does not list cord: add `kin::Dep[id = \"{}\", version = \"{}\"]` to its `dependencies`", what, STD_ID, STD_VERSION)).at(toks[i].span));
        }
    }
    // Signatar's names are English letters, digits and `_`, bare or
    // quoted (owner, 2026-10-06); any other script, or an emoji, is WSTN's.
    if p.cores.as_ref().is_some_and(|c| !c.iter().any(|c| c == "WSTN")) {
        for t in toks.iter() {
            let shown = match &t.tok {
                Tok::Word(w) if !w.is_ascii() => format!("`{}`", w),
                Tok::Name(n) if !n.is_ascii() => format!("`'{}'`", n),
                _ => continue,
            };
            return err(Diagnostic::error(format!("{} is not an English name: Brainstem's names are English letters, digits and `_`; a name in another script, or an emoji, needs the lobe WSTN in this Neuron's `lobes` (`{}`)", shown, core_line("WSTN"))).at(t.span));
        }
    }
    // Old spellings, in every Limb (the 2026-10-07 revision; owner: "MTM
    // is gone."): each refused naming the one that replaced it.
    for i in 0..toks.len() {
        if let Some(msg) = old_spelling(toks, i) {
            return err(Diagnostic::error(msg).at(toks[i].span));
        }
    }
    // A core's words, in a Limb that does not list the core.
    if let Some(cores) = &p.cores {
        for i in 0..toks.len() {
            if let Some((what, core)) = core_word(toks, i) {
                if !cores.iter().any(|c| c == core) {
                    return core_missing(what, core, toks[i].span);
                }
            }
        }
    }
    let strict = None;
    // `#strict` is gone (owner, 2026-10-07: "We ain't no C stability,
    // remove it now"); what it wrote out is every file's.
    if p.is(&Tok::Strict) {
        return p.fail("`#strict` is gone: the labelled ranges, `pos`, `count`, `is` and typed parameters it asked for are every file's now");
    }
    let mut items = Vec::new();
    let mut bare_file = None;
    let mut bare_kinds = Vec::new();
    let mut case_file = None;
    let mut case_kinds = Vec::new();
    while !p.at_eof() {
        // A file's `case` mark at its top, beside its `bare` mark in either
        // order, and the kind lines (owner, 2026-10-06).
        if p.at_case_mark() && matches!(p.peek_at(3), Tok::Semi | Tok::LBracket) {
            let (mark, at, kinds) = p.case_line()?;
            match kinds {
                Some(kinds) => case_kinds.push((mark, kinds, at)),
                None if items.is_empty() && case_file.is_none() => case_file = Some((mark, at)),
                None => return p.fail_at(at, CASE_FILE_MARK_FIRST),
            }
            continue;
        }
        if p.is(&Tok::Strict) {
            return p.fail("`#strict` goes once, at the top of the file, before any item");
        }
        // A file's mark, its first line, and the kind lines (owner,
        // 2026-10-06).
        if p.is_word("bare") {
            let at = p.span();
            let (mark, kinds) = p.bare_line()?;
            match kinds {
                Some(kinds) => bare_kinds.push((mark, kinds, at)),
                None if items.is_empty() && bare_file.is_none() => bare_file = Some((mark, at)),
                None => return p.fail_at(at, FILE_MARK_FIRST),
            }
            continue;
        }
        if p.is_word("on") {
            items.extend(p.on_items()?);
            continue;
        }
        push_item(&mut items, p.item()?);
    }
    Ok(File { items, strict, bare_file, bare_kinds, case_file, case_kinds })
}

/// A file that is one value and its `;`, as a `CCA.cplr` is (owner,
/// 2026-10-06): `cca::Manifest[ … ];`.
pub fn parse_value(toks: &[Token]) -> Result<Expr> {
    let mut p = Parser { toks, pos: 0, strict: false, returns_value: false, ret_lent: None, prec: Default::default(), no_std: false, cores: None, c_seen: Default::default(), from_c: false };
    let value = p.expr(0)?;
    p.expect_semi("the value")?;
    if !p.at_eof() {
        return p.fail("the file is one value and its `;`, with nothing after it");
    }
    Ok(value)
}

const NOT_A_COMMENT: &str = "";

const CASE_FILE_MARK_FIRST: &str = "a file's `case` mark is at its top, before anything else; a block module takes `case.any module name { … }`";

fn case_spelled(insensitive: bool) -> &'static str {
    if insensitive { "case.any" } else { "case.exact" }
}

const FILE_MARK_FIRST: &str = "a file's `bare` mark is its first line, before anything else; a block module takes `pub bare.… module name { … }`";

impl<'a> Parser<'a> {
    // ------------------------------------------------------------ helpers

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i].tok
    }
    fn span(&self) -> Span {
        self.toks[self.pos].span
    }
    fn span_at(&self, n: usize) -> Span {
        let i = (self.pos + n).min(self.toks.len() - 1);
        self.toks[i].span
    }
    fn prev_span(&self) -> Span {
        self.toks[self.pos.saturating_sub(1)].span
    }
    fn at_eof(&self) -> bool {
        matches!(self.peek(), Tok::Eof)
    }
    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if !self.at_eof() {
            self.pos += 1;
        }
        t
    }
    fn is_word(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Word(x) if x == w)
    }
    fn is_word_at(&self, n: usize, w: &str) -> bool {
        matches!(self.peek_at(n), Tok::Word(x) if x == w)
    }
    fn is_op(&self, o: &str) -> bool {
        matches!(self.peek(), Tok::Op(x) if x == o)
    }
    fn is_op_at(&self, n: usize, o: &str) -> bool {
        matches!(self.peek_at(n), Tok::Op(x) if x == o)
    }
    fn is(&self, t: &Tok) -> bool {
        self.peek() == t
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.is(t) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_word(&mut self, w: &str) -> bool {
        if self.is_word(w) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_op(&mut self, o: &str) -> bool {
        if self.is_op(o) {
            self.bump();
            true
        } else {
            false
        }
    }
    /// Two tokens are adjacent (no space between).
    fn adjacent(&self, a: usize, b: usize) -> bool {
        self.span_at(a).end == self.span_at(b).start
    }

    /// Refuses one of std's operators or paths in a Limb that does not
    /// list std (owner, 2026-10-06).
    fn std_gate(&self, what: &str, span: Span) -> Result<()> {
        if !self.no_std {
            return Ok(());
        }
        self.fail_at(span, format!("`{}` is cord's, and this Neuron's Kinesin.nrp does not list cord: add `kin::Dep[id = \"{}\", version = \"{}\"]` to its `dependencies`", what, STD_ID, STD_VERSION))
    }

    fn fail<T>(&self, msg: impl Into<String>) -> Result<T> {
        err(Diagnostic::error(msg).at(self.span()))
    }
    fn fail_at<T>(&self, span: Span, msg: impl Into<String>) -> Result<T> {
        err(Diagnostic::error(msg).at(span))
    }

    fn expect(&mut self, t: &Tok, what: &str) -> Result<Span> {
        if self.is(t) {
            Ok(self.bump().span)
        } else {
            self.fail(format!("expected {}, found {}", what, self.peek().describe()))
        }
    }
    fn expect_semi(&mut self, after: &str) -> Result<()> {
        if self.eat(&Tok::Semi) {
            Ok(())
        } else {
            self.fail(format!("expected `;` after {}, found {}", after, self.peek().describe()))
        }
    }
    fn expect_word(&mut self, w: &str) -> Result<Span> {
        if self.is_word(w) {
            Ok(self.bump().span)
        } else {
            self.fail(format!("expected `{}`, found {}", w, self.peek().describe()))
        }
    }

    fn ident(&mut self, what: &str) -> Result<Ident> {
        match self.peek().clone() {
            Tok::Word(name) => {
                let span = self.bump().span;
                Ok(Ident { name, span })
            }
            Tok::Name(n) => self.fail(format!("expected {}, found the quoted name '{}': {} is bare — quotes mark a value you hold", what, n, what)),
            other => self.fail(format!("expected {}, found {}", what, other.describe())),
        }
    }

    fn qname(&mut self, what: &str) -> Result<QName> {
        match self.peek().clone() {
            Tok::Name(name) => {
                let span = self.bump().span;
                Ok(QName { name, span })
            }
            Tok::Word(w) => self.fail(format!("expected {}, found the bare word `{}`: {} is a value you hold, so it is written in quotes — `'{}'`", what, w, what, w)),
            other => self.fail(format!("expected {}, found {}", what, other.describe())),
        }
    }

    /// `a.b.c` or `std::a.b`. At least one segment.
    fn path(&mut self) -> Result<Path> {
        let start = self.span();
        let mut limb = None;
        let first = self.ident("a name")?;
        let mut segs = Vec::new();
        if self.is(&Tok::ColonColon) {
            self.bump();
            limb = Some(first);
            segs.push(self.ident("a name after `::`")?);
        } else {
            segs.push(first);
        }
        // `.word` continues the path; `.'name'` is a const reached by path and stops it.
        while self.is(&Tok::Dot) && matches!(self.peek_at(1), Tok::Word(_)) {
            self.bump();
            segs.push(self.ident("a name after `.`")?);
        }
        let span = start.to(self.prev_span());
        // The root path is `my.` (owner, 2026-10-06: "Is it `my.`?" —
        // "yea"); `pub.limb` and `limb.cpls` keep the word.
        if limb.is_none() && segs[0].name == "limb" {
            return self.fail_at(segs[0].span, "the root of this Neuron is `my.` since 2026-10-06, no longer `limb.`: `my.helper[]`, `my.'pi'`");
        }
        if let Some(l) = limb.as_ref().filter(|l| l.name == "std") {
            self.std_gate(&format!("cord::{}", segs[0].name), l.span.to(segs[0].span))?;
        }
        Ok(Path { limb, segs, span })
    }

    // ------------------------------------------------------------ items

    fn item(&mut self) -> Result<Item> {
        let start = self.span();
        let mut vis = Vis::Private;
        let mut bare = None;
        let mut case = None;
        // `case.any fn …` with no `pub`; a mark line here is in a
        // block module or not at the top (owner, 2026-10-06).
        if self.at_case_mark() {
            let (mark, at) = self.case_mark()?;
            if self.is_word("pub") {
                return self.fail_at(at, format!("`pub` comes first: `pub {} fn …`", case_spelled(mark)));
            }
            if self.is(&Tok::LBracket) {
                return self.fail_at(at, "a `case` mark on kinds (`case.any [fn, struct];`) covers the whole Neuron, so it is written in `neuron.nrv`");
            }
            if self.is(&Tok::Semi) {
                return self.fail_at(at, CASE_FILE_MARK_FIRST);
            }
            case = Some((mark, at));
        }
        // A mark line inside a block module, or a mark with no `pub`.
        if self.is_word("bare") {
            let at = self.span();
            return match self.bare_line()? {
                (_, Some(_)) => self.fail_at(at, "a `bare.… […];` line covers the whole Neuron, so it is written in `neuron.nrv`"),
                (_, None) => self.fail_at(at, FILE_MARK_FIRST),
            };
        }
        if self.is_word("pub") {
            self.bump();
            if self.eat(&Tok::Dot) {
                let narrowing = self.ident("a word after `pub.`")?;
                if narrowing.name != "neuron" {
                    return self.fail_at(narrowing.span, format!("`pub.{}` is not a visibility: the only narrowing of `pub` is `pub.neuron` (this Neuron only); plain `pub` is other Neurons too", narrowing.name));
                }
                vis = Vis::Limb;
            } else {
                vis = Vis::Pub;
            }
            if self.at_case_mark() {
                case = Some(self.case_mark()?);
            }
            if self.is_word("bare") {
                let (mark, at) = self.bare_mark()?;
                if vis == Vis::Limb {
                    return self.fail_at(at, "a `bare` mark says how other Neurons write the path, and a `pub.neuron` thing is never reached from them");
                }
                bare = Some((mark, at));
            }
            if case.is_none() && self.at_case_mark() {
                case = Some(self.case_mark()?);
            }
            if self.is_word("MAIN") {
                return self.fail("`MAIN` is never `pub`: it is the program's block, not something another Neuron can reach");
            }
            if self.is_word("let") {
                return self.fail("a Neuron-level value is a `const`, not a `let`: `pub const 'name' := …;`");
            }
            if let Tok::Name(n) = self.peek().clone() {
                return self.fail(format!("a Neuron-level value is a `const`, not a variable: `pub const '{}' := …;`", n));
            }
        }
        // `test fn` (owner, 2026-09-29): never `pub`, run by `cca test`.
        if self.is_word("test") && self.is_word_at(1, "fn") {
            if vis != Vis::Private {
                return self.fail("a `test fn` is never `pub`: nothing reaches it but `synapse test`");
            }
            self.bump();
            let kind = self.item_kind(vis)?;
            let kind = match kind {
                ItemKind::Fn(mut d) => {
                    d.test = true;
                    ItemKind::Fn(d)
                }
                other => other,
            };
            let span = start.to(self.prev_span());
            return Ok(Item { vis, kind, span, bare, case });
        }
        let kind = self.item_kind(vis)?;
        if let (ItemKind::Const(_), Some((_, at))) = (&kind, &bare) {
            return self.fail_at(*at, CONST_TAKES_NO_MARK);
        }
        if let (ItemKind::Op(d), Some(_)) = (&kind, &bare) {
            if !d.op.name.starts_with(|c: char| c.is_ascii_alphabetic()) {
                return self.fail_at(d.op.span, format!("`{}` is a glyph operator, which has nowhere to write a path, so there is nothing for `bare` to say", d.op.name));
            }
        }
        if let (ItemKind::Main(..), Some((_, at))) = (&kind, &case) {
            return self.fail_at(*at, "`MAIN` has no name to mark: a `case` mark goes on a named item, a module, or a file");
        }
        let span = start.to(self.prev_span());
        Ok(Item { vis, kind, span, bare, case })
    }

    /// One condition of an `on` chain, up to its `{` (owner, 2026-10-06):
    /// `on.target aarch64-macos`, `on.target.chip x86_64`, `on.target.os
    /// windows` — `Ok(Ok(matched))`, chosen against the target now — or
    /// `on.machine.chip sve`, asked when running: `Ok(Err(what))`.
    fn on_cond(&mut self, top: bool) -> Result<std::result::Result<bool, String>> {
        let narrowed = "`on` is narrowed: `on.target …` (chosen when compiling) or `on.machine …` (when running)";
        self.bump();
        if !self.is(&Tok::Dot) {
            return self.fail(narrowed);
        }
        self.bump();
        let which = self.ident("`target` or `machine`")?;
        let list = |names: &[&str]| names.join(", ");
        match which.name.as_str() {
            "machine" => {
                if top {
                    return self.fail_at(which.span, "`on.machine` chooses when running, so it goes inside a body; around items, `on.target`");
                }
                if !self.is(&Tok::Dot) || !self.is_word_at(1, "chip") {
                    return self.fail("`on.machine` asks the chip what it can do: `on.machine.chip sve { … }`");
                }
                self.bump();
                self.bump();
                let f = self.ident("a chip or a feature")?;
                if !ON_FEATURES.contains(&f.name.as_str()) {
                    return self.fail_at(f.span, format!("`{}` is not a chip or feature Synapse can ask about: {}", f.name, list(&ON_FEATURES)));
                }
                Ok(Err(f.name))
            }
            "target" => {
                // The build's target (`crate::target`), set before any file
                // is read.
                let (chip, os) = (crate::target::chip(), crate::target::os());
                let (chip, os) = (chip.as_str(), os.as_str());
                if self.eat(&Tok::Dot) {
                    let part = self.ident("`chip` or `os`")?;
                    let name = self.ident("a name")?;
                    let matched = match part.name.as_str() {
                        "chip" => {
                            if !ON_CHIPS.contains(&name.name.as_str()) {
                                return self.fail_at(name.span, format!("`{}` is not a chip `on.target.chip` knows: {}", name.name, list(&ON_CHIPS)));
                            }
                            if let Tok::Word(f) = self.peek().clone() {
                                return self.fail(format!("a chip's minimum (`on.target.chip {} {}`) is not built: how a build names the chip features it needs is not decided; `on.machine.chip` checks when running", name.name, f));
                            }
                            name.name == chip
                        }
                        "os" => {
                            if !ON_OSES.contains(&name.name.as_str()) {
                                return self.fail_at(name.span, format!("`{}` is not an OS `on.target.os` knows: {}", name.name, list(&ON_OSES)));
                            }
                            if let Tok::Num { text, .. } = self.peek().clone() {
                                return self.fail(format!("an OS minimum (`on.target.os {} {}`) is not built: how a build names the oldest version it runs on is not decided", name.name, text));
                            }
                            name.name == os
                        }
                        other => return self.fail_at(part.span, format!("`on.target.{}` is not a narrowing: `on.target.chip …` or `on.target.os …`", other)),
                    };
                    return Ok(Ok(matched));
                }
                let c = self.ident("a target, chip-os")?;
                if !self.is_op("-") {
                    return self.fail_at(c.span, format!("`{}` is a chip; a whole target is chip-os (`{}-macos`), or narrow: `on.target.chip {}`", c.name, c.name, c.name));
                }
                self.bump();
                let o = self.ident("an OS after `-`")?;
                if !ON_CHIPS.contains(&c.name.as_str()) || !ON_OSES.contains(&o.name.as_str()) {
                    return self.fail_at(c.span.to(o.span), format!("`{}-{}` is not a target: chip-os, from {} and {}", c.name, o.name, list(&ON_CHIPS), list(&ON_OSES)));
                }
                Ok(Ok(c.name == chip && o.name == os))
            }
            _ => self.fail_at(which.span, narrowed),
        }
    }

    /// After a branch: `else on.…` goes on, `else { … }` ends, and
    /// `else if` is refused. `Some(true)` when another condition follows.
    fn on_else(&mut self) -> Result<Option<bool>> {
        if !self.is_word("else") {
            return Ok(None);
        }
        self.bump();
        if self.is_word("on") {
            return Ok(Some(true));
        }
        if self.is_word("if") {
            return self.fail("an `on` chain goes on with `else on.…` or ends with `else { }`; an `if` goes inside a branch");
        }
        Ok(Some(false))
    }

    /// An `on` chain inside a body: the branches `on.target` rules out are
    /// read and dropped, and what is left is an `if` chain asking the
    /// machine, its `else` the branch the target chose.
    fn on_stmt(&mut self) -> Result<StmtKind> {
        let start = self.span();
        let mut asked: Vec<(String, Block)> = Vec::new();
        let mut chosen: Option<Block> = None;
        loop {
            let cond = self.on_cond(false)?;
            let body = self.block("this branch")?;
            match cond {
                Ok(true) if chosen.is_none() => chosen = Some(body),
                Err(what) if chosen.is_none() => asked.push((what, body)),
                _ => {}
            }
            match self.on_else()? {
                Some(true) => continue,
                Some(false) => {
                    let body = self.block("`else`")?;
                    if chosen.is_none() {
                        chosen = Some(body);
                    }
                }
                None => {}
            }
            break;
        }
        let mut rest = chosen.map(ElseArm::Block);
        for (what, then) in asked.into_iter().rev() {
            let at = then.span;
            let segs = ["on", "machine", "chip", &what].iter().map(|n| Ident { name: n.to_string(), span: at }).collect();
            let cond = Expr { kind: ExprKind::Call { path: Path { limb: None, segs, span: at }, args: Vec::new(), otherwise: None }, span: at };
            let kind = StmtKind::If { cond, then, else_: rest };
            rest = Some(ElseArm::If(Box::new(Stmt { kind, span: at })));
        }
        Ok(match rest {
            Some(ElseArm::If(s)) => s.kind,
            Some(ElseArm::Block(b)) => StmtKind::Block(b),
            None => StmtKind::Block(Block { stmts: Vec::new(), span: start.to(self.prev_span()) }),
        })
    }

    /// An `on.target` chain around items: only the chosen branch's items
    /// exist.
    fn on_items(&mut self) -> Result<Vec<Item>> {
        let mut chosen: Option<Vec<Item>> = None;
        loop {
            let matched = self.on_cond(true)?.unwrap_or(false);
            let items = self.on_item_block()?;
            if matched && chosen.is_none() {
                chosen = Some(items);
            }
            match self.on_else()? {
                Some(true) => continue,
                Some(false) => {
                    let items = self.on_item_block()?;
                    if chosen.is_none() {
                        chosen = Some(items);
                    }
                }
                None => {}
            }
            break;
        }
        Ok(chosen.unwrap_or_default())
    }

    fn on_item_block(&mut self) -> Result<Vec<Item>> {
        let open = self.expect(&Tok::LBrace, "`{` opening the branch")?;
        let mut items = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail_at(open, "this branch's `{` is never closed");
            }
            if self.is_word("on") {
                items.extend(self.on_items()?);
                continue;
            }
            push_item(&mut items, self.item()?);
        }
        self.bump();
        Ok(items)
    }

    /// `case.any` or `case.exact` here (owner, 2026-10-06; the words
    /// 2026-10-10: "(a)"), or an old spelling to refuse: `case` is no
    /// keyword, read as a mark only so.
    fn at_case_mark(&self) -> bool {
        self.is_word("case") && matches!(self.peek_at(1), Tok::Dot) && ["any", "exact", "insensitive", "sensitive"].iter().any(|w| self.is_word_at(2, w))
    }

    /// `true` for `case.any`. `case.any` and `case.exact` are
    /// the old spellings, refused naming the new.
    fn case_mark(&mut self) -> Result<(bool, Span)> {
        let at = self.span();
        self.bump();
        self.bump();
        let mode = self.bump().clone();
        match &mode.tok {
            Tok::Word(w) if w == "insensitive" => self.fail_at(at.to(mode.span), "`case.insensitive` is the old spelling: write `case.any` (owner, 2026-10-10)"),
            Tok::Word(w) if w == "sensitive" => self.fail_at(at.to(mode.span), "`case.sensitive` is the old spelling: write `case.exact` (owner, 2026-10-10)"),
            Tok::Word(w) => Ok((w == "any", at.to(mode.span))),
            _ => unreachable!("at_case_mark said a word follows"),
        }
    }

    /// A `case` mark line: `case.any;` (a file's), or
    /// `case.any [fn, var];` (by kind); `None` kinds for the first.
    fn case_line(&mut self) -> Result<(bool, Span, Option<Vec<String>>)> {
        let (mark, at) = self.case_mark()?;
        if self.eat(&Tok::Semi) {
            return Ok((mark, at, None));
        }
        if !self.eat(&Tok::LBracket) {
            return self.fail_at(at, format!("a `case` mark is `{m};` at the top of a file, `{m} [fn, var];` in `neuron.nrv`, `pub {m} fn …` on an item, or `{m} module name {{ … }}`", m = case_spelled(mark)));
        }
        let kinds_list = || CASE_KINDS.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", ");
        let mut kinds = Vec::new();
        loop {
            let Tok::Word(w) = self.peek().clone() else {
                return self.fail(format!("expected a kind ({}), found {}", kinds_list(), self.peek().describe()));
            };
            if !CASE_KINDS.contains(&w.as_str()) {
                return self.fail(format!("`{}` is not a kind a `case` mark covers: {} (a module is marked on itself)", w, kinds_list()));
            }
            self.bump();
            kinds.push(w);
            if !self.eat(&Tok::Comma) || self.is(&Tok::RBracket) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the kinds")?;
        self.expect_semi("the kinds")?;
        Ok((mark, at, Some(kinds)))
    }

    /// `bare.never`, `bare.optional` or `bare.only` (owner, 2026-10-06).
    fn bare_mark(&mut self) -> Result<(Bare, Span)> {
        let at = self.span();
        self.bump();
        if !self.eat(&Tok::Dot) {
            return self.fail_at(at, "say how other Neurons write the path: `bare.never` (required), `bare.optional` or `bare.only` (forbidden); a plain `bare` does not say");
        }
        let mode = self.ident("`never`, `optional` or `only`")?;
        let mark = match mode.name.as_str() {
            "never" => Bare::Never,
            "optional" => Bare::Optional,
            "only" => Bare::Only,
            other => return self.fail_at(mode.span, format!("no `bare.{}`: a path is `bare.never` (required), `bare.optional` or `bare.only` (forbidden)", other)),
        };
        Ok((mark, at.to(mode.span)))
    }

    /// A mark line: `bare.optional;` (a file's), or `bare.never [fn,
    /// struct];` (by kind); `None` kinds for the first.
    fn bare_line(&mut self) -> Result<(Bare, Option<Vec<String>>)> {
        let (mark, at) = self.bare_mark()?;
        if self.eat(&Tok::Semi) {
            return Ok((mark, None));
        }
        if !self.eat(&Tok::LBracket) {
            return self.fail_at(at, format!("a `bare` mark is `{m};` at the top of a file, `{m} [fn, op];` in `neuron.nrv`, or `pub {m} fn …` on an item", m = mark.spelled()));
        }
        let mut kinds = Vec::new();
        loop {
            let Tok::Word(w) = self.peek().clone() else {
                return self.fail(format!("expected a kind ({}), found {}", BARE_KINDS.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "), self.peek().describe()));
            };
            if w == "const" {
                return self.fail(CONST_TAKES_NO_MARK);
            }
            if !BARE_KINDS.contains(&w.as_str()) {
                return self.fail(format!("`{}` is not a kind a `bare` mark covers: {} (a module is marked on itself)", w, BARE_KINDS.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", ")));
            }
            self.bump();
            kinds.push(w);
            if !self.eat(&Tok::Comma) || self.is(&Tok::RBracket) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the kinds")?;
        self.expect_semi("the kinds")?;
        Ok((mark, Some(kinds)))
    }

    fn item_kind(&mut self, vis: Vis) -> Result<ItemKind> {
        let w = match self.peek().clone() {
            Tok::Word(w) => w,
            Tok::Name(n) => return self.fail(format!("a Neuron-level value is a `const`: `const '{}' := …;` — a variable lives inside a body", n)),
            other => return self.fail(format!("expected a top-level item (`fn`, `struct`, `enum`, `const`, `module`, `MAIN`), found {}", other.describe())),
        };
        match w.as_str() {
            "MAIN" => {
                self.bump();
                // C's own two (owner, 2026-09-29: "low-level shit,
                // decided"), or none: the types are checked with the rest.
                let params = if self.is(&Tok::LBracket) {
                    let at = self.span();
                    let ps = self.params(ParamContext::Fn)?;
                    match ps.len() {
                        0 | 2 => {}
                        1 => return self.fail_at(at, "`MAIN` takes C's two parameters or none: `,` and the second parameter — `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] { … }`"),
                        _ => return self.fail_at(at, "`MAIN` takes C's two parameters or none — `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] { … }`"),
                    }
                    ps
                } else {
                    Vec::new()
                };
                let body = self.block("MAIN")?;
                Ok(ItemKind::Main(params, body))
            }
            "fn" => {
                self.bump();
                if self.is_word("MAIN") {
                    return self.fail("`MAIN` is the program's block, not a function: write `MAIN { … }` with no `fn`, no return type, no `return`");
                }
                let decl = self.fn_decl(true)?;
                Ok(ItemKind::Fn(decl))
            }
            "func" => self.fail("`func` became `fn`: `fn name [params] -> type { … }`"),
            "incl" if matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Word(c) if c == "C" || c == "c") => self.incl_c(),
            "extern" => {
                self.bump();
                // `extern` names its language (owner, 2026-10-02): `extern.C`,
                // a line or a block.
                if !self.eat(&Tok::Dot) {
                    return self.fail("`extern` names its language: `extern.C fn exit ['code': int32];`, or a block, `extern.C { … }`");
                }
                let lang = self.ident("the language after `extern.`")?;
                match lang.name.as_str() {
                    "C" => {}
                    "c" => return self.fail_at(lang.span, "C is written capitalized here, as every language's name is: `extern.C`"),
                    other => return self.fail_at(lang.span, format!("only `extern.C` exists: `extern.{}` is not a language CyborgPL reaches", other)),
                }
                if self.is(&Tok::LBrace) {
                    let open = self.bump().span;
                    let mut lines = Vec::new();
                    while !self.is(&Tok::RBrace) {
                        // Another item's first word at the left margin, or the
                        // end of the file: the block was never closed.
                        if self.at_eof() || self.is_word("MAIN") || self.is_word("struct") || self.is_word("enum") || self.is_word("const") || self.is_word("module") || self.is_word("extern") || self.is_word("layout") {
                            return self.fail_at(open, "the `extern.C {` block is never closed: is its `}` missing?");
                        }
                        // A line may say its own `pub` or `pub.limb`; one
                        // before the block overrides them (owner, 2026-10-02:
                        // "allow mixing, but block overrides").
                        let mut line_vis = Vis::Private;
                        if self.eat_word("pub") {
                            line_vis = Vis::Pub;
                            if self.eat(&Tok::Dot) {
                                let narrowing = self.ident("a word after `pub.`")?;
                                if narrowing.name != "neuron" {
                                    return self.fail_at(narrowing.span, format!("`pub.{}` is not a visibility: the only narrowing of `pub` is `pub.neuron`", narrowing.name));
                                }
                                line_vis = Vis::Limb;
                            }
                        }
                        lines.push((line_vis, self.extern_line()?));
                    }
                    self.bump();
                    return Ok(ItemKind::ExternBlock(lines));
                }
                self.extern_line()
            }
            "struct" => {
                self.bump();
                Ok(ItemKind::Struct(self.struct_decl(false)?))
            }
            "layout" => {
                self.bump();
                self.expect(&Tok::Dot, "`.C` after `layout`")?;
                let which = self.ident("`C` after `layout.`")?;
                // A language's name is capitalized (owner, 2026-10-02).
                if which.name == "c" {
                    return self.fail_at(which.span, "C is written capitalized here, as every language's name is: `layout.C struct`, `layout.C enum`");
                }
                if which.name != "C" {
                    return self.fail_at(which.span, format!("the only layout is `layout.C`, C's shape; `layout.{}` does not exist", which.name));
                }
                if self.eat_word("enum") {
                    return Ok(ItemKind::Enum(self.enum_decl(true)?));
                }
                if !self.is_word("struct") {
                    return self.fail(format!("`layout.C` is followed by `struct` or `enum`, found {}", self.peek().describe()));
                }
                self.bump();
                Ok(ItemKind::Struct(self.struct_decl(true)?))
            }
            "enum" => {
                self.bump();
                Ok(ItemKind::Enum(self.enum_decl(false)?))
            }
            "interface" => {
                self.bump();
                Ok(ItemKind::Interface(self.iface_decl()?))
            }
            "op" => {
                self.bump();
                Ok(ItemKind::Op(self.op_decl()?))
            }
            "const" => {
                self.bump();
                Ok(ItemKind::Const(self.const_decl()?))
            }
            "module" => {
                self.bump();
                let name = self.ident("the module's name")?;
                if self.is(&Tok::Semi) {
                    return self.fail(format!("there is no include line: a file module is found by its file name, and `module {} {{ … }}` is a block — nothing declares a module with `;`", name.name));
                }
                self.check_module_name(&name)?;
                self.expect(&Tok::LBrace, "`{` after the module's name")?;
                let mut items = Vec::new();
                let outer_c = std::mem::take(&mut self.c_seen);
                while !self.is(&Tok::RBrace) {
                    if self.at_eof() {
                        return self.fail(format!("module `{}` is never closed", name.name));
                    }
                    if self.is_word("on") {
                        items.extend(self.on_items()?);
                        continue;
                    }
                    push_item(&mut items, self.item()?);
                }
                self.bump();
                self.c_seen = outer_c;
                Ok(ItemKind::Module { name, items })
            }
            "mod" => self.fail("`mod` is the remainder operator; a module is `module name { … }`"),
            "precedence" => {
                // Read before the items (`parse_file`); here it is passed,
                // as an empty block of lines that adds no item.
                self.precedence_block()?;
                Ok(ItemKind::ExternBlock(Vec::new()))
            }
            "let" => self.fail("a Neuron-level value is a `const`, not a `let`: `const 'name' := …;`"),
            "check" | "nocheck" => {
                if self.is_word_at(1, "fn") {
                    self.fail(format!("`{}` cannot go on a `fn`: the tiers are never global — there is no file-wide or Neuron-wide tier; put `{}` in front of the statements inside", w, w))
                } else {
                    self.fail(format!("`{}` goes in front of a statement inside a body, never at the top level: the tiers are never global", w))
                }
            }
            "inline" => self.fail("`inline.ASM` goes inside a body, as a statement, under `no_check`"),
            "var" => self.fail("`var` is gone, and a Neuron's top-level value is a `const`: `const 'name' := …;`"),
            _ => {
                // `Vec satisfies Ordered;`
                if !is_keyword(&w) {
                    let save = self.pos;
                    if let Ok(ty) = self.type_() {
                        if self.eat_word("satisfies") {
                            let iface = self.path()?;
                            self.expect_semi("the claim")?;
                            let _ = vis;
                            return Ok(ItemKind::Satisfies { ty, iface });
                        }
                    }
                    self.pos = save;
                    if self.is(&Tok::LBracket) || matches!(self.peek_at(1), Tok::LBracket) {
                        return self.fail(format!("a call to `{}` cannot stand at the top level: statements go inside `MAIN {{ }}` or a `fn`", w));
                    }
                }
                self.fail(format!("expected a top-level item (`fn`, `struct`, `enum`, `const`, `module`, `MAIN`), found `{}`", w))
            }
        }
    }

    fn check_module_name(&self, name: &Ident) -> Result<()> {
        let n = &name.name;
        let ok = n.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) && n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !ok {
            return self.fail_at(name.span, format!("`{}` cannot name a module: module names are lowercase ASCII letters, digits and `_`, starting with a letter (lowercase only, so the name works on every file system)", n));
        }
        if is_device_name(n) {
            return self.fail_at(name.span, format!("`{}` is a reserved device name on Windows and cannot name a module", n));
        }
        if is_keyword(n) {
            return self.fail_at(name.span, format!("`{}` is a keyword and cannot name a module", n));
        }
        Ok(())
    }

    /// `>` closing a `<T>`.
    fn expect_close_angle(&mut self, name: &str) -> Result<()> {
        match self.peek().clone() {
            Tok::Op(o) if o == ">" => {
                self.bump();
                Ok(())
            }
            Tok::Op(o) if o.starts_with('>') => self.fail(format!("`{}` is one operator: put a space after the `>` closing `<{}>`", o, name)),
            other => self.fail(format!("expected `>` closing `<{}>`, found {}", name, other.describe())),
        }
    }

    /// What a declaration announces right after its name (owner,
    /// 2026-10-02): each generic in its own `< >` with what it accepts
    /// after a `:` — `<T>: Number and Hashable`, `<A>: any` — then the
    /// lifetime groups, `@x @y`, with a `|` between the two kinds only.
    fn announce(&mut self, owner: &str, what: &str) -> Result<Announce> {
        let mut a = Announce::default();
        // `<T: Number, U: any>` (owner, 2026-10-07): one `< >`, each
        // blank with what it accepts, commas between.
        if self.is_op("<") && matches!(self.peek_at(1), Tok::Word(_)) {
            self.bump();
            loop {
                let name = self.ident("the generic's name")?;
                if self.is_op(">") && matches!(self.peek_at(1), Tok::Colon) {
                    return self.fail_at(name.span, format!("what `<{n}>` accepts goes inside the brackets: `<{n}: any>`", n = name.name));
                }
                if !self.eat(&Tok::Colon) {
                    return self.fail_at(name.span, format!("`<{n}>` says what it accepts, never every type by default: `{w} {o} <{n}: any>`, or an interface — `<{n}: Number>`", n = name.name, w = what, o = owner));
                }
                let mut bounds = Vec::new();
                if self.is_word("any") {
                    let any = self.bump().span;
                    if self.is_word("and") {
                        return self.fail_at(any, format!("`any` stands alone: `<{}: any>` accepts every type, so nothing joins it with `and`", name.name));
                    }
                } else {
                    loop {
                        if self.is_word("any") {
                            return self.fail(format!("`any` stands alone: `<{}: any>` accepts every type, so nothing joins it with `and`", name.name));
                        }
                        bounds.push(self.path()?);
                        if !self.eat_word("and") {
                            break;
                        }
                    }
                }
                a.generics.push(Generic { name, bounds });
                if self.eat(&Tok::Comma) {
                    if self.is_op(">") {
                        break;
                    }
                    continue;
                }
                break;
            }
            let last = a.generics.last().map(|g| g.name.name.clone()).unwrap_or_default();
            self.expect_close_angle(&last)?;
            if self.is_op("<") && matches!(self.peek_at(1), Tok::Word(_)) {
                return self.fail("the blanks go in one `< >`, commas between: `<A: any, B: Hashable>`");
            }
        }
        // The lifetime groups in a bracket of their own, before the
        // parameters (owner, 2026-10-07): `[@x, @y]`.
        if self.is_op("|") || self.is_op("@") {
            return self.fail(format!("lifetime groups go in a bracket of their own, before the parameters: `{} {} [@x, @y] [...]`", what, owner));
        }
        if self.is(&Tok::LBracket) && self.is_op_at(1, "@") {
            self.bump();
            loop {
                let at = self.span();
                if !self.eat_op("@") {
                    return self.fail_at(at, "a lifetime group is named after `@`: `[@x, @y]`");
                }
                match self.peek().clone() {
                    Tok::Word(_) => a.groups.push(self.ident("the lifetime group's name")?),
                    _ => return self.fail_at(at, "a lifetime group is named after `@`: `@x`"),
                }
                if !self.eat(&Tok::Comma) || self.is(&Tok::RBracket) {
                    break;
                }
            }
            self.expect(&Tok::RBracket, "`]` closing the lifetime groups")?;
        }
        Ok(a)
    }

    /// `incl.C["vendor/mylib.h"];` or `incl.C.system["stdio.h", …];`
    /// (owner, 2026-10-10): the `extern.C` block Synapse writes from the
    /// headers (`cheader.rs`), read here as if it stood on the line, every
    /// part of it at the line.
    fn incl_c(&mut self) -> Result<ItemKind> {
        let start = self.bump().span;
        self.bump();
        let lang = self.ident("`C`")?;
        if lang.name == "c" {
            return self.fail_at(lang.span, "C is written capitalized here, as every language's name is: `incl.C`");
        }
        let system = if self.eat(&Tok::Dot) {
            let w = self.ident("`system`")?;
            if w.name != "system" {
                return self.fail_at(w.span, format!("`incl.C.{}` is not a kind of header: `incl.C.system[\"stdio.h\"]` is the system's, `incl.C[\"vendor/mylib.h\"]` one in the Neuron", w.name));
            }
            true
        } else {
            false
        };
        self.expect(&Tok::LBracket, "`[` and the headers, in quotes")?;
        let mut paths = Vec::new();
        while !self.is(&Tok::RBracket) {
            let Tok::Str(p) = self.peek().clone() else {
                return self.fail("a header is written in quotes, as text: `incl.C.system[\"stdio.h\"]`");
            };
            self.bump();
            paths.push(p);
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the headers")?;
        if paths.is_empty() {
            return self.fail("`incl.C` names at least one header: `incl.C.system[\"stdio.h\"]`");
        }
        let line = start.to(self.prev_span());
        self.expect_semi("the `incl.C` line")?;
        let spelled = format!("`incl.C{}[{}]`", if system { ".system" } else { "" }, paths.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", "));
        let root = NEURON_ROOT.with(|r| r.borrow().clone());
        let decls = crate::cheader::declare(system, &paths, &root).map_err(|why| Box::new(Diagnostic::error(format!("{}: {}", spelled, why)).at(line)))?;
        let mut text = String::new();
        for d in decls {
            match self.c_seen.get(&d.name) {
                Some(t) if *t == d.text => continue,
                _ => {}
            }
            self.c_seen.insert(d.name, d.text.clone());
            text.push_str(&d.text);
            text.push('\n');
        }
        let mut toks = crate::lexer::lex(line.file, &text)?;
        for t in toks.iter_mut() {
            t.span = line;
        }
        let mut p = Parser { toks: &toks, pos: 0, strict: false, returns_value: false, ret_lent: None, prec: self.prec.clone(), no_std: false, cores: None, c_seen: Default::default(), from_c: true };
        let mut lines = Vec::new();
        while !p.at_eof() {
            let it = p.item()?;
            match it.kind {
                ItemKind::ExternBlock(more) => lines.extend(more),
                kind => lines.push((Vis::Private, kind)),
            }
        }
        Ok(ItemKind::ExternBlock(lines))
    }

    /// One of C's names after `extern.C`, alone or in its block: `fn …;`,
    /// or a variable, `'x': T;` (`mut 'x': T;` to write it; owner,
    /// 2026-09-25).
    fn extern_line(&mut self) -> Result<ItemKind> {
        if self.eat_word("fn") {
            let decl = self.fn_decl(false)?;
            return Ok(ItemKind::ExternFn(decl));
        }
        if self.is_word("let") {
            return self.fail("C's variable has no `let`: `extern.C 'x': T;` (`extern.C mut 'x': T;` to write it)");
        }
        if !(matches!(self.peek(), Tok::Name(_)) || self.is_word("mut") || self.is_word("immut")) {
            return self.fail("`extern.C` is followed by `fn` (a C function) or a quoted name (a C variable: `extern.C 'x': int32;`)");
        }
        let mutable = self.eat_word("mut");
        if self.is_word("immut") {
            return self.fail("`immut` means nothing on C's variable: it is read-only unless you write `mut`");
        }
        let name = self.qname("the C variable's name")?;
        if !self.is(&Tok::Colon) {
            return self.fail("C's variable needs a type: it has a C-shaped type and no value here — `extern.C 'x': int32;`");
        }
        self.bump();
        let ty = self.type_()?;
        if self.is_op("=") || self.is(&Tok::Walrus) {
            return self.fail("C's variable has no value here: C owns it; write only its type");
        }
        self.expect_semi("the extern declaration")?;
        Ok(ItemKind::ExternLet { mutable, name, ty })
    }

    fn fn_decl(&mut self, has_body: bool) -> Result<FnDecl> {
        if let Tok::Name(n) = self.peek().clone() {
            return self.fail(format!("function names are bare: `fn {} […]`, not `fn '{}'` — quotes mark a value you hold, and a function is not one", n, n));
        }
        if self.is(&Tok::Dot) {
            return self.fail("the return type goes after the parameters — `fn name [params] -> int32 { … }` — not on `fn`");
        }
        // A keyword may name a function — `fn true`, `fn to` — reached by
        // path, never bare (limbs.md, 2026-09-19): `limb.true[…]`.
        let mut name = self.ident("the function's name")?;
        // `fn Counter.next […]`: a function belonging to a type (owner,
        // 2026-10-09: "`b`"), called `Counter.next[…]`.
        let mut owner = None;
        if has_body && self.is(&Tok::Dot) && matches!(self.peek_at(1), Tok::Word(_)) {
            self.bump();
            let own = std::mem::replace(&mut name, self.ident("the function's name after its type")?);
            owner = Some(own);
        }
        let announce = self.announce(&name.name, "fn")?;
        let params = self.params(if has_body { ParamContext::Fn } else { ParamContext::Extern })?;
        if self.is(&Tok::Colon) {
            return self.fail("the return type is `-> int32`, not `: int32` — `:` attaches a type to a name, and a result is not the function's type");
        }
        let (ret, fails) = self.return_spec()?;
        let ret_lent = self.ret_lent.take();
        // A returned borrow names its group only when there is a choice
        // (owner, 2026-10-07: "`b`"): two or more borrowed inputs.
        let borrowed = params.iter().filter(|p| matches!(p.kind, ParamKind::Lent | ParamKind::LentMut)).count();
        if let (true, Some(RetLent { group: None, span, .. })) = (borrowed >= 2, &ret_lent) {
            return self.fail_at(*span, format!("`{n}` hands back a borrow and borrows more than one input: say which with a group, `fn {n} [@x] [lent @x 'a': …, lent 'b': …] -> lent @x …`", n = name.name));
        }
        let body = if has_body {
            if self.is(&Tok::Semi) {
                return self.fail(format!("`{}` has no body: a function without one is an `extern fn`", name.name));
            }
            let outer = std::mem::replace(&mut self.returns_value, false);
            let b = self.block("the function")?;
            // Every return is typed (owner, 2026-10-07: "`a`").
            if self.returns_value && ret.is_none() {
                return self.fail_at(name.span, format!("`{}` returns a value, so it says its type: `-> int32`", name.name));
            }
            self.returns_value = outer;
            Some(b)
        } else {
            if self.is(&Tok::LBrace) {
                return self.fail(format!("an `extern fn` has no body: `{}`'s body is in C — end the declaration with `;`", name.name));
            }
            self.expect_semi("the extern declaration")?;
            None
        };
        Ok(FnDecl { name, owner, announce, params, ret, ret_lent, fails, body, test: false })
    }

    /// The group's name, right after its `@`.
    /// `lent @x`, the group spaced from `lent` (owner, 2026-10-07).
    fn lent_group(&mut self) -> Result<Option<Ident>> {
        if !self.is_op("@") {
            return Ok(None);
        }
        if self.prev_span().end == self.span().start {
            return self.fail("a space before the group: `lent @x`");
        }
        self.bump();
        Ok(Some(self.group_after_at("lent @x")?))
    }

    fn group_after_at(&mut self, shape: &str) -> Result<Ident> {
        match self.peek().clone() {
            Tok::Word(_) => self.ident("the lifetime group's name after `@`"),
            _ => self.fail(format!("a lifetime group is named after `@`: `{}`", shape)),
        }
    }

    /// `-> T`, `-> T or fail[…]`, `-> fail[…]`, or nothing.
    fn return_spec(&mut self) -> Result<(Option<Type>, Vec<FailSpec>)> {
        if let Tok::Op(o) = self.peek().clone() {
            if o.starts_with("->") && o != "->" {
                return self.fail(format!("`{}` is one operator: put a space after `->` (`-> <T>`)", o));
            }
        }
        self.ret_lent = None;
        if !self.eat_op("->") {
            return Ok((None, Vec::new()));
        }
        // `-> lent T`, `-> lent.mut@x T`: a borrow handed back (owner,
        // 2026-09-29). `lent[…]` would be a call, and never follows `->`.
        if let Tok::Op(o) = self.peek().clone() {
            if o == "&" || o == "&@" {
                let start = self.bump().span;
                let mutable = o == "&" && self.eat_word("mut");
                let group = if o == "&@" {
                    Some(self.group_after_at("&@x")?)
                } else if self.eat_op("@") {
                    Some(self.group_after_at(if mutable { "&mut@x" } else { "&@x" })?)
                } else {
                    None
                };
                self.ret_lent = Some(RetLent { mutable, group, span: start.to(self.prev_span()) });
            }
        }
        if self.ret_lent.is_none() && self.is_word("lent") {
            let start = self.bump().span;
            let mutable = if self.eat(&Tok::Dot) {
                let m = self.ident("`mut` after `lent.`")?;
                if m.name != "mut" {
                    return self.fail_at(m.span, format!("`lent.{}` is not a borrow: `-> lent T` or `-> lent.mut T`", m.name));
                }
                true
            } else {
                false
            };
            let group = self.lent_group()?;
            self.ret_lent = Some(RetLent { mutable, group, span: start.to(self.prev_span()) });
        }
        if self.is_word("fail") {
            self.bump();
            let fails = self.fail_list()?;
            return Ok((None, fails));
        }
        let ret = self.type_()?;
        let mut fails = Vec::new();
        if self.is_word("or") {
            self.bump();
            self.expect_word("fail")?;
            fails = self.fail_list()?;
        }
        Ok((Some(ret), fails))
    }

    /// `[a, b: T]` after `fail`.
    fn fail_list(&mut self) -> Result<Vec<FailSpec>> {
        self.expect(&Tok::LBracket, "`[` after `fail`")?;
        let mut specs = Vec::new();
        loop {
            if self.is(&Tok::RBracket) {
                break;
            }
            if let Tok::Name(n) = self.peek().clone() {
                return self.fail(format!("a failure's name is bare — `fail[{}]`, not `fail['{}']` — it is a reason, not a value you hold", n, n));
            }
            let name = self.ident("a failure name")?;
            let payload = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
            specs.push(FailSpec { name, payload });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the failure list")?;
        if specs.is_empty() {
            return self.fail("`fail[]` lists no failure: name at least one reason");
        }
        Ok(specs)
    }

    fn params(&mut self, ctx: ParamContext) -> Result<Vec<Param>> {
        if !self.is(&Tok::LBracket) {
            if ctx == ParamContext::Fn && self.is_op("=") {
                return self.fail("not yet: `pub fn short = module.name;` gives other Neurons a shorter name (owner, 2026-09-23), and Synapse builds one Neuron so far");
            }
            return self.fail(format!("expected `[` starting the parameters, found {}", self.peek().describe()));
        }
        self.bump();
        let mut params = Vec::new();
        loop {
            if self.is(&Tok::RBracket) {
                break;
            }
            let p = self.param(ctx)?;
            let variadic = p.variadic;
            params.push(p);
            if !self.eat(&Tok::Comma) {
                break;
            }
            if variadic && !self.is(&Tok::RBracket) {
                return self.fail("a variadic `...'name'` must be the last parameter: the rest of the arguments go to it");
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the parameters")?;
        // Every parameter is typed (owner, 2026-10-07: "`a`"); an untyped
        // variadic keeps each argument's own (design claude's reading).
        if let Some(p) = params.iter().find(|p| p.ty.is_none() && !p.variadic) {
            return self.fail_at(p.name.span, format!("parameter '{}' says its type: `'{}': int32`", p.name.name, p.name.name));
        }
        Ok(params)
    }

    fn param(&mut self, ctx: ParamContext) -> Result<Param> {
        let start = self.span();
        // An operator's operands are lent unless the op says `give`.
        let mut kind = if ctx == ParamContext::Op { ParamKind::Lent } else { ParamKind::Owned };
        let mut amp = false;
        let mut mutability = None;
        let mut variadic = false;
        let mut group = None;
        match self.peek().clone() {
            Tok::Word(w) if w == "lent" => {
                self.bump();
                if self.eat(&Tok::Dot) {
                    let m = self.ident("`mut` after `lent.`")?;
                    if m.name != "mut" {
                        return self.fail_at(m.span, format!("`lent.{}` is not a parameter kind: `lent` (looks at it) or `lent.mut` (may modify it)", m.name));
                    }
                    kind = ParamKind::LentMut;
                } else {
                    kind = ParamKind::Lent;
                }
                group = self.lent_group()?;
                if ctx == ParamContext::Op {
                    return self.fail_at(start, "an operator's operands are lent by default: write `'a': T` with no `lent`; `give 'a': T` takes it by value");
                }
                if ctx == ParamContext::Extern {
                    return self.fail_at(start, "an `extern fn` parameter is not `lent`: C receives values and pointers, so write `'p': ptr[T]` and hand it a lend at the call");
                }
            }
            // `&` is `lent` everywhere (owner, 2026-10-02): `&@x`, `&mut@x`.
            Tok::Op(o) if o == "&" || o == "&@" => {
                self.bump();
                amp = true;
                if o == "&@" {
                    kind = ParamKind::Lent;
                    group = Some(self.group_after_at("&@x")?);
                } else if self.eat_word("mut") {
                    kind = ParamKind::LentMut;
                    if self.eat_op("@") {
                        group = Some(self.group_after_at("&mut@x")?);
                    }
                } else {
                    kind = ParamKind::Lent;
                }
            }
            Tok::Word(w) if w == "lend" => {
                return self.fail("a parameter that receives a borrow is `lent 'x': T` (the caller writes `lend`); `lend` is the caller's word");
            }
            Tok::Word(w) if w == "give" => {
                self.bump();
                if ctx != ParamContext::Op {
                    return self.fail_at(start, "an owning parameter has no keyword — `'x': T` — and the caller writes `give` or `copy`");
                }
                kind = ParamKind::Owned;
                let p = self.param_rest(start, kind, amp, mutability, true)?;
                return Ok(p);
            }
            Tok::Word(w) if w == "var" => {
                return self.fail("a parameter has no `var`: an owning parameter is bare — `'a': int32`");
            }
            Tok::Word(w) if w == "mut" || w == "immut" => {}
            Tok::Word(w) if is_primitive_head(&w) || (!is_keyword(&w) && matches!(self.peek_at(1), Tok::Name(_))) => {
                return self.fail("a parameter is written name first, then its type: `'a': int32`");
            }
            Tok::Ellipsis => {
                self.bump();
                variadic = true;
            }
            _ => {}
        }
        if self.is_word("mut") {
            self.bump();
            mutability = Some(Mutability::Mut);
        } else if self.is_word("immut") {
            self.bump();
            mutability = Some(Mutability::Immut);
        }
        if !variadic && self.is(&Tok::Ellipsis) {
            self.bump();
            variadic = true;
        }
        let mut p = self.param_rest(start, kind, amp, mutability, false)?;
        p.variadic = variadic;
        p.group = group;
        if variadic && ctx == ParamContext::Extern {
            // C's own varargs: allowed, with no type.
        }
        Ok(p)
    }

    fn param_rest(&mut self, start: Span, kind: ParamKind, amp: bool, mutability: Option<Mutability>, gives: bool) -> Result<Param> {
        let _ = gives;
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                let span = self.bump().span;
                QName { name: n, span }
            }
            Tok::Word(w) if w == "lent" && matches!(self.peek_at(1), Tok::Dot) => {
                return self.fail("`lent` and the type both follow their own rules: `lent 'text': str.utf8` — the verb before the name, the type after it");
            }
            Tok::Word(w) => {
                return self.fail(format!("a parameter's name is quoted: `'{}': int32`", w));
            }
            other => return self.fail(format!("expected a parameter name, found {}", other.describe())),
        };
        if self.is(&Tok::Ellipsis) {
            return self.fail("a variadic parameter is `...'name'`: the dots go before the name");
        }
        let ty = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
        let span = start.to(self.prev_span());
        Ok(Param { kind, group: None, amp, mutability, variadic: false, name, ty, span })
    }

    fn struct_decl(&mut self, layout_c: bool) -> Result<StructDecl> {
        let name = self.ident("the struct's name")?;
        if let Some(t) = crate::resolve::rust_alias(&name.name) {
            return self.fail_at(name.span, format!("`{}` is another name for `{}` (Rust's spelling of it), so it cannot name a struct", name.name, t));
        }
        let announce = self.announce(&name.name, "struct")?;
        if !announce.groups.is_empty() {
            return self.fail_at(announce.groups[0].span, "not yet: a struct holding a borrow, and so a lifetime group, is not decided (asked)");
        }
        if self.from_c && layout_c && self.eat(&Tok::Semi) {
            return Ok(StructDecl { name, announce, layout_c, fields: Vec::new(), opaque: true });
        }
        self.expect(&Tok::LBrace, "`{` after the struct's name")?;
        let mut fields = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail(format!("struct `{}` is never closed", name.name));
            }
            let mut immut = false;
            if self.is_word("immut") {
                self.bump();
                if self.is(&Tok::Dot) {
                    return self.fail("`immut` goes before the field's name: `immut number: int32`");
                }
                immut = true;
            } else if self.is_word("mut") {
                return self.fail("a struct field is not `mut`: a struct always owns its fields and may change them unless the field is `immut`");
            } else if self.is_word("let") || self.is_word("var") {
                return self.fail("a field has no `let`: a struct always owns its fields — write `name: type`");
            }
            let fname = match self.peek().clone() {
                Tok::Word(w) => {
                    if is_keyword(&w) && !self.from_c && !matches!(self.peek_at(1), Tok::Name(_)) {
                        if matches!(self.peek_at(1), Tok::Colon) {
                            return self.fail(format!("`{}` is a keyword and cannot name a field", w));
                        }
                    }
                    if !matches!(self.peek_at(1), Tok::Colon) {
                        if let Tok::Name(n) = self.peek_at(1).clone() {
                            return self.fail(format!("a field is written name first, then its type: `{}: {}`", n, w));
                        }
                        return self.fail(format!("a field's type follows its name after `:` — `{}: int32`", w));
                    }
                    self.ident("a field name")?
                }
                Tok::Name(n) => return self.fail(format!("a field's name is bare — `{}: int32` — it is a path into the struct, not a value you hold", n)),
                other => return self.fail(format!("expected a field (`name: type`) or `}}`, found {}", other.describe())),
            };
            self.expect(&Tok::Colon, "`:` after the field's name")?;
            let ty = self.type_()?;
            if self.is(&Tok::Semi) {
                return self.fail("no `;` after a field: fields are not statements — one per line, nothing between");
            }
            if self.is(&Tok::Comma) {
                return self.fail("no `,` between fields: one per line, nothing between");
            }
            fields.push(Field { immut, name: fname, ty });
        }
        self.bump();
        Ok(StructDecl { name, announce, layout_c, fields, opaque: false })
    }

    fn enum_decl(&mut self, layout_c: bool) -> Result<EnumDecl> {
        let name = self.ident("the enum's name")?;
        if let Some(t) = crate::resolve::rust_alias(&name.name) {
            return self.fail_at(name.span, format!("`{}` is another name for `{}` (Rust's spelling of it), so it cannot name a enum", name.name, t));
        }
        let announce = self.announce(&name.name, "enum")?;
        if !announce.groups.is_empty() {
            return self.fail_at(announce.groups[0].span, "not yet: an enum holding a borrow, and so a lifetime group, is not decided (asked)");
        }
        self.expect(&Tok::LBrace, "`{` after the enum's name")?;
        let mut cases = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail(format!("enum `{}` is never closed", name.name));
            }
            let cname = match self.peek().clone() {
                Tok::Word(_) => self.ident("a case name")?,
                Tok::Name(n) => return self.fail(format!("a case's name is bare — `{}` — it is a path into the type, not a value you hold", n)),
                Tok::Comma => return self.fail("no `,` between cases: one per line, nothing between"),
                other => return self.fail(format!("expected a case name or `}}`, found {}", other.describe())),
            };
            let payload = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
            // `a = 65`: a C enum's case says its number (owner, 2026-09-27).
            let number = if self.is(&Tok::Op("=".into())) {
                let at = self.span();
                self.bump();
                let negative = self.is(&Tok::Op("-".into()));
                if negative {
                    self.bump();
                }
                let n = match self.peek().clone() {
                    Tok::Num { text, bin: false } => {
                        self.bump();
                        let v: Option<i128> = text.parse().ok();
                        match v {
                            Some(v) => v,
                            None => return self.fail(format!("C's enum is 32 bits, C's `int`: {} does not fit int32", text)),
                        }
                    }
                    other => return self.fail(format!("a C enum's case number is a whole number, found {}", other.describe())),
                };
                Some((if negative { -n } else { n }, at.to(self.prev_span())))
            } else {
                None
            };
            if self.is(&Tok::Comma) {
                return self.fail("no `,` between cases: one per line, nothing between");
            }
            if self.is(&Tok::Semi) {
                return self.fail("no `;` after a case: cases are not statements");
            }
            cases.push(EnumCase { name: cname, payload, number });
        }
        self.bump();
        if cases.is_empty() {
            return self.fail_at(name.span, format!("enum `{}` has no cases: an enum is one of a few named cases, so name at least one", name.name));
        }
        Ok(EnumDecl { name, announce, cases, layout_c })
    }

    fn iface_decl(&mut self) -> Result<IfaceDecl> {
        let name = self.ident("the interface's name")?;
        if let Some(t) = crate::resolve::rust_alias(&name.name) {
            return self.fail_at(name.span, format!("`{}` is another name for `{}` (Rust's spelling of it), so it cannot name a interface", name.name, t));
        }
        if name.name == "any" {
            return self.fail_at(name.span, "`any` is the built-in that accepts every type (`<T>: any`), so it cannot name an interface");
        }
        // Its own `<T>`, which accepts nothing in particular: the bound is
        // the interface itself (owner, 2026-10-02).
        let own = if self.is_op("<") {
            self.bump();
            let t = self.ident("the interface's type's name")?;
            if self.is(&Tok::Colon) {
                return self.fail(format!("an interface's own `<{}>` accepts nothing in particular — the interface is what it accepts: `interface {} <{}> {{ … }}`", t.name, name.name, t.name));
            }
            self.expect_close_angle(&t.name)?;
            Some(t)
        } else {
            None
        };
        let open = self.expect(&Tok::LBrace, "`{` after the interface's name")?;
        let mut entries = Vec::new();
        let mut types: Vec<Type> = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail(format!("interface `{}` is never closed", name.name));
            }
            let start = self.span();
            let ename = if self.eat_word("fn") {
                IfaceEntryName::Fn(self.ident("the function's name")?)
            } else if self.eat_word("op") {
                IfaceEntryName::Op(self.op_name()?)
            } else if matches!(self.peek(), Tok::Word(_)) {
                // A type it accepts, one per line, as enum cases go.
                if !types.is_empty() && !self.toks[self.pos].line_first {
                    return self.fail(format!("an interface lists its types one per line, as an enum lists its cases: `interface {} {{` and then each type on a line of its own", name.name));
                }
                types.push(self.type_()?);
                continue;
            } else {
                return self.fail(format!("an interface lists types, one per line, and `fn` and `op` signatures; found {}", self.peek().describe()));
            };
            let params = self.params(ParamContext::Iface)?;
            let (ret, fails) = self.return_spec()?;
        if let Some(r) = self.ret_lent.take() {
            return self.fail_at(r.span, "not yet: only a named `fn` hands back a borrow (`fn f [lent 'a': T] -> lent T`)");
        }
            if self.is(&Tok::LBrace) {
                return self.fail("an interface lists promises, not code: end the signature with `;` and leave the body out");
            }
            self.expect_semi("the signature")?;
            let span = start.to(self.prev_span());
            entries.push(IfaceEntry { name: ename, params, ret, fails, span });
        }
        self.bump();
        if entries.is_empty() && types.is_empty() {
            return self.fail_at(open.to(self.prev_span()), format!("interface `{}` promises nothing: list at least one `fn` or `op` signature, or the types it accepts, one per line", name.name));
        }
        if own.is_none() && !entries.is_empty() {
            return self.fail_at(name.span, format!("interface `{}` lists operations, so it announces the type they are about: `interface {} <T> {{ … }}`", name.name, name.name));
        }
        Ok(IfaceDecl { name, own, entries, types })
    }

    fn op_name(&mut self) -> Result<Ident> {
        let op = self.op_spelling()?;
        if op.name == "->" || op.name == "<-" || op.name == "<->" {
            return self.fail_at(op.span, "an arrow cannot name an operator: `->`, `<-` and `<->` are arrows (owner, 2026-10-07)");
        }
        // A type's own `plus` is `op plus` (owner, 2026-10-03).
        if let Some(word) = retired_spelling(&op.name) {
            return self.fail_at(op.span, format!("`op {}` is `op {}` now (owner, 2026-10-03): `op {} ['a': V, 'b': V] -> V {{ … }}`", op.name, word, word));
        }
        Ok(op)
    }

    fn op_spelling(&mut self) -> Result<Ident> {
        match self.peek().clone() {
            Tok::Op(o) => {
                let span = self.bump().span;
                // A comparison is named by its word (owner, 2026-10-07).
                let name = match o.as_str() {
                    "==" => "equal_to".to_string(),
                    "!==" => "not_equal".to_string(),
                    "</==" => "at_most".to_string(),
                    ">/==" => "at_least".to_string(),
                    _ => o,
                };
                Ok(Ident { name, span })
            }
            Tok::Word(w) => {
                let span = self.bump().span;
                let mut name = w;
                // `and.bit` style word operators are the language's; a user op is one word.
                if self.is(&Tok::Dot) {
                    self.bump();
                    let n = self.ident("a word after `.`")?;
                    name = format!("{}.{}", name, n.name);
                }
                Ok(Ident { name, span: span.to(self.prev_span()) })
            }
            other => self.fail(format!("expected the operator's spelling (a glyph run or a word), found {}", other.describe())),
        }
    }

    fn op_decl(&mut self) -> Result<OpDecl> {
        let op = self.op_name()?;
        let announce = self.announce(&op.name, "op")?;
        let params = self.params(ParamContext::Op)?;
        if params.len() != 2 {
            return self.fail_at(op.span, format!("an operator takes exactly two operands; `{}` is declared with {}", op.name, params.len()));
        }
        if !self.is_op("->") {
            return self.fail(format!("`-> T` is required on an operator: `op {} [...] -> T {{ … }}`", op.name));
        }
        let (ret, fails) = self.return_spec()?;
        if let Some(r) = self.ret_lent.take() {
            return self.fail_at(r.span, "not yet: only a named `fn` hands back a borrow (`fn f [lent 'a': T] -> lent T`)");
        }
        if !fails.is_empty() {
            return self.fail_at(op.span, format!("an operator cannot fail; make `{}` a `fn` if it needs `fail[…]`", op.name));
        }
        if ret.is_none() {
            return self.fail_at(op.span, format!("`-> T` is required on an operator: `op {} [...] -> T {{ … }}`", op.name));
        }
        let body = self.block("the operator")?;
        Ok(OpDecl { op, announce, params, ret, body })
    }

    fn const_decl(&mut self) -> Result<ConstDecl> {
        if self.is_word("mut") || self.is_word("immut") {
            return self.fail("a `const` is neither `mut` nor `immut`: it is a value known at compile time, never assigned");
        }
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                let span = self.bump().span;
                QName { name: n, span }
            }
            Tok::Word(w) => return self.fail(format!("a const's name is quoted, like a variable's: `const '{}' := …;`", w)),
            other => return self.fail(format!("expected the const's name, found {}", other.describe())),
        };
        let (ty, _shadow) = self.let_type_and_marker("const")?;
        let value = self.expr_top()?;
        self.expect_semi("the const")?;
        Ok(ConstDecl { name, ty, value })
    }

    /// After a `let`/`const` name: `:= `, `: T =`, and the `$` forms.
    fn let_type_and_marker(&mut self, what: &str) -> Result<(Option<Type>, bool)> {
        let ty = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
        // Shadowing is the verb `shadow` in `let`'s place since 2026-10-07
        // (owner: "yea"); `$` is gone (the pre-scan names it).
        let _ = what;
        let shadow = false;
        if self.eat(&Tok::Walrus) {
            if ty.is_some() {
                return self.fail_at(self.prev_span(), "`:=` means Cortex chooses the type, so it takes no type; with a type written, use `=` — you said everything");
            }
        } else if self.eat_op("=") {
            if ty.is_none() {
                return self.fail_at(self.prev_span(), "`=` means you said everything, so it needs the type: `let 'x': int32 = …;` — or `:=` to let Cortex choose");
            }
        } else if self.is(&Tok::Semi) {
            return self.fail(format!("a {} needs a value: there is no implicit zero — write `= 0` (or whatever it should hold)", what));
        } else if self.is_word("mut") || self.is_word("immut") {
            return self.fail(format!("expected `:=` or `=`; `{}` goes before the name: `{} 'x' := …`", self.word_text(), self.word_text()));
        } else {
            return self.fail(format!("expected `:=` or `=`, found {}", self.peek().describe()));
        }
        Ok((ty, shadow))
    }

    fn word_text(&self) -> String {
        match self.peek() {
            Tok::Word(w) => w.clone(),
            _ => String::new(),
        }
    }

    // ------------------------------------------------------------ types

    fn type_(&mut self) -> Result<Type> {
        self.type_in(false)
    }

    /// `conv`: after `to.`, where a final non-bound bracket is the value to convert.
    fn type_in(&mut self, conv: bool) -> Result<Type> {
        let start = self.span();
        let kind = match self.peek().clone() {
            Tok::Num { text, bin } => {
                let span = self.bump().span;
                if bin {
                    return self.fail_at(span, "a count is a whole number");
                }
                match text.parse::<u32>() {
                    Ok(n) => TypeKind::Count(n),
                    Err(_) => return self.fail_at(span, "this number is too large to be a count"),
                }
            }
            Tok::Op(o) if o == "<" => {
                self.bump();
                let name = match self.peek().clone() {
                    Tok::Word(_) => self.ident("the blank's name")?,
                    Tok::Name(n) => return self.fail(format!("a type variable's name is bare: `<{}>`, not `<'{}'>`", n, n)),
                    other => return self.fail(format!("expected the blank's name after `<`, found {}", other.describe())),
                };
                if self.is(&Tok::Colon) {
                    return self.fail(format!("what `<{}>` accepts is said once, where it is announced (`fn f <{}>: any […]`): a mention is `<{}>` alone", name.name, name.name, name.name));
                }
                let bound = None;
                match self.peek().clone() {
                    Tok::Op(o) if o == ">" => {
                        self.bump();
                    }
                    Tok::Op(o) if o.starts_with('>') => {
                        return self.fail(format!("`{}` is one operator: put a space after the `>` closing `<{}>`", o, name.name));
                    }
                    other => return self.fail(format!("expected `>` closing `<{}>`, found {}", name.name, other.describe())),
                }
                TypeKind::Var { name, bound }
            }
            Tok::Op(o) if o.starts_with('<') => {
                return self.fail(format!("`{}` is one operator: put a space around a `<T>` so `<` stands alone", o));
            }
            Tok::Word(w) if w == "fn" => {
                self.bump();
                self.expect(&Tok::LBracket, "`[` after `fn` in a function type")?;
                let mut params = Vec::new();
                while !self.is(&Tok::RBracket) {
                    let mut kind = ParamKind::Owned;
                    let mut amp = false;
                    if self.eat_word("lent") {
                        kind = ParamKind::Lent;
                        if self.eat(&Tok::Dot) {
                            let m = self.ident("`mut`")?;
                            if m.name != "mut" {
                                return self.fail_at(m.span, "`lent` or `lent.mut`");
                            }
                            kind = ParamKind::LentMut;
                        }
                    } else if self.is_op("&") {
                        self.bump();
                        amp = true;
                        kind = if self.eat_word("mut") { ParamKind::LentMut } else { ParamKind::Lent };
                    }
                    if let Tok::Name(n) = self.peek().clone() {
                        return self.fail(format!("a function type has no parameter names: `fn[int32] -> bool`, not `fn['{}': int32]` — it is the signature without the names", n));
                    }
                    let ty = self.type_()?;
                    params.push(FnTypeParam { kind, amp, ty });
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RBracket, "`]` closing the function type's parameters")?;
                let (ret, fails) = self.return_spec()?;
        if let Some(r) = self.ret_lent.take() {
            return self.fail_at(r.span, "not yet: only a named `fn` hands back a borrow (`fn f [lent 'a': T] -> lent T`)");
        }
                TypeKind::Fn { params, ret: ret.map(Box::new), fails }
            }
            Tok::Word(w) if w == "list" && matches!(self.peek_at(1), Tok::Dot) => {
                return self.fail("the element type goes in brackets: `list[int32]`, not `list.int32` (a dot picks a fixed variant, brackets supply a type)");
            }
            Tok::Word(w) if w == "int" || w == "uint" || w == "bin" || w == "deci" => {
                return self.fail(format!("`{}` needs a width: the typed form is always fully explicit — `{}32`, `{}64`, …", w, w, w));
            }
            Tok::Word(w) if w == "str" && !matches!(self.peek_at(1), Tok::Dot) => {
                return self.fail("`str` is never bare: say the encoding — `str.utf8`, `str.utf16`, `str.utf32`, `str.ascii`");
            }
            Tok::Word(w) if w == "char" && !matches!(self.peek_at(1), Tok::Dot) => {
                return self.fail("`char` is never bare: say the unit — `char.point`, `char.grapheme`, `char.byte`");
            }
            Tok::Word(w) if w == "range" => {
                return self.fail("`range` builds its bounds from values: `for 'i': int32 in range[0, 3]` — the type goes on the counter's name");
            }
            // An integer of any width, marked as not standard (owner,
            // 2026-10-03): `!std.int24`, "not standard" narrowed with `.`
            // (owner, 2026-10-06; `!std_int24` before). One name inside; the
            // resolver reads the width.
            Tok::Op(o) if o == "!" && matches!(self.peek_at(1), Tok::Word(w) if w.starts_with("std_")) && self.adjacent(0, 1) => {
                let Tok::Word(w) = self.peek_at(1).clone() else { unreachable!() };
                return self.fail(format!("`!{}` is `not_std.{}` since 2026-10-06: \"not standard\", narrowed with `.`", w, &w[4..]));
            }
            // `not_std.int24` (owner, 2026-10-07; `!std.int24` before).
            Tok::Word(n) if n == "not_std" && matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Word(_)) => {
                let at = self.bump().span;
                self.bump();
                let Tok::Word(w) = self.peek().clone() else { unreachable!() };
                let wspan = self.bump().span;
                let name = Ident { name: format!("!std_{}", w), span: at.to(wspan) };
                TypeKind::Path { path: Path { limb: None, segs: vec![name], span: at.to(wspan) }, args: Vec::new() }
            }
            Tok::Word(_) => {
                let path = self.path()?;
                let head = path.head().to_string();
                let mut args = Vec::new();
                let container = path.limb.is_none() && matches!(head.as_str(), "list" | "array" | "map" | "channel" | "ptr");
                let user_type = !is_primitive_head(&head) && !is_keyword(&head) || path.limb.is_some();
                let takes_type_args = container || (user_type && !conv);
                if self.is(&Tok::LBracket) && takes_type_args && !self.bracket_is_bound() {
                    self.bump();
                    while !self.is(&Tok::RBracket) {
                        args.push(self.type_()?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(&Tok::RBracket, "`]` closing the type parameters")?;
                } else if self.is(&Tok::LBracket) && is_primitive_head(&head) && !self.bracket_is_bound() && !conv {
                    if head == "bool" || head.starts_with("str") || head.starts_with("char") {
                        return self.fail(format!("`{}` takes no type parameters; a bracket after it is a bound — `[lo, hi]`", path.spelled()));
                    }
                    return self.fail(format!("`{}` takes no type parameters; a bracket after it is a range — `{}[0, 100]`", path.spelled(), path.spelled()));
                }
                TypeKind::Path { path, args }
            }
            Tok::Name(n) => return self.fail(format!("expected a type, found the name '{}'", n)),
            other => return self.fail(format!("expected a type, found {}", other.describe())),
        };
        let mut ty = Type { span: start.to(self.prev_span()), kind };
        // A bound: `T[lo, hi]`, `str.utf8[1, 40 graphemes]`, `list[T][0, 10]`.
        while self.is(&Tok::LBracket) && self.bracket_is_bound() {
            let bstart = self.span();
            self.bump();
            let lo = self.bound_value()?;
            self.expect(&Tok::Comma, "`,` between the bounds")?;
            let hi = self.bound_value()?;
            let unit = if let Tok::Word(w) = self.peek().clone() {
                if matches!(w.as_str(), "points" | "graphemes") && self.cores.as_ref().is_some_and(|c| !c.iter().any(|c| c == "UniTect")) {
                    return core_missing("counting in points or graphemes", "UniTect", self.span());
                }
                if !is_keyword(&w) && w != "x" {
                    Some(self.ident("a unit")?)
                } else {
                    None
                }
            } else {
                None
            };
            self.expect(&Tok::RBracket, "`]` closing the bound")?;
            let span = bstart.to(self.prev_span());
            ty = Type { span: start.to(self.prev_span()), kind: TypeKind::Bound { inner: Box::new(ty), lo: Box::new(lo), hi: Box::new(hi), unit, span } };
        }
        Ok(ty)
    }

    /// Is the bracket at the cursor `[expr, expr [unit]]` — a bound — rather
    /// than type arguments or a call's argument? Decided by a trial parse.
    fn bracket_is_bound(&mut self) -> bool {
        let save = self.pos;
        let mut ok = false;
        if self.eat(&Tok::LBracket) {
            if self.bound_end() && self.eat(&Tok::Comma) && self.bound_end() {
                if let Tok::Word(w) = self.peek().clone() {
                    if !is_keyword(&w) {
                        self.bump();
                    }
                }
                ok = self.is(&Tok::RBracket);
            }
        }
        self.pos = save;
        ok
    }

    /// One end of a bound as written: `min` or `max`, the base type's own
    /// limit (`int32[1, max]`), or a value.
    fn bound_value(&mut self) -> Result<Expr> {
        if let Tok::Word(w) = self.peek().clone() {
            if w == "min" || w == "max" {
                let span = self.bump().span;
                let path = Path { limb: None, segs: vec![Ident { name: w, span }], span };
                return Ok(Expr { kind: ExprKind::Path(path), span });
            }
        }
        self.expr(0)
    }

    /// One end of a bound: a number, possibly negative, or `min` / `max`.
    fn bound_end(&mut self) -> bool {
        if let Tok::Word(w) = self.peek() {
            if w == "min" || w == "max" {
                self.bump();
                return true;
            }
        }
        if self.is_op("-") {
            self.bump();
        }
        if let Tok::Num { .. } = self.peek() {
            self.bump();
            true
        } else {
            false
        }
    }

    // ------------------------------------------------------------ blocks and statements

    fn block(&mut self, what: &str) -> Result<Block> {
        let start = match self.peek() {
            Tok::LBrace => self.bump().span,
            Tok::Raw(_) => return self.fail("only `inline.ASM` holds raw text"),
            other => return self.fail(format!("expected `{{` opening {}'s body, found {}", what, other.describe())),
        };
        let mut stmts = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail_at(start, format!("{}'s `{{` is never closed", what));
            }
            stmts.push(self.stmt()?);
        }
        let end = self.bump().span;
        Ok(Block { stmts, span: start.to(end) })
    }

    fn stmt(&mut self) -> Result<Stmt> {
        let start = self.span();
        let kind = self.stmt_kind()?;
        Ok(Stmt { kind, span: start.to(self.prev_span()) })
    }

    /// A declaration, from `let` to its `;` (owner, 2026-10-02): `let 'x'
    /// := …`, `let 'x': T = …`, `mut`/`immut`/`!mut` and `!$` between `let`
    /// and the name, `$` before the `:=`/`=` to shadow.
    fn declaration(&mut self, shadowing: bool) -> Result<StmtKind> {
        if self.is(&Tok::Dot) {
            return self.fail("`let` is a word of its own: `let mut 'x' := …;`");
        }
        let mut sealed = false;
        let mut mutability = None;
        loop {
            // `no_shadow` (owner, 2026-10-07; `!$` before).
            if self.eat_word("no_shadow") {
                sealed = true;
            } else if self.eat_word("mut") {
                mutability = Some(Mutability::Mut);
            } else if self.eat_word("immut") {
                mutability = Some(Mutability::Immut);
            } else {
                break;
            }
        }
        if self.strict && mutability.is_none() {
            if let Tok::Name(n) = self.peek().clone() {
                return self.fail(format!("a declaration says whether it changes: `let mut '{}' := …;` or `let immut '{}' := …;` — a `#strict` file writes it out", n, n));
            }
        }
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                let span = self.bump().span;
                QName { name: n, span }
            }
            Tok::Word(w) if is_primitive_head(&w) || matches!(self.peek_at(1), Tok::Name(_)) => {
                return self.fail(format!("a type goes after the name: `let 'x': {} = …;`", w));
            }
            Tok::Word(w) => return self.fail(format!("a variable's name is quoted: `let '{}' := …;`", w)),
            other => return self.fail(format!("expected the variable's name, found {}", other.describe())),
        };
        if self.is_word("mut") || self.is_word("immut") {
            return self.fail(format!("`{}` goes before the name: `let {} '{}' := …;`", self.word_text(), self.word_text(), name.name));
        }
        let (ty, marker) = self.let_type_and_marker("declaration")?;
        let shadow = marker || shadowing;
        let value = self.expr_top()?;
        if self.is_op("=") {
            return self.fail("`=` gives a declaration its value once; equality is `equal_to`");
        }
        self.expect_semi("the declaration")?;
        Ok(StmtKind::Let { mutability, name, ty, shadow, value, sealed })
    }

    fn stmt_kind(&mut self) -> Result<StmtKind> {
        let w = match self.peek().clone() {
            Tok::Word(w) => w,
            Tok::LBrace => {
                let b = self.block("the block")?;
                return Ok(StmtKind::Block(b));
            }
            Tok::Semi => return self.fail("expected a statement, found `;` — there are no empty statements"),
            Tok::Strict => return self.fail("`#strict` goes once, at the top of the file, before any item"),
            // A declaration starts with `let` (owner, 2026-10-02).
            Tok::Name(n) => {
                return match self.peek_at(1) {
                    Tok::Colon | Tok::Walrus | Tok::Dollar => self.fail(format!("a declaration starts with `let`: `let '{}' := …;`", n)),
                    Tok::Word(m) if m == "mut" || m == "immut" => self.fail_at(self.span(), format!("a declaration starts with `let`, and `{}` goes before the name: `let {} '{}' := …;`", m, m, n)),
                    _ => self.expr_stmt(),
                };
            }
            Tok::Op(o) if o == "!" => return self.fail("a declaration starts with `let`: `let no_shadow 'x' := …;`, `let immut 'x' := …;`"),
            _ => return self.expr_stmt(),
        };
        match w.as_str() {
            "let" => {
                self.bump();
                self.declaration(false)
            }
            // `shadow 'x' := 2;` (owner, 2026-10-07: "yea"), a verb in
            // `let`'s place, as `change` is.
            "shadow" => {
                self.bump();
                if self.is_word("let") {
                    return self.fail("`shadow` takes `let`'s place: `shadow 'x' := …;`");
                }
                self.declaration(true)
            }
            "var" => self.fail("`var` is gone: a declaration starts with `let` — `let 'x' := …;`"),
            "mut" | "immut" => self.fail(format!("a declaration starts with `let`: `let {} 'x' := …;`", w)),
            // Changing says so (owner, 2026-09-25): `change 'x' = …;`.
            "change" => {
                self.bump();
                let target = self.expr_top()?;
                match &target.kind {
                    ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::Index { .. } => {}
                    ExprKind::ConstPath { .. } => return self.fail_at(target.span, "a const is never changed: it is a value known at compile time"),
                    // `pos` on every index (owner, 2026-10-07).
                    ExprKind::CallValue { args, .. } if args.len() == 1 => return self.fail_at(target.span, "an index says it is a position: `'xs'[pos 2]`"),
                    _ => return self.fail_at(target.span, "only a variable, a field or an element can be changed"),
                }
                if !self.eat_op("=") {
                    return self.fail(format!("expected `=` after what is changed, found {}", self.peek().describe()));
                }
                let value = self.expr_top()?;
                if self.is_op("=") {
                    return self.fail("`=` changes once; equality is `equal_to`");
                }
                self.expect_semi("the change")?;
                Ok(StmtKind::Assign { target, value })
            }
            w if is_primitive_head(w) && matches!(self.peek_at(1), Tok::Name(_)) => self.fail(format!("a type goes after the name: `let 'x': {} = …;`", w)),
            "check" | "nocheck" | "NO_FUCKING_CHECKS" => {
                let w = if w == "nocheck" { "no_check".to_string() } else { w };
                let tier_span = self.bump().span;
                let tier = match w.as_str() {
                    "check" => Tier::Check,
                    "no_check" => Tier::Nocheck,
                    _ => Tier::NoChecks,
                };
                if self.is(&Tok::Semi) {
                    return self.fail_at(tier_span, format!("`{}` goes in front of a statement, covering its check sites: `{} change 'xs'['i'] = 1;`, `{} {{ … }}`", w, w, w));
                }
                if self.is_word("check") || self.is_word("nocheck") || self.is_word("NO_FUCKING_CHECKS") {
                    return self.fail(format!("one tier per statement: `{}` and `{}` cannot both cover it", w, self.word_text()));
                }
                if self.is_word("fn") {
                    return self.fail(format!("`{}` goes in front of a statement, not a `fn`: a function's safety is the sum of its statements", w));
                }
                if self.is_word("struct") || self.is_word("const") || self.is_word("module") || self.is_word("enum") {
                    return self.fail(format!("`{}` goes in front of a statement, not a `{}`", w, self.word_text()));
                }
                let stmt = self.stmt()?;
                Ok(StmtKind::Tier { tier, stmt: Box::new(stmt) })
            }
            "if" => {
                self.bump();
                self.if_rest()
            }
            "else" => self.fail("`else` without an `if` before it"),
            "loop" => {
                self.bump();
                if self.is(&Tok::Dot) {
                    self.bump();
                    let what = self.ident("a word")?;
                    return match what.name.as_str() {
                        "while" => self.fail_at(what.span, "`loop.while` became `while`: `while 'x' < 3 { … }`"),
                        "for" | "temp" | "perm" | "seq" | "range" => self.fail_at(what.span, "`loop.temp.for` / `loop.range` became `for`: `for 'x' in lend 'xs' { … }`, `for 'i' in range[0, 9] { … }`"),
                        _ => self.fail_at(what.span, "`loop` takes no chain: `loop { … }` runs until `break`; `while` and `for` are bare words"),
                    };
                }
                let body = self.block("`loop`")?;
                Ok(StmtKind::Loop(body))
            }
            "while" => {
                self.bump();
                let cond = self.expr(0)?;
                let body = self.block("`while`")?;
                Ok(StmtKind::While { cond, body })
            }
            "for" => {
                self.bump();
                self.for_rest()
            }
            "return" => {
                self.bump();
                if self.eat(&Tok::Semi) {
                    return Ok(StmtKind::Return(None));
                }
                let e = self.expr_top()?;
                self.expect_semi("the return value")?;
                self.returns_value = true;
                Ok(StmtKind::Return(Some(e)))
            }
            // `must` (owner, 2026-09-29): `must …;`, `must.fail …;`,
            // `must.fail.<reason> …;`, `must.stop …;`. A function called
            // `must` is still `must[…]`.
            "must" if !matches!(self.peek_at(1), Tok::LBracket) => {
                self.bump();
                let kind = if self.eat(&Tok::Dot) {
                    let w = self.ident("`fail` or `stop` after `must.`")?;
                    match w.name.as_str() {
                        "fail" => {
                            if self.eat(&Tok::Dot) {
                                MustKind::Fail(Some(self.ident("the reason after `must.fail.`")?))
                            } else {
                                MustKind::Fail(None)
                            }
                        }
                        "stop" => MustKind::Stop,
                        other => return self.fail_at(w.span, format!("`must.{}` is not one of the family: `must …` (what must hold), `must.fail …` (it must fail), `must.stop …` (a check must fire)", other)),
                    }
                } else {
                    MustKind::Hold
                };
                let expr = self.expr_top()?;
                self.expect_semi("the `must`")?;
                Ok(StmtKind::Must { kind, expr })
            }
            "fail" => {
                self.bump();
                let name = self.ident("the failure's name")?;
                let payload = if self.is(&Tok::LBracket) {
                    self.bump();
                    let e = self.expr(0)?;
                    self.expect(&Tok::RBracket, "`]` closing the payload")?;
                    Some(e)
                } else {
                    None
                };
                self.expect_semi("the failure")?;
                Ok(StmtKind::Fail { name, payload })
            }
            "break" => {
                self.bump();
                self.expect_semi("`break`")?;
                Ok(StmtKind::Break)
            }
            "continue" => {
                self.bump();
                self.expect_semi("`continue`")?;
                Ok(StmtKind::Continue)
            }
            "on" => self.on_stmt(),
            "concur" => {
                self.bump();
                self.concur_rest()
            }
            "whichever" => {
                self.bump();
                self.whichever_rest()
            }
            "match" => {
                self.bump();
                self.match_rest()
            }
            // A case says its value with `is` (owner, 2026-10-07), `is
            // match …;` when the value is a match. `is[…]` is a call.
            "is" if !matches!(self.peek_at(1), Tok::LBracket | Tok::Dot) => {
                self.bump();
                let value = self.expr_top()?;
                self.expect_semi("the case's value")?;
                Ok(StmtKind::Gives { value, bare: false })
            }
            "inline" => {
                self.bump();
                self.inline_rest()
            }
            "asm" => self.fail("an inline block is `inline.ASM { … }` (the language's name in capitals, like `MAIN`), under `no_check`"),
            "otherwise" => self.fail("`otherwise` follows a call that can fail: `let 'x' := f[] otherwise 0;`"),
            "fn" | "struct" | "interface" | "op" => self.fail(format!("a `{}` sits at the top level of a file or a module, not inside a body", w)),
            "enum" => self.fail("an `enum` sits at the top level of a file or a module, not inside a body"),
            // A const in any body (owner, 2026-10-10: "I want it.", "`a`").
            "const" => {
                self.bump();
                Ok(StmtKind::Const(self.const_decl()?))
            }
            "module" => self.fail("a `module` sits at the top level of a file, not inside a body"),
            "pub" => self.fail("`pub` goes at the top level, on a `fn`, `struct`, `enum` or `const`"),
            "extern" => self.fail("an `extern` declaration sits at the top level, not inside a body"),
            "MAIN" => self.fail("`MAIN` cannot be nested or called: it is the program's one block"),
            "lang" if matches!(self.peek_at(1), Tok::Colon) => self.fail("`lang:` was removed: keywords are bare — `let 'x' := 1;`"),
            _ => self.expr_stmt(),
        }
    }

    fn if_rest(&mut self) -> Result<StmtKind> {
        let cond = self.expr(0)?;
        let then = self.block("`if`")?;
        let else_ = if self.is_word("else") {
            self.bump();
            if self.is(&Tok::Dot) {
                return self.fail("`else.if` became `else if`: two bare words");
            }
            if self.is_word("if") {
                let start = self.span();
                self.bump();
                let kind = self.if_rest()?;
                Some(ElseArm::If(Box::new(Stmt { kind, span: start.to(self.prev_span()) })))
            } else {
                Some(ElseArm::Block(self.block("`else`")?))
            }
        } else {
            None
        };
        Ok(StmtKind::If { cond, then, else_ })
    }

    fn for_rest(&mut self) -> Result<StmtKind> {
        let mut seq = false;
        let mut para = false;
        let mut perm = false;
        // `no_cheat` first: the strongest modifier (owner, 2026-10-01;
        // `!c`, then `!cheat`, before 2026-10-07).
        let nocheat = self.eat_word("no_cheat");
        // `seq` keeps a loop in order, `para` makes it parallel (owner,
        // 2026-09-30); never both. CCA runs every loop in order so far.
        if self.eat_word("seq") {
            seq = true;
        } else if self.eat_word("para") {
            para = true;
        }
        if self.is_word("seq") || self.is_word("para") {
            return self.fail("a `for` is `seq` or `para`, not both");
        }
        if self.is_word("no_cheat") {
            return self.fail("`no_cheat` comes first, the strongest modifier first: `for no_cheat seq …`, `for no_cheat para …`");
        }
        if self.eat_word("perm") {
            perm = true;
        } else if self.eat_word("temp") {
            perm = false;
        } else if self.strict && !self.is_word("seq") && !self.is_word("para") {
            return self.fail("a `for` says whether its variable outlives it: `for temp 'i' in …` or `for perm 'i' in …` — a `#strict` file writes it out");
        }
        if self.is_word("seq") || self.is_word("para") {
            return self.fail("expected a quoted loop variable: `seq` or `para` goes first, then `temp` / `perm`, then the name — `for seq perm 'i' in …`");
        }
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                let span = self.bump().span;
                QName { name: n, span }
            }
            Tok::Word(w) => return self.fail(format!("expected a quoted loop variable, found `{}`: `for '{}' in …`", w, w)),
            other => return self.fail(format!("expected a quoted loop variable, found {}", other.describe())),
        };
        let ty = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
        // `for 'i', 'x' in …`: the position first, then the item, as in
        // `'xs'['i']` (owner, 2026-09-29).
        let (pos, name) = if self.eat(&Tok::Comma) {
            let item = match self.peek().clone() {
                Tok::Name(n) => {
                    let span = self.bump().span;
                    QName { name: n, span }
                }
                other => return self.fail(format!("expected the item's quoted name after the position's: `for 'i', 'x' in …`, found {}", other.describe())),
            };
            (Some(name), item)
        } else {
            (None, name)
        };
        if self.is_word("temp") || self.is_word("perm") {
            return self.fail(format!("expected `in`; `{}` goes before the name — `for {} '{}' in …`", self.word_text(), self.word_text(), name.name));
        }
        if !self.eat_word("in") {
            return self.fail(format!("expected `in` after the loop variable, found {}", self.peek().describe()));
        }
        let iter = if self.is_word("range") {
            let rstart = self.span();
            self.bump();
            if self.is(&Tok::Dot) {
                return self.fail("`range` builds its bounds from values, and the counter's type goes on its name: `for 'i': int32 in range[0, 3]`");
            }
            self.expect(&Tok::LBracket, "`[` after `range`")?;
            if self.is(&Tok::Walrus) || self.is_op("=") {
                return self.fail("the bounds of a `range` are plain values: `range[0, 3]` — the `:=` / `=` markers are gone (they would say the same thing twice)");
            }
            // Every range labels each bound and writes its step (owner,
            // 2026-10-07: "`a`"): `range[from 2, to 8, by 1]`.
            let labelled = |this: &mut Self, word: &str| -> Result<Expr> {
                if this.range_label() != Some(word) {
                    return this.fail(format!("each bound says what it is: `range[from 2, to 8, by 1]` — this one is `{}`", word));
                }
                this.bump();
                this.expr(0)
            };
            let start = labelled(self, "from")?;
            self.expect(&Tok::Comma, "`,` between the range's start and stop")?;
            let stop = labelled(self, "to")?;
            if !self.eat(&Tok::Comma) {
                return self.fail("a range writes its step too, even when it is 1: `range[from 2, to 8, by 1]`");
            }
            let step = Some(labelled(self, "by")?);
            self.expect(&Tok::RBracket, "`]` closing the range")?;
            let _ = rstart;
            ForIter::Range { start, stop, step }
        } else {
            ForIter::Over(self.expr(0)?)
        };
        if let (Some(p), ForIter::Range { .. }) = (&pos, &iter) {
            return self.fail_at(p.span, format!("a range's counter already is the position: `for '{}' in range[…]`, one name", p.name));
        }
        let body = self.block("`for`")?;
        Ok(StmtKind::For { nocheat, seq, para, perm, name, pos, ty, iter, body })
    }

    /// `from`, `to` or `by` heading a range bound; `to.int32[…]` is a
    /// conversion, not a label.
    fn range_label(&self) -> Option<&'static str> {
        if self.is_word("from") {
            Some("from")
        } else if self.is_word("by") {
            Some("by")
        } else if self.is_word("to") && !matches!(self.peek_at(1), Tok::Dot) {
            Some("to")
        } else {
            None
        }
    }

    fn captures(&mut self) -> Result<Vec<Capture>> {
        // after `with`
        self.expect(&Tok::LBracket, "`[` after `with`")?;
        let mut caps = Vec::new();
        while !self.is(&Tok::RBracket) {
            let start = self.span();
            let (verb, amp) = match self.peek().clone() {
                Tok::Word(w) if w == "give" => {
                    self.bump();
                    (Verb::Give, false)
                }
                Tok::Word(w) if w == "copy" => {
                    self.bump();
                    (Verb::Copy, false)
                }
                Tok::Word(w) if w == "lend" => {
                    self.bump();
                    if self.eat(&Tok::Dot) {
                        let m = self.ident("`mut`")?;
                        if m.name != "mut" {
                            return self.fail_at(m.span, "`lend` or `lend.mut`");
                        }
                        (Verb::LendMut, false)
                    } else {
                        (Verb::Lend, false)
                    }
                }
                Tok::Op(o) if o == "&" => {
                    self.bump();
                    if self.eat_word("mut") {
                        (Verb::LendMut, true)
                    } else {
                        (Verb::Lend, true)
                    }
                }
                Tok::Name(n) => return self.fail(format!("nothing is captured silently: say how '{}' comes in — `lend '{}'`, `copy '{}'` or `give '{}'`", n, n, n, n)),
                other => return self.fail(format!("expected a capture (`lend 'x'`, `copy 'x'`, `give 'x'`), found {}", other.describe())),
            };
            let name = self.qname("the captured variable")?;
            caps.push(Capture { verb, amp, name, span: start.to(self.prev_span()) });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the captures")?;
        Ok(caps)
    }

    fn concur_rest(&mut self) -> Result<StmtKind> {
        let mut detached = false;
        if self.eat(&Tok::Dot) {
            let w = self.ident("`detached` after `concur.`")?;
            if w.name != "detached" {
                return self.fail_at(w.span, format!("`concur.{}` does not exist: `concur` (structured) or `concur.detached` (lives as long as the program)", w.name));
            }
            detached = true;
        }
        let captures = if self.eat_word("with") { self.captures()? } else { Vec::new() };
        let body = if self.is(&Tok::LBrace) {
            ConcurBody::Block(self.block("the task")?)
        } else {
            let e = self.expr_top()?;
            ConcurBody::Call(e)
        };
        let otherwise = if self.is_word("otherwise") {
            self.bump();
            Some(self.otherwise_rest()?)
        } else {
            None
        };
        if matches!(body, ConcurBody::Call(_)) {
            self.expect_semi("the task's call")?;
        } else {
            self.eat(&Tok::Semi);
        }
        Ok(StmtKind::Concur { detached, captures, body, otherwise })
    }

    fn whichever_rest(&mut self) -> Result<StmtKind> {
        let open = self.expect(&Tok::LBrace, "`{` after `whichever`")?;
        let mut arms = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail_at(open, "`whichever` is never closed");
            }
            let name = self.qname("the arm's value name")?;
            if !self.eat_word("from") {
                return self.fail(format!("expected `from` after '{}': each arm is `'value' from lend 'channel' {{ … }}`", name.name));
            }
            let channel = self.expr(0)?;
            let body = self.block("the arm")?;
            arms.push(WhicheverArm { name, channel, body });
        }
        self.bump();
        if arms.is_empty() {
            return self.fail_at(open.to(self.prev_span()), "`whichever` needs at least one arm: `'v' from lend 'c' { … }`");
        }
        let otherwise = if self.is_word("otherwise") {
            self.bump();
            Some(self.otherwise_rest()?)
        } else {
            None
        };
        self.eat(&Tok::Semi);
        Ok(StmtKind::Whichever { arms, otherwise })
    }

    fn match_rest(&mut self) -> Result<StmtKind> {
        let (value, arms) = self.match_parts()?;
        Ok(StmtKind::Match { value, arms })
    }

    fn match_parts(&mut self) -> Result<(Expr, Vec<MatchArm>)> {
        let value = self.expr(0)?;
        let open = self.expect(&Tok::LBrace, "`{` after the matched value")?;
        let mut arms = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail_at(open, "`match` is never closed");
            }
            let case = match self.peek().clone() {
                Tok::Word(_) => self.ident("a case name")?,
                Tok::Name(n) => return self.fail(format!("an arm starts with the case's bare name, then a quoted name for its data: `claimed '{}' {{ … }}`", n)),
                other => return self.fail(format!("expected a case name, found {}", other.describe())),
            };
            let bind = if let Tok::Name(_) = self.peek() { Some(self.qname("a name")?) } else { None };
            let body = self.block("the arm")?;
            arms.push(MatchArm { case, bind, body });
        }
        self.bump();
        if arms.is_empty() {
            return self.fail_at(open.to(self.prev_span()), "`match` needs an arm per case");
        }
        Ok((value, arms))
    }

    fn inline_rest(&mut self) -> Result<StmtKind> {
        self.expect(&Tok::Dot, "`.ASM` after `inline`")?;
        let lang = self.ident("the language after `inline.`")?;
        match lang.name.as_str() {
            // `inline.ASM` (owner, 2026-10-07: "Main should now be
            // `inline.ASM`"); `inline.Assembly` and `inline.S` are MTM's.
            "ASM" | "Assembly" | "S" => {}
            // IEG's and Metal's, named but not read yet.
            "LL" | "LLVM" | "LLVM_Intermediate_Representation" => return self.fail_at(lang.span, "`inline.LL`, LLVM IR in place, is IEG's, and Synapse does not read it yet"),
            "MSL" | "Metal_Shading_Language" => return self.fail_at(lang.span, "`inline.MSL`, the Metal Shading Language, is the Metal lobe's, and Synapse does not read it yet"),
            "asm" | "s" | "assembly" => return self.fail_at(lang.span, "an inline language's name is in capitals: `inline.ASM`"),
            "ll" | "llvm" => return self.fail_at(lang.span, "an inline language's name is in capitals: `inline.LL`"),
            "msl" => return self.fail_at(lang.span, "an inline language's name is in capitals: `inline.MSL`"),
            "C" | "c" => return self.fail_at(lang.span, "the inline languages are `inline.ASM`, `inline.LL` and `inline.MSL`; `inline.C` is not decided"),
            other => return self.fail_at(lang.span, format!("the inline languages are `inline.ASM`, `inline.LL` and `inline.MSL`; `inline.{}` is not one", other)),
        }
        let text = match self.peek().clone() {
            Tok::Raw(t) => {
                self.bump();
                t
            }
            Tok::LBracket => return self.fail("`inline.ASM` holds raw assembly in braces, not a list of strings: `inline.ASM { mov x16, #4 }`"),
            other => return self.fail(format!("expected `{{` holding the assembly, found {}", other.describe())),
        };
        let mut bindings = Vec::new();
        if self.eat_word("with") {
            self.expect(&Tok::LBracket, "`[` after `with`")?;
            while !self.is(&Tok::RBracket) {
                match self.peek().clone() {
                    Tok::Word(w) if w == "clobber" => {
                        self.bump();
                        let reg = self.ident("a register after `clobber`")?;
                        bindings.push(AsmBinding::Clobber(reg));
                    }
                    Tok::Word(w) if w == "in" || w == "out" || w == "inout" => {
                        return self.fail(format!("there is no `{}` word: the arrow is the direction — `'v' -> x0` copies in, `'v' <- x0` stores out, `'v' <-> x0` both", w));
                    }
                    // A lend satisfies a pointer, as at an extern call
                    // (owner, 2026-09-30: a): `lend 'text' -> x1`.
                    Tok::Word(w) if w == "lend" => {
                        self.bump();
                        let mutable = if self.eat(&Tok::Dot) {
                            let m = self.ident("`mut`")?;
                            if m.name != "mut" {
                                return self.fail_at(m.span, "`lend` or `lend.mut`");
                            }
                            true
                        } else {
                            false
                        };
                        let name = self.qname("the variable lent")?;
                        match self.peek().clone() {
                            Tok::Op(o) if o == "->" => {}
                            Tok::Op(o) if o == "<-" || o == "<->" => {
                                return self.fail(format!("a lend goes into a register only: `{} '{}' -> x1` puts the address of '{}' there", if mutable { "lend.mut" } else { "lend" }, name.name, name.name));
                            }
                            other => return self.fail(format!("expected `->` after `lend '{}'`: its address goes into a register, found {}", name.name, other.describe())),
                        }
                        self.bump();
                        let reg = self.ident("a register")?;
                        bindings.push(AsmBinding::Lend { name, mutable, reg });
                    }
                    Tok::Name(_) => {
                        let name = self.qname("a variable")?;
                        let dir = match self.peek().clone() {
                            Tok::Op(o) if o == "->" => AsmDir::In,
                            Tok::Op(o) if o == "<-" => AsmDir::Out,
                            Tok::Op(o) if o == "<->" => AsmDir::InOut,
                            other => return self.fail(format!("expected an arrow after '{}' (`->` in, `<-` out, `<->` both), found {}", name.name, other.describe())),
                        };
                        self.bump();
                        let reg = self.ident("a register")?;
                        bindings.push(AsmBinding::Reg { name, dir, reg });
                    }
                    other => return self.fail(format!("expected a binding (`'v' -> x0`, `'n' <- x0`, `lend 'text' -> x1`, `clobber x16`), found {}", other.describe())),
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RBracket, "`]` closing the bindings")?;
        }
        self.expect_semi("the inline block")?;
        Ok(StmtKind::InlineAsm { text, bindings })
    }

    /// An expression statement or an assignment.
    fn expr_stmt(&mut self) -> Result<StmtKind> {
        let e = self.expr_top()?;
        // A value alone before the `}` is its block's value — a `match`
        // case's or an `otherwise` arm's (owner, 2026-09-29: a).
        if self.is(&Tok::RBrace) {
            return self.fail_at(e.span, "a case says its value with `is`: `a { is …; }`");
        }
        if self.is(&Tok::Colon) {
            if let ExprKind::Name(n) = &e.kind {
                return self.fail(format!("a declaration is the name, its type, then `=`: `let '{}': int32 = …;`", n.name));
            }
        }
        if self.is_op("=") {
            let spelled = match &e.kind {
                ExprKind::Name(n) => format!("'{}'", n.name),
                _ => "'x'".to_string(),
            };
            return self.fail_at(e.span, format!("changing a variable says so: `change {} = …;`", spelled));
        }
        if self.is(&Tok::Walrus) {
            return self.fail("a declaration is a name alone: `let 'x' := …;`");
        }
        match &e.kind {
            ExprKind::Call { .. } | ExprKind::CallValue { .. } | ExprKind::Conv { .. } => {}
            ExprKind::Path(p) => return self.fail_at(e.span, format!("`{}` alone is not a statement: a call needs `[…]` — `{}[…];`", p.spelled(), p.spelled())),
            // One value in brackets: a list read, or a call of a function
            // held in a variable — the checker knows which.
            ExprKind::Index { .. } => {}
            _ => {
                if self.is(&Tok::Semi) {
                    return self.fail_at(e.span, "a value alone is not a statement: assign it, or call something with it");
                }
                return self.fail(format!("expected `;` after the statement, found {}", self.peek().describe()));
            }
        }
        if !self.is(&Tok::Semi) {
            return self.fail(format!("expected `;` after the call, found {}", self.peek().describe()));
        }
        self.bump();
        Ok(StmtKind::Expr(e))
    }

    // ------------------------------------------------------------ expressions

    /// A whole value: an expression, optionally followed by `otherwise` when
    /// it is a call that can fail.
    fn expr_top(&mut self) -> Result<Expr> {
        let mut e = self.expr(0)?;
        if self.is_word("otherwise") {
            let ow_span = self.span();
            self.bump();
            let ow = self.otherwise_rest()?;
            // `'f'[2] otherwise …` is a call: a list read never fails.
            if let ExprKind::Index { .. } = e.kind {
                let ExprKind::Index { base, index } = std::mem::replace(&mut e.kind, ExprKind::Bool(false)) else { unreachable!() };
                e.kind = ExprKind::CallValue { callee: base, args: vec![Arg::Positional(*index)], otherwise: None };
            }
            match &mut e.kind {
                ExprKind::Call { otherwise, .. } | ExprKind::CallValue { otherwise, .. } | ExprKind::Conv { otherwise, .. } => {
                    if otherwise.is_some() {
                        return self.fail_at(ow_span, "a call takes one `otherwise`");
                    }
                    *otherwise = Some(Box::new(ow));
                    e.span = e.span.to(self.prev_span());
                }
                _ => {
                    return self.fail_at(e.span, "a call that can fail must be the whole value: put it in its own declaration, then use the result — `otherwise` cannot sit inside a larger expression");
                }
            }
        }
        Ok(e)
    }

    fn otherwise_rest(&mut self) -> Result<Otherwise> {
        match self.peek().clone() {
            Tok::Semi | Tok::RBrace | Tok::Eof => self.fail("`otherwise` needs what to do: a value, `return …`, `fail`, `break`, `continue`, or `{ reason { … } … }` arms"),
            Tok::Word(w) if w == "fail" => {
                let span = self.span();
                self.bump();
                if let Tok::Word(_) = self.peek() {
                    let name = self.ident("the failure's name")?;
                    let payload = if self.is(&Tok::LBracket) {
                        self.bump();
                        let e = self.expr(0)?;
                        self.expect(&Tok::RBracket, "`]`")?;
                        Some(e)
                    } else {
                        None
                    };
                    let s = Stmt { kind: StmtKind::Fail { name, payload }, span: span.to(self.prev_span()) };
                    Ok(Otherwise::Leave(Box::new(s)))
                } else {
                    Ok(Otherwise::PassUp(span))
                }
            }
            Tok::Word(w) if w == "return" => {
                let span = self.span();
                self.bump();
                let value = if self.is(&Tok::Semi) || self.is(&Tok::RBrace) { None } else { Some(self.expr_top()?) };
                let s = Stmt { kind: StmtKind::Return(value), span: span.to(self.prev_span()) };
                Ok(Otherwise::Leave(Box::new(s)))
            }
            Tok::Word(w) if w == "break" => {
                let span = self.bump().span;
                Ok(Otherwise::Leave(Box::new(Stmt { kind: StmtKind::Break, span })))
            }
            Tok::Word(w) if w == "continue" => {
                let span = self.bump().span;
                Ok(Otherwise::Leave(Box::new(Stmt { kind: StmtKind::Continue, span })))
            }
            Tok::LBrace => {
                let open = self.bump().span;
                let mut arms = Vec::new();
                while !self.is(&Tok::RBrace) {
                    if self.at_eof() {
                        return self.fail_at(open, "the `otherwise` arms are never closed");
                    }
                    let name = match self.peek().clone() {
                        Tok::Word(_) => self.ident("a failure name")?,
                        Tok::Name(n) => return self.fail(format!("an arm starts with the reason's bare name, then a quoted name for its payload: `bad_item '{}' {{ … }}`", n)),
                        other => return self.fail(format!("expected a failure name, found {}", other.describe())),
                    };
                    let bind = if let Tok::Name(_) = self.peek() { Some(self.qname("a name")?) } else { None };
                    let body = self.block("the arm")?;
                    arms.push(FailArm { name, bind, body });
                }
                let close = self.bump().span;
                if arms.is_empty() {
                    return self.fail_at(open.to(close), "`otherwise { }` has no arms: one arm per reason, `reason { … }`");
                }
                Ok(Otherwise::Arms(arms, open.to(close)))
            }
            _ => Ok(Otherwise::Value(self.expr(0)?)),
        }
    }

    /// Binding powers. `None` for a token that is not a binary operator here.
    /// The operator after `op`: any a value can be, whatever follows it.
    /// A Limb's `precedence` block (owner, 2026-10-03): `precedence.low_first`
    /// counts level 1 as the loosest, `precedence.high_first` as the
    /// tightest; each `level.when_chained.left_first N { … }` (or
    /// `.right_first`, `.refused`) lists operators by name. A language
    /// operator pins its level where the language has it, and must chain as
    /// it does; a level with none sits evenly between its neighbours — from
    /// 0 below, to the next language level above. Returns each named
    /// operator's level and chain.
    fn precedence_block(&mut self) -> Result<Vec<(String, (u16, Chain))>> {
        let at = self.bump().span;
        if !self.eat(&Tok::Dot) {
            return self.fail_at(at, "say which way the levels count: `precedence.low_first` (level 1 binds loosest, the good practice) or `precedence.high_first`");
        }
        let way = self.ident("`low_first` or `high_first`")?;
        let low_first = match way.name.as_str() {
            "low_first" => true,
            "high_first" => false,
            other => return self.fail_at(way.span, format!("`precedence.{}` is not a way to count: `precedence.low_first` or `precedence.high_first`", other)),
        };
        self.expect(&Tok::LBrace, "`{` opening the levels")?;
        // (number, chain, names, the number's span)
        let mut levels: Vec<(i128, Chain, Vec<(String, Span)>, Span)> = Vec::new();
        while !self.eat(&Tok::RBrace) {
            let lat = self.span();
            if !self.eat_word("level") {
                return self.fail(format!("a `precedence` block holds levels, `level.when_chained.left_first 1 {{ plus }}`; found {}", self.peek().describe()));
            }
            let how = "say how a chain of this level reads: `level.when_chained.left_first`, `level.when_chained.right_first` or `level.when_chained.refused`";
            if !self.eat(&Tok::Dot) {
                return self.fail_at(lat, how);
            }
            let w = self.ident("`when_chained`")?;
            if w.name != "when_chained" || !self.eat(&Tok::Dot) {
                return self.fail_at(w.span, how);
            }
            let c = self.ident("`left_first`, `right_first` or `refused`")?;
            let chain = match c.name.as_str() {
                "left_first" => Chain::Left,
                "right_first" => Chain::Right,
                "refused" => Chain::Refused,
                _ => return self.fail_at(c.span, how),
            };
            let (n, nspan) = match self.peek().clone() {
                Tok::Num { text, bin: false } => (text.replace('_', "").parse::<i128>().unwrap_or(0), self.bump().span),
                other => return self.fail(format!("a level's number follows how it chains: `level.when_chained.left_first 1 {{ … }}`, found {}", other.describe())),
            };
            if let Some(first) = levels.iter().find(|l| l.0 == n) {
                let _ = first;
                return self.fail_at(nspan, format!("two levels numbered {}: each level has its own number", n));
            }
            self.expect(&Tok::LBrace, "`{` opening the level's operators")?;
            let mut names = Vec::new();
            while !self.eat(&Tok::RBrace) {
                let ospan = self.span();
                let name = match self.peek().clone() {
                    Tok::Op(o) => {
                        self.bump();
                        o
                    }
                    Tok::Word(w) => {
                        self.bump();
                        let mut name = w;
                        while self.is(&Tok::Dot) && matches!(self.peek_at(1), Tok::Word(_)) {
                            self.bump();
                            let Tok::Word(more) = self.bump().tok else { unreachable!() };
                            name = format!("{}.{}", name, more);
                        }
                        name
                    }
                    other => return self.fail(format!("a level lists operators by name, `{{ plus  dot }}`; found {}", other.describe())),
                };
                if let Some(word) = retired_spelling(&name) {
                    return self.fail_at(ospan, format!("`{}` is `{}` now (owner, 2026-10-03)", name, word));
                }
                if matches!(name.as_str(), "xor" | "shl" | "shr" | "and.bit" | "or.bit" | "not.bit") {
                    return self.fail_at(ospan, format!("`{}` has no precedence in the language: a bitwise operator is parenthesised beside another, so no level holds it", name));
                }
                names.push((name, ospan));
            }
            levels.push((n, chain, names, nspan));
        }
        // Loosest first.
        levels.sort_by_key(|l| l.0);
        if !low_first {
            levels.reverse();
        }
        let mut pinned: Vec<Option<u16>> = Vec::new();
        for (_, chain, names, _) in &levels {
            let mut pin: Option<(u16, &str)> = None;
            for (name, span) in names {
                let Some((lv, lchain)) = language_level(name) else { continue };
                if let Some(c) = lchain {
                    if c != *chain {
                        let says = match c {
                            Chain::Left => "left_first",
                            Chain::Right => "right_first",
                            Chain::Refused => "refused",
                        };
                        return self.fail_at(*span, format!("`{}` chains {} in the language, so its level must say `level.when_chained.{}`", name, says.replace('_', " "), says));
                    }
                }
                match pin {
                    Some((p, other)) if p != lv => return self.fail_at(*span, format!("`{}` and `{}` sit at different levels in the language: one level cannot hold both", other, name)),
                    _ => pin = Some((lv, name)),
                }
            }
            pinned.push(pin.map(|p| p.0));
        }
        // A level with no language operator: evenly between its neighbours.
        let mut values: Vec<u16> = vec![0; levels.len()];
        let mut i = 0;
        while i < levels.len() {
            if let Some(v) = pinned[i] {
                values[i] = v;
                i += 1;
                continue;
            }
            let start = i;
            while i < levels.len() && pinned[i].is_none() {
                i += 1;
            }
            let lo = if start == 0 { 0 } else { values[start - 1] as u32 };
            let hi = if i < levels.len() { pinned[i].unwrap() as u32 } else { lo + 1000 };
            let n = (i - start) as u32;
            for k in 0..n {
                values[start + k as usize] = (lo + (hi.saturating_sub(lo)) * (k + 1) / (n + 1)) as u16;
            }
        }
        let mut out = Vec::new();
        for ((_, chain, names, _), v) in levels.into_iter().zip(values) {
            for (name, _) in names {
                out.push((name, (v, chain)));
            }
        }
        Ok(out)
    }

    /// The rounding after `div` or `mod`, the word already taken: `.floor`,
    /// `.ceil`, `.trunc` or `.near`, or none — which the checker refuses,
    /// naming the one meant.
    /// `.wrap` after a division's rounding (owner, 2026-10-03): the
    /// hardware's wrapped quotient. A remainder never overflows, so `mod`
    /// takes none.
    /// `div.X.ieee` (owner, 2026-10-07): IEEE's `-0` kept, `div.near.ieee`
    /// too ("yea").
    fn ieee_narrow(&mut self, w: &str, r: Round) -> Result<bool> {
        if !(self.is(&Tok::Dot) && self.is_word_at(1, "ieee")) || r == Round::Bare {
            return Ok(false);
        }
        let at = self.span();
        self.bump();
        self.bump();
        let at = at.to(self.prev_span());
        // `mod.near` leaves nothing over on a bin (owner, 2026-10-07).
        if w == "mod" && r == Round::Near {
            return self.fail_at(at, "`.ieee` goes on an operator that can make a -0, `plus`, `minus`, `times`, `pow`, a `div.X`, `mod.floor`, `mod.ceil` or `mod.trunc`; `mod.near` has no `.ieee` form");
        }
        Ok(true)
    }

    fn wrap_narrow(&mut self, w: &str, r: Round) -> Result<bool> {
        if !(self.is(&Tok::Dot) && self.is_word_at(1, "wrap")) || r == Round::Bare {
            return Ok(false);
        }
        let at = self.span();
        self.bump();
        self.bump();
        if w == "mod" {
            return self.fail_at(at.to(self.prev_span()), format!("`mod` never overflows: `mod.{}.wrap` is `mod.{}`", r.word(), r.word()));
        }
        Ok(true)
    }

    fn rounding(&mut self, w: &str) -> Result<Round> {
        if !self.is(&Tok::Dot) {
            return Ok(Round::Bare);
        }
        self.bump();
        let what = self.ident("`floor`, `ceil`, `trunc` or `near`")?;
        match Round::from_word(&what.name) {
            Some(r) => Ok(r),
            None => self.fail_at(what.span, format!("no `{w}.{}`: a division rounds as `{w}.floor`, `{w}.ceil`, `{w}.trunc` or `{w}.near`", what.name)),
        }
    }

    fn op_as_value(&mut self) -> Result<(BinOp, String)> {
        let at = self.span();
        let op = match self.peek().clone() {
            Tok::Op(o) => {
                self.bump();
                if let Some(word) = retired_spelling(&o) {
                    return self.fail_at(at, format!("`{}` is `{}` now (owner, 2026-10-03): `op {}`", o, word, word));
                }
                match o.as_str() {
                    "/" => return self.fail_at(at, "division is `div.floor`, `div.ceil`, `div.trunc` or `div.near`: `op div.floor`"),
                    "++" => BinOp::Join,
                    "==" => BinOp::Eq,
                    "!==" => BinOp::Ne,
                    "<" => BinOp::Lt,
                    ">" => BinOp::Gt,
                    "</==" => BinOp::Le,
                    ">/==" => BinOp::Ge,
                    "<==" => return self.fail_at(at, "less-or-equal is `at_most`: `op at_most`"),
                    ">==" => return self.fail_at(at, "greater-or-equal is `at_least`: `op at_least`"),
                    "=" => return self.fail_at(at, "`=` is not an operator: equality is `op equal_to`"),
                    other => BinOp::Custom(other.to_string()),
                }
            }
            Tok::Word(w) => {
                let wraps = self.is_op_at(1, "~") && self.adjacent(0, 1);
                self.bump();
                match w.as_str() {
                    "x" | "xx" => {
                        let old = if wraps { format!("{}~", w) } else { w.clone() };
                        let word = retired_spelling(&old).unwrap_or("times");
                        return self.fail_at(at, format!("`{}` is `{}` now (owner, 2026-10-03): `op {}`", old, word, word));
                    }
                    "plus" | "minus" | "times" | "pow" => {
                        let wrap = self.is(&Tok::Dot) && self.is_word_at(1, "wrap");
                        if wrap {
                            self.bump();
                            self.bump();
                        }
                        match (w.as_str(), wrap) {
                            ("plus", false) => BinOp::Add,
                            ("plus", true) => BinOp::WrapAdd,
                            ("minus", false) => BinOp::Sub,
                            ("minus", true) => BinOp::WrapSub,
                            ("times", false) => BinOp::Mul,
                            ("times", true) => BinOp::WrapMul,
                            (_, false) => BinOp::Pow,
                            (_, true) => BinOp::WrapPow,
                        }
                    }
                    "mod" | "div" => {
                        let r = self.rounding(&w)?;
                        let wrap = self.wrap_narrow(&w, r)?;
                        match (w.as_str(), wrap) {
                            ("div", true) => BinOp::DivWrap(r),
                            ("div", false) => BinOp::Div(r),
                            _ => BinOp::Mod(r),
                        }
                    }
                    "and" | "or" if self.eat(&Tok::Dot) => {
                        let what = self.ident("`then`, `else` or `bit`")?;
                        match (w.as_str(), what.name.as_str()) {
                            ("and", "then") => BinOp::AndThen,
                            ("or", "else") => BinOp::OrElse,
                            ("and", "bit") => BinOp::AndBit,
                            ("or", "bit") => BinOp::OrBit,
                            _ => return self.fail_at(what.span, format!("`{}.{}` is not an operator", w, what.name)),
                        }
                    }
                    "and" => BinOp::And,
                    "or" => BinOp::Or,
                    "xor" => BinOp::Xor,
                    "shl" => BinOp::Shl,
                    "shr" => BinOp::Shr,
                    other if is_keyword(other) => return self.fail_at(at, format!("`{}` is a keyword, not an operator: `op` takes one, as in `op plus`", other)),
                    other => {
                        // A Limb's narrowed operator: `op mean.low`.
                        let mut name = other.to_string();
                        while self.is(&Tok::Dot) && matches!(self.peek_at(1), Tok::Word(_)) {
                            self.bump();
                            let Tok::Word(more) = self.bump().tok else { unreachable!() };
                            name = format!("{}.{}", name, more);
                        }
                        BinOp::Custom(name)
                    }
                }
            }
            other => return self.fail(format!("`op` as a value takes an operator: `op plus`, `op times`, `op <`, found {}", other.describe())),
        };
        let spelled = op.spelled().to_string();
        Ok((op, spelled))
    }

    /// The binary operator at the cursor, if one is: what it is, its
    /// binding powers, how many tokens spell it, and the word it is now when
    /// written in a retired spelling (owner, 2026-10-03).
    fn binop(&self) -> Option<OpTok> {
        let span = self.span();
        let lang = |op: BinOp, level: u16, chain: Chain| {
            let (l, r) = powers(level, chain);
            OpTok { op, l, r, span, len: 1, old: None }
        };
        let nofix = |op: BinOp| OpTok { op, l: 0, r: 1, span, len: 1, old: None };
        let leveled = |name: &str, op: BinOp, len: usize| -> OpTok {
            match self.prec.get(name) {
                Some((level, chain)) => {
                    let (l, r) = powers(*level, *chain);
                    OpTok { op, l, r, span, len, old: None }
                }
                None => OpTok { op, l: 0, r: 1, span, len, old: None },
            }
        };
        // A dotted name after `n` tokens: `mean.low`, `pow.wrap`.
        let dotted = |mut n: usize, first: &str| -> (String, usize) {
            let mut name = first.to_string();
            while matches!(self.peek_at(n), Tok::Dot) {
                let Tok::Word(more) = self.peek_at(n + 1) else { break };
                name = format!("{}.{}", name, more);
                n += 2;
            }
            (name, n)
        };
        let t = match self.peek() {
            Tok::Op(o) => match o.as_str() {
                "+" => OpTok { old: Some("plus"), ..lang(BinOp::Add, ADD_LEVEL, Chain::Left) },
                "-" => OpTok { old: Some("minus"), ..lang(BinOp::Sub, ADD_LEVEL, Chain::Left) },
                "+~" => OpTok { old: Some("plus.wrap"), ..lang(BinOp::WrapAdd, ADD_LEVEL, Chain::Left) },
                "-~" => OpTok { old: Some("minus.wrap"), ..lang(BinOp::WrapSub, ADD_LEVEL, Chain::Left) },
                "++" => lang(BinOp::Join, JOIN_LEVEL, Chain::Left),
                "==" => lang(BinOp::Eq, CMP_LEVEL, Chain::Refused),
                "!==" => lang(BinOp::Ne, CMP_LEVEL, Chain::Refused),
                "<" => lang(BinOp::Lt, CMP_LEVEL, Chain::Refused),
                ">" => lang(BinOp::Gt, CMP_LEVEL, Chain::Refused),
                "</==" => lang(BinOp::Le, CMP_LEVEL, Chain::Refused),
                ">/==" => lang(BinOp::Ge, CMP_LEVEL, Chain::Refused),
                "=" => return None,
                other => leveled(other, BinOp::Custom(other.to_string()), 1),
            },
            Tok::Word(w) => {
                let wraps = self.is_op_at(1, "~") && self.adjacent(0, 1);
                match w.as_str() {
                    "x" if wraps => OpTok { len: 2, old: Some("times.wrap"), ..lang(BinOp::WrapMul, MUL_LEVEL, Chain::Left) },
                    "xx" if wraps => OpTok { len: 2, old: Some("pow.wrap"), ..lang(BinOp::WrapPow, POW_LEVEL, Chain::Right) },
                    "x" => OpTok { old: Some("times"), ..lang(BinOp::Mul, MUL_LEVEL, Chain::Left) },
                    "xx" => OpTok { old: Some("pow"), ..lang(BinOp::Pow, POW_LEVEL, Chain::Right) },
                    // The operators are words (owner, 2026-10-02/03), and
                    // `.wrap` narrows them to the wrapping forms.
                    // `.ieee` keeps IEEE's -0 (owner, 2026-10-07: "yea").
                    "plus" | "minus" | "times" | "pow" if matches!(self.peek_at(1), Tok::Dot) && self.is_word_at(2, "ieee") && !matches!(self.peek_at(3), Tok::Dot) => {
                        let (op, level, chain) = match w.as_str() {
                            "plus" => (BinOp::AddIeee, ADD_LEVEL, Chain::Left),
                            "minus" => (BinOp::SubIeee, ADD_LEVEL, Chain::Left),
                            "times" => (BinOp::MulIeee, MUL_LEVEL, Chain::Left),
                            _ => (BinOp::PowIeee, POW_LEVEL, Chain::Right),
                        };
                        OpTok { len: 3, ..lang(op, level, chain) }
                    }
                    "plus" | "minus" | "times" | "pow" if matches!(self.peek_at(1), Tok::Dot) && !self.is_word_at(2, "wrap") => {
                        let (name, end) = dotted(1, w);
                        OpTok { len: end, ..nofix(BinOp::Custom(name)) }
                    }
                    "plus" | "minus" | "times" | "pow" => {
                        let wrap = matches!(self.peek_at(1), Tok::Dot) && self.is_word_at(2, "wrap");
                        let (op, level, chain) = match (w.as_str(), wrap) {
                            ("plus", false) => (BinOp::Add, ADD_LEVEL, Chain::Left),
                            ("plus", true) => (BinOp::WrapAdd, ADD_LEVEL, Chain::Left),
                            ("minus", false) => (BinOp::Sub, ADD_LEVEL, Chain::Left),
                            ("minus", true) => (BinOp::WrapSub, ADD_LEVEL, Chain::Left),
                            ("times", false) => (BinOp::Mul, MUL_LEVEL, Chain::Left),
                            ("times", true) => (BinOp::WrapMul, MUL_LEVEL, Chain::Left),
                            (_, false) => (BinOp::Pow, POW_LEVEL, Chain::Right),
                            (_, true) => (BinOp::WrapPow, POW_LEVEL, Chain::Right),
                        };
                        OpTok { len: if wrap { 3 } else { 1 }, ..lang(op, level, chain) }
                    }
                    // The rounding word is read when the operator is taken.
                    "mod" => lang(BinOp::Mod(Round::Bare), MUL_LEVEL, Chain::Left),
                    "div" => lang(BinOp::Div(Round::Bare), MUL_LEVEL, Chain::Left),
                    "and" if matches!(self.peek_at(1), Tok::Dot) && self.is_word_at(2, "then") => lang(BinOp::AndThen, AND_LEVEL, Chain::Left),
                    "or" if matches!(self.peek_at(1), Tok::Dot) && self.is_word_at(2, "else") => lang(BinOp::OrElse, OR_LEVEL, Chain::Left),
                    "and" if matches!(self.peek_at(1), Tok::Dot) => nofix(BinOp::AndBit),
                    "or" if matches!(self.peek_at(1), Tok::Dot) => nofix(BinOp::OrBit),
                    "and" => lang(BinOp::And, AND_LEVEL, Chain::Left),
                    "or" => lang(BinOp::Or, OR_LEVEL, Chain::Left),
                    "xor" => nofix(BinOp::Xor),
                    "shl" => nofix(BinOp::Shl),
                    "shr" => nofix(BinOp::Shr),
                    // A path on an operator picks that Limb's (owner,
                    // 2026-10-03): `'x' areas::mean.low 'y'`; std's is
                    // `pow`, `'a' std::pow 2`.
                    limb if matches!(self.peek_at(1), Tok::ColonColon) && matches!(self.peek_at(2), Tok::Word(_)) => {
                        let Tok::Word(first) = self.peek_at(2) else { unreachable!() };
                        let (name, end) = dotted(3, first);
                        if limb == "std" && matches!(name.as_str(), "pow" | "pow.wrap" | "pow.ieee") {
                            let op = match name.as_str() {
                                "pow" => BinOp::Pow,
                                "pow.wrap" => BinOp::WrapPow,
                                _ => BinOp::PowIeee,
                            };
                            OpTok { len: end, ..lang(op, POW_LEVEL, Chain::Right) }
                        } else {
                            leveled(&name, BinOp::Custom(format!("{}::{}", limb, name)), end)
                        }
                    }
                    w if is_keyword(w) || DURATION_UNITS.contains(&w) => return None,
                    // a word operator needs a right operand; a unit word before `]` is not one
                    _ if matches!(self.peek_at(1), Tok::RBracket | Tok::RParen | Tok::Comma | Tok::Semi | Tok::LBrace | Tok::RBrace | Tok::Eof) => return None,
                    other => {
                        let (name, end) = dotted(1, other);
                        leveled(&name, BinOp::Custom(name.clone()), end)
                    }
                }
            }
            _ => return None,
        };
        Some(t)
    }

    fn expr(&mut self, min_bp: u16) -> Result<Expr> {
        let mut lhs = self.prefix()?;
        loop {
            // std's word operators with their path, `'a' std::plus 'b'`
            // (owner, 2026-10-06: std is `bare.optional` for all), read as
            // the operator itself; `std::pow` has its own arm.
            let skip = if self.is_word("std") && matches!(self.peek_at(1), Tok::ColonColon) && matches!(self.peek_at(2), Tok::Word(w) if matches!(w.as_str(), "plus" | "minus" | "times" | "div" | "mod" | "and" | "or" | "xor" | "shl" | "shr")) { 2 } else { 0 };
            self.pos += skip;
            let Some(t) = self.binop() else {
                self.pos -= skip;
                break;
            };
            let (mut op, l_bp, r_bp, op_span) = (t.op.clone(), t.l, t.r, t.span);
            if l_bp < min_bp {
                self.pos -= skip;
                break;
            }
            // consume the operator token(s)
            self.bump();
            match op {
                BinOp::AndThen | BinOp::OrElse => {
                    self.bump(); // .
                    self.bump(); // then / else
                }
                BinOp::Div(_) | BinOp::Mod(_) => {
                    let w = if op.is_div() { "div" } else { "mod" };
                    let r = self.rounding(w)?;
                    let ieee = self.ieee_narrow(w, r)?;
                    let wrap = !ieee && self.wrap_narrow(w, r)?;
                    op = match (op.is_div(), wrap) {
                        (true, _) if ieee => BinOp::DivIeee(r),
                        (false, _) if ieee => BinOp::ModIeee(r),
                        (true, true) => BinOp::DivWrap(r),
                        (true, false) => BinOp::Div(r),
                        (false, _) => BinOp::Mod(r),
                    };
                }
                BinOp::AndBit | BinOp::OrBit => {
                    self.bump(); // .
                    let b = self.ident("`bit`")?;
                    if b.name != "bit" {
                        return self.fail_at(b.span, format!("`{}.{}` is not an operator: the bitwise forms are `and.bit` and `or.bit`", op.spelled().trim_end_matches(".bit"), b.name));
                    }
                    // `.wrap`: the operands as their bits (owner, 2026-10-07).
                    if self.is(&Tok::Dot) && self.is_word_at(1, "wrap") {
                        self.bump();
                        self.bump();
                        op = if op == BinOp::AndBit { BinOp::AndBitWrap } else { BinOp::OrBitWrap };
                    }
                }
                BinOp::Shl | BinOp::Shr if self.is(&Tok::Dot) && self.is_word_at(1, "masked") => {
                    self.bump();
                    self.bump();
                    op = if op == BinOp::Shl { BinOp::ShlMasked } else { BinOp::ShrMasked };
                }
                BinOp::Xor if self.is(&Tok::Dot) && self.is_word_at(1, "wrap") => {
                    self.bump();
                    self.bump();
                    op = BinOp::XorWrap;
                }
                _ => {
                    for _ in 1..t.len {
                        self.bump();
                    }
                }
            }
            let op_span = op_span.to(self.prev_span());
            if let Some(word) = t.old {
                let was = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::WrapAdd => "+~",
                    BinOp::WrapSub => "-~",
                    BinOp::Mul => "x",
                    BinOp::WrapMul => "x~",
                    BinOp::Pow => "xx",
                    _ => "xx~",
                };
                return self.fail_at(op_span, format!("`{}` is `{}` now (owner, 2026-10-03): `'a' {} 'b'`", was, word, word));
            }
            if let BinOp::Custom(name) = &op {
                if let Some((base, narrow)) = name.split_once('.') {
                    if matches!(base, "plus" | "minus" | "times" | "pow") {
                        return self.fail_at(op_span, format!("`{}.{}` is not an operator: the narrowings of `{}` are `{}.wrap` and `{}.ieee`", base, narrow, base, base, base));
                    }
                }
            }
            if op == BinOp::Join && self.is(&Tok::Semi) {
                return self.fail_at(op_span, "`++` joins text; there is no increment — write `change 'i' = 'i' plus 1;`");
            }
            // The near-misses the language refuses by name.
            if let BinOp::Custom(c) = &op {
                // The comparisons are words (owner, 2026-10-07).
                let word = match c.as_str() {
                    "!=" => Some("not-equal is `not_equal`"),
                    ">=" | ">==" => Some("greater-or-equal is `at_least`"),
                    "<=" | "<==" => Some("less-or-equal is `at_most`"),
                    _ => None,
                };
                if let Some(h) = word {
                    return self.fail_at(op_span, format!("`{}` is not an operator; {}", c, h));
                }
                let hint = match c.as_str() {
                    "/" => Some("division is `div.floor`, `div.ceil`, `div.trunc` or `div.near`: `'a' div.floor 'b'`"),
                    "--" => Some("there is no decrement: write `change 'i' = 'i' minus 1;`"),
                    "+=" | "-=" => Some("there is no compound assignment: write `change 'i' = 'i' plus 1;`"),
                    _ => None,
                };
                if let Some(h) = hint {
                    return self.fail_at(op_span, format!("`{}` is not an operator here: {}", c, h));
                }
            }
            // The operators are the Brainstem's (owner, 2026-10-08: "`a`",
            // built 2026-10-09); only `++`, which builds growing text and
            // lists, is cord's, as is a cord operator named by its path.
            let std_op = match &op {
                BinOp::Join => true,
                BinOp::Custom(c) => c.starts_with("std::"),
                _ => false,
            };
            if std_op {
                self.std_gate(&op.spelled(), op_span)?;
            }
            let rhs = self.expr(r_bp)?;
            let span = lhs.span.to(rhs.span);
            self.check_precedence(&op, op_span, &lhs, &rhs)?;
            lhs = Expr { kind: ExprKind::Binary { op, op_span, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Ok(lhs)
    }

    /// Where mathematics has no settled reading, parentheses are required.
    fn check_precedence(&self, op: &BinOp, op_span: Span, lhs: &Expr, rhs: &Expr) -> Result<()> {
        let bare_op = |e: &Expr| -> Option<BinOp> {
            match &e.kind {
                ExprKind::Binary { op, .. } => Some(op.clone()),
                _ => None,
            }
        };
        let bare_unary = |e: &Expr| matches!(e.kind, ExprKind::Unary { .. });
        // A custom operator a `precedence` level names has one.
        let custom_level = |o: &BinOp| match o {
            BinOp::Custom(name) => self.prec.get(name.rsplit("::").next().unwrap_or(name)).copied(),
            _ => None,
        };
        let nofix = |o: &BinOp| o.is_bitwise() || (matches!(o, BinOp::Custom(_)) && custom_level(o).is_none());
        for side in [lhs, rhs] {
            let inner = bare_op(side);
            // A level whose chain is refused (owner, 2026-10-03).
            if let Some(inner) = &inner {
                if let (Some((a, Chain::Refused)), b) | (b, Some((a, Chain::Refused))) = (custom_level(op), custom_level(inner)) {
                    let b = b.map(|x| x.0).or_else(|| language_level(&inner.spelled()).map(|x| x.0)).or_else(|| language_level(&op.spelled()).map(|x| x.0));
                    if b == Some(a) {
                        return self.fail_at(op_span, format!("`{}` next to `{}`: this level does not chain (`level.when_chained.refused`) — parenthesise one side", inner.spelled(), op.spelled()));
                    }
                }
            }
            if nofix(op) && (inner.is_some() || bare_unary(side)) {
                let kind = if op.is_bitwise() { "a bitwise operator has no precedence" } else { "a custom operator has no precedence" };
                return self.fail_at(op_span, format!("`{}` next to another operator: {} — parenthesise one side", op.spelled(), kind));
            }
            if let Some(inner) = &inner {
                if nofix(inner) {
                    let kind = if inner.is_bitwise() { "a bitwise operator has no precedence" } else { "a custom operator has no precedence" };
                    return self.fail_at(op_span, format!("`{}` next to `{}`: {} — parenthesise", inner.spelled(), op.spelled(), kind));
                }
                if op.is_mod() || inner.is_mod() {
                    let m = if op.is_mod() { op.spelled() } else { inner.spelled() };
                    return self.fail_at(op_span, format!("`{}` next to `{}`: mathematics has no settled reading for `{}` beside another operator — parenthesise (`('i' {} 2) equal_to 0`)", inner.spelled(), op.spelled(), m, if m.contains('.') { m.clone() } else { format!("{}.floor", m) }));
                }
                if (op.is_div() && inner.is_multiplicative()) || (op.is_multiplicative() && inner.is_div()) {
                    return self.fail_at(op_span, format!("`{}` next to `{}`: mathematics has no settled reading here (`6 div.floor 2 times 3`) — parenthesise one side", inner.spelled(), op.spelled()));
                }
                if op.is_comparison() && inner.is_comparison() {
                    return self.fail_at(op_span, format!("`{}` next to `{}`: comparisons do not chain — write `('a' < 'b') and ('b' < 'c')`", inner.spelled(), op.spelled()));
                }
            }
            if op.is_comparison() && matches!(side.kind, ExprKind::Unary { op: UnOp::Not, .. }) {
                return self.fail_at(op_span, format!("`!` next to `{}`: mathematics has no settled reading — parenthesise (`! ('a' equal_to 'b')`)", op.spelled()));
            }
        }
        Ok(())
    }

    fn prefix(&mut self) -> Result<Expr> {
        let start = self.span();
        match self.peek().clone() {
            // `*0.1`: exactly this decimal, a `deci` (owner, 2026-10-02).
            Tok::Op(o) if o == "*" && matches!(self.peek_at(1), Tok::Num { .. }) => {
                self.bump();
                let Tok::Num { text, bin } = self.peek().clone() else { unreachable!() };
                let at = self.bump().span;
                if !bin {
                    return self.fail_at(start.to(at), format!("`{}` is exact already: an integer is held as written, so `*` adds nothing — write `{}`", text, text));
                }
                let span = start.to(at);
                self.postfix_on(Expr { kind: ExprKind::Exact(text.replace('_', "")), span })
            }
            Tok::Op(o) if o == "-" => {
                self.bump();
                // A negative binds below std's `pow`: `-2 pow 2` is -4.
                let operand = self.expr(2 * NEG_LEVEL)?;
                let span = start.to(operand.span);
                Ok(Expr { kind: ExprKind::Unary { op: UnOp::Neg, expr: Box::new(operand) }, span })
            }
            // `neg.ieee 'x'` (owner, 2026-10-07: "yea"): IEEE's negation,
            // binding as `-` does.
            Tok::Word(w) if w == "neg" && matches!(self.peek_at(1), Tok::Dot) && self.is_word_at(2, "ieee") => {
                self.bump();
                self.bump();
                self.bump();
                let operand = self.expr(2 * NEG_LEVEL)?;
                let span = start.to(operand.span);
                Ok(Expr { kind: ExprKind::Unary { op: UnOp::NegIeee, expr: Box::new(operand) }, span })
            }
            Tok::Op(o) if o == "&" => {
                self.bump();
                let (verb, mut_word) = if self.is_word("mut") {
                    self.bump();
                    (Verb::LendMut, true)
                } else {
                    (Verb::Lend, false)
                };
                match self.peek() {
                    Tok::Name(_) => {}
                    Tok::Word(w) if !mut_word && w.starts_with("mut") => return self.fail(format!("`&` lends a variable: `&'x'` or `&mut 'x'`; `&{}` is neither", w)),
                    _ => return self.fail("`&` lends a variable: write `&'x'` (`lend 'x'`) or `&mut 'x'` (`lend.mut 'x'`)"),
                }
                let operand = self.postfix()?;
                let span = start.to(operand.span);
                Ok(Expr { kind: ExprKind::Verb { verb, amp: true, expr: Box::new(operand) }, span })
            }
            // "Not" is `!` (owner, 2026-10-06): spaced, it negates a value,
            // std's; glued, it marks a rule.
            Tok::Op(o) if o == "!" => {
                if self.adjacent(0, 1) {
                    return self.fail("a glued `!` marks a rule (`immut`, `no_shadow`, `no_check`, `no_cheat`); to negate a value, space it: `! 'ready'`");
                }
                self.bump();
                let operand = self.expr(2 * CMP_LEVEL)?;
                if let ExprKind::Binary { op, op_span, .. } = &operand.kind {
                    if op.is_comparison() {
                        return self.fail_at(*op_span, format!("`!` next to `{}`: mathematics has no settled reading — parenthesise (`! ('a' equal_to 'b')`)", op.spelled()));
                    }
                }
                let span = start.to(operand.span);
                Ok(Expr { kind: ExprKind::Unary { op: UnOp::Not, expr: Box::new(operand) }, span })
            }
            Tok::Op(o) => self.fail(format!("expected a value, found `{}`: an operator sits between two values", o)),
            Tok::Word(w) => match w.as_str() {
                "not" => {
                    self.bump();
                    if self.eat(&Tok::Dot) {
                        let b = self.ident("`bit`")?;
                        if b.name != "bit" {
                            return self.fail_at(b.span, format!("`not.{}` is not an operator: the bitwise one is `not.bit`, and a value's not is `!`", b.name));
                        }
                        let operand = self.expr(2 * NEG_LEVEL)?;
                        let span = start.to(operand.span);
                        return Ok(Expr { kind: ExprKind::Unary { op: UnOp::NotBit, expr: Box::new(operand) }, span });
                    }
                    let operand = self.expr(2 * CMP_LEVEL)?;
                    if let ExprKind::Binary { op, op_span, .. } = &operand.kind {
                        if op.is_comparison() {
                            return self.fail_at(*op_span, format!("`!` next to `{}`: mathematics has no settled reading — parenthesise (`! ('a' equal_to 'b')`)", op.spelled()));
                        }
                    }
                    let span = start.to(operand.span);
                    Ok(Expr { kind: ExprKind::Unary { op: UnOp::Not, expr: Box::new(operand) }, span })
                }
                "give" | "copy" | "lend" => {
                    self.bump();
                    let verb = match w.as_str() {
                        "give" => Verb::Give,
                        "copy" => Verb::Copy,
                        _ => {
                            if self.eat(&Tok::Dot) {
                                let m = self.ident("`mut` after `lend.`")?;
                                if m.name != "mut" {
                                    return self.fail_at(m.span, format!("`lend.{}` is not a verb: `lend` (shared) or `lend.mut` (exclusive, may modify)", m.name));
                                }
                                Verb::LendMut
                            } else {
                                Verb::Lend
                            }
                        }
                    };
                    if self.is_word("lend") || self.is_word("give") || self.is_word("copy") {
                        return self.fail("one verb per value");
                    }
                    let operand = self.postfix()?;
                    let span = start.to(operand.span);
                    Ok(Expr { kind: ExprKind::Verb { verb, amp: false, expr: Box::new(operand) }, span })
                }
                "lent" => self.fail("`lent` is the parameter's word; at a call the caller writes `lend 'x'` (or `lend.mut 'x'`)"),
                "check" | "nocheck" => self.fail(format!("`{}` goes in front of a statement, not inside a value: if half a statement needs it, split the statement", w)),
                "true" | "false" => {
                    self.bump();
                    // Bare, it is the boolean; a function of that name is
                    // reached by path (limbs.md, 2026-09-19).
                    if self.is(&Tok::LBracket) {
                        return self.fail(format!("bare `{}` is the boolean, so `{}[…]` is no call: a function named `{}` is reached by path — `limb.{}[…]`", w, w, w, w));
                    }
                    let e = Expr { kind: ExprKind::Bool(w == "true"), span: start };
                    self.postfix_on(e)
                }
                "fn" => {
                    self.bump();
                    self.fn_lit(start)
                }
                "to" => {
                    self.bump();
                    self.conv(start)
                }
                "channel" if !matches!(self.peek_at(1), Tok::Dot) => {
                    self.bump();
                    self.expect(&Tok::LBracket, "`[` after `channel`: `channel[T]`")?;
                    let elem = self.type_()?;
                    self.expect(&Tok::RBracket, "`]` closing the channel's type")?;
                    let bound = if self.is(&Tok::LBracket) {
                        self.bump();
                        let lo = self.expr(0)?;
                        self.expect(&Tok::Comma, "`,` between the channel's bounds")?;
                        let hi = self.expr(0)?;
                        self.expect(&Tok::RBracket, "`]` closing the bound")?;
                        Some(Box::new((lo, hi)))
                    } else {
                        None
                    };
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::Channel { elem, bound }, span })
                }
                // `ptr.cast[T]['p']` (owner, 2026-09-30: a); the other
                // `ptr.` operations are ordinary calls.
                "ptr" if matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Word(c) if c == "cast") => {
                    self.bump();
                    self.bump();
                    self.bump();
                    self.expect(&Tok::LBracket, "`[` and the type it points at, after `ptr.cast`: `ptr.cast[Point]['p']`")?;
                    let ty = self.type_()?;
                    self.expect(&Tok::RBracket, "`]`")?;
                    self.expect(&Tok::LBracket, "`[` and the pointer: `ptr.cast[Point]['p']`")?;
                    let value = self.expr(0)?;
                    self.expect(&Tok::RBracket, "`]`")?;
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::PtrCast { ty, value: Box::new(value) }, span })
                }
                // A file's bytes, read while building (owner, 2026-10-10).
                "incl" => {
                    self.bump();
                    if !self.eat(&Tok::Dot) {
                        return self.fail("`incl` is followed by what it makes of the file: `incl.bytes[\"assets/logo.png\"]`, `incl.str.utf8[…]`, `incl.str.ascii[…]`");
                    }
                    let what = self.ident("`bytes` or `str`")?;
                    let form = match what.name.as_str() {
                        "C" => return self.fail_at(start, "`incl.C` sits at the top level of a file or in a `module` block, as `extern.C` does, never in a body"),
                        "c" => return self.fail_at(what.span, "C is written capitalized here, as every language's name is: `incl.C`"),
                        "bytes" => InclForm::Bytes,
                        "str" => {
                            self.expect(&Tok::Dot, "`.` and the text's encoding: `incl.str.utf8[…]`")?;
                            let enc = self.ident("the text's encoding")?;
                            match enc.name.as_str() {
                                "utf8" | "ascii" => {
                                    if self.is(&Tok::Dot) {
                                        return self.fail_at(enc.span, format!("`str.{}` has no byte order: write `incl.str.{}[…]`", enc.name, enc.name));
                                    }
                                    if enc.name == "utf8" { InclForm::Utf8 } else { InclForm::Ascii }
                                }
                                // Which byte order a file without a mark has is said
                                // (owner, 2026-10-10: "saying the order in syntax").
                                "utf16" | "utf32" => {
                                    let says = format!("`incl.str.{e}` says its byte order: `incl.str.{e}.low_first[…]` (\"A\" stored low byte first, as Windows writes it) or `incl.str.{e}.high_first[…]`", e = enc.name);
                                    if !self.eat(&Tok::Dot) {
                                        return self.fail_at(enc.span, says);
                                    }
                                    let order = self.ident("`low_first` or `high_first`")?;
                                    let low_first = match order.name.as_str() {
                                        "low_first" => true,
                                        "high_first" => false,
                                        _ => return self.fail_at(order.span, says),
                                    };
                                    if enc.name == "utf16" { InclForm::Utf16 { low_first } } else { InclForm::Utf32 { low_first } }
                                }
                                other => return self.fail_at(enc.span, format!("`incl.str.{}` is not an encoding: `incl.str.utf8[…]`, `incl.str.ascii[…]`, or `incl.str.utf16` / `utf32` with its byte order", other)),
                            }
                        }
                        other => return self.fail_at(what.span, format!("`incl.{}` does not exist: `incl.bytes[…]` gives the bytes, `incl.str.utf8[…]` the text", other)),
                    };
                    self.expect(&Tok::LBracket, "`[` and the file's path, in quotes")?;
                    let path_span = self.span();
                    let Tok::Str(path) = self.peek().clone() else {
                        return self.fail("an `incl` path is written in quotes, as text (`incl.bytes[\"assets/logo.png\"]`): it is read while building");
                    };
                    self.bump();
                    if !self.is(&Tok::RBracket) {
                        return self.fail_at(path_span, "an `incl` path is written in quotes, as text (`incl.bytes[\"assets/logo.png\"]`): it is read while building");
                    }
                    self.bump();
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::Incl { form, path, path_span }, span })
                }
                "size" | "align" => {
                    self.bump();
                    if !self.eat(&Tok::Dot) {
                        return self.fail(format!("`{w}` is a keyword: a type's {w} is `{w}.of[int32]` (`.of` after `{w}`), and a function named `{w}`, a C header's say, is reached through a path, `my.{w}[…]` or its module's"));
                    }
                    let of = self.ident("`of`")?;
                    if of.name != "of" {
                        return self.fail_at(of.span, format!("`{}.{}` does not exist: `{}.of[T]`", w, of.name, w));
                    }
                    self.expect(&Tok::LBracket, &format!("`[` after `{}.of`", w))?;
                    let ty = self.type_()?;
                    self.expect(&Tok::RBracket, "`]`")?;
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::SizeOf { align: w == "align", ty }, span })
                }
                "range" => {
                    self.bump();
                    if self.is(&Tok::Dot) {
                        return self.fail("`range` builds its bounds from values; a counter's type goes on its name: `for 'i': int32 in range[0, 3]`");
                    }
                    self.expect(&Tok::LBracket, "`[` after `range`")?;
                    self.eat_word("from");
                    let s = self.expr(0)?;
                    self.expect(&Tok::Comma, "`,`")?;
                    self.eat_word("to");
                    let e = self.expr(0)?;
                    let step = if self.eat(&Tok::Comma) {
                        self.eat_word("by");
                        Some(Box::new(self.expr(0)?))
                    } else {
                        None
                    };
                    self.expect(&Tok::RBracket, "`]`")?;
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::Range { start: Box::new(s), stop: Box::new(e), step }, span })
                }
                "otherwise" => self.fail("`otherwise` follows a call that can fail: `let 'x' := f[] otherwise 0;`"),
                // `op +` (owner, 2026-09-29): the operator as a value.
                "op" if !matches!(self.peek_at(1), Tok::LBracket) => {
                    let start = self.bump().span;
                    // `op neg` / `op not`: the operators of one operand.
                    let unary = if self.is_word("neg") {
                        self.bump();
                        Some((UnOp::Neg, "neg"))
                    } else if self.is_word("not") && !matches!(self.peek_at(1), Tok::Dot) {
                        self.bump();
                        Some((UnOp::Not, "not"))
                    } else if self.is_word("not") && self.is_word_at(2, "bit") {
                        self.bump();
                        self.bump();
                        self.bump();
                        Some((UnOp::NotBit, "not.bit"))
                    } else {
                        None
                    };
                    let (op, spelled, unary) = match unary {
                        Some((u, s)) => (BinOp::Custom(s.to_string()), s.to_string(), Some(u)),
                        None => {
                            let (op, spelled) = self.op_as_value()?;
                            (op, spelled, None)
                        }
                    };
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::OpValue { op, spelled, unary }, span })
                }
                "match" => {
                    let start = self.bump().span;
                    let (value, arms) = self.match_parts()?;
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::Match { value: Box::new(value), arms }, span })
                }
                "lang" if matches!(self.peek_at(1), Tok::Colon) => self.fail("`lang:` was removed: keywords are bare"),
                w if is_keyword(w) && w != "map" && matches!(self.peek_at(1), Tok::LBracket) => {
                    self.fail(format!("`{}` is a keyword and is never bare: a function named with one is reached only through a path — `limb.{}[…]` inside its own Neuron, `name::{}[…]` from outside", w, w, w))
                }
                "let" | "return" | "if" | "else" | "loop" | "while" | "for" | "break" | "continue" | "fail" | "concur" | "whichever" | "struct" | "enum" | "pub" | "const" | "module" | "extern" | "interface" | "op" | "satisfies" | "with" | "in" | "from" | "temp" | "perm" | "seq" | "mut" | "immut" | "inline" | "layout" | "var" | "func" | "MAIN" => {
                    self.fail(format!("expected a value, found `{}`", w))
                }
                // Another Limb's const, straight after `::`: `areas::'pi'`
                // (`limbs.md`).
                _ if matches!(self.peek_at(1), Tok::ColonColon) && matches!(self.peek_at(2), Tok::Name(_)) => {
                    let limb = self.ident("a Neuron's name")?;
                    self.bump();
                    let Tok::Name(n) = self.peek().clone() else { unreachable!() };
                    let span = self.bump().span;
                    if limb.name == "std" {
                        self.std_gate(&format!("cord::'{}'", n), limb.span.to(span))?;
                    }
                    let path = Path { limb: Some(limb.clone()), segs: Vec::new(), span: limb.span.to(span) };
                    let e = Expr { span: start.to(span), kind: ExprKind::ConstPath { path, name: QName { name: n, span } } };
                    self.postfix_on(e)
                }
                _ => {
                    let path = self.path()?;
                    // a const by path: `a.'k'`, `std::'pi'`, `limb.'scale'`
                    if self.is(&Tok::Dot) {
                        if let Tok::Name(n) = self.peek_at(1).clone() {
                            self.bump();
                            let span = self.bump().span;
                            let name = QName { name: n, span };
                            let e = Expr { span: start.to(span), kind: ExprKind::ConstPath { path, name } };
                            return self.postfix_on(e);
                        }
                    }
                    if path.segs.len() == 1 && path.limb.is_none() && path.head() == "my" && !self.is(&Tok::Dot) {
                        return self.fail_at(path.span, "`my` is followed by `.` and the root item it reaches: `my.helper[…]`");
                    }
                    // `std::print.weird…` takes each value after a `<-`, C++'s
                    // `cout <<` (owner, 2026-10-01), and no `[ ]`.
                    // Bare too: stdkit is `bare.optional` for all (owner,
                    // 2026-10-06).
                    let weird = path.limb.as_ref().is_none_or(|l| l.name == "std") && path.segs.len() >= 2 && path.segs[0].name == "print" && path.segs.iter().any(|s| s.name == "weird");
                    if weird && self.is(&Tok::LBracket) {
                        return self.fail(format!("`{}` takes its values with arrows, not `[ ]`: `{} <- \"n is \" <- 'n' <- newln;`", path.spelled(), path.spelled()));
                    }
                    if weird && self.is_op("<-") {
                        let mut args = Vec::new();
                        while self.eat_op("<-") {
                            // Each value up to the next arrow: a custom
                            // operator binds loosest, so it stops there.
                            let v = self.expr(1)?;
                            args.push(Arg::Positional(v));
                        }
                        let span = start.to(self.prev_span());
                        let e = Expr { kind: ExprKind::Call { path, args, otherwise: None }, span };
                        return Ok(e);
                    }
                    if weird {
                        return self.fail(format!("`{}` takes at least one value, each after a `<-`: `{} <- 'n' <- newln;`", path.spelled(), path.spelled()));
                    }
                    if self.is(&Tok::LBracket) {
                        self.bump();
                        let args = self.args()?;
                        let span = start.to(self.prev_span());
                        let e = Expr { kind: ExprKind::Call { path, args, otherwise: None }, span };
                        self.postfix_on(e)
                    } else {
                        let span = path.span;
                        let e = Expr { kind: ExprKind::Path(path), span };
                        self.postfix_on(e)
                    }
                }
            },
            Tok::Ellipsis => {
                self.bump();
                let verb = if self.eat_word("copy") {
                    Some(Verb::Copy)
                } else if self.eat_word("give") {
                    Some(Verb::Give)
                } else if self.is_word("lend") {
                    return self.fail("`...` with no verb already lends each element; `...lend` is not written");
                } else {
                    None
                };
                let operand = self.postfix()?;
                let span = start.to(operand.span);
                Ok(Expr { kind: ExprKind::Spread { verb, expr: Box::new(operand) }, span })
            }
            _ => self.postfix(),
        }
    }

    /// A primary followed by `.field`, `[i]`, `[args]`.
    fn postfix(&mut self) -> Result<Expr> {
        let start = self.span();
        let e = match self.peek().clone() {
            Tok::Name(n) => {
                self.bump();
                Expr { kind: ExprKind::Name(QName { name: n, span: start }), span: start }
            }
            Tok::Str(s) => {
                self.bump();
                Expr { kind: ExprKind::Str(s), span: start }
            }
            Tok::Char(c) => {
                self.bump();
                Expr { kind: ExprKind::Char(c), span: start }
            }
            Tok::Newline => {
                self.bump();
                Expr { kind: ExprKind::Newline, span: start }
            }
            Tok::Num { text, bin } => {
                self.bump();
                if let Tok::Word(u) = self.peek().clone() {
                    if DURATION_UNITS.contains(&u.as_str()) {
                        let unit = self.ident("a unit")?;
                        if bin {
                            return self.fail_at(start.to(unit.span), "a duration is a whole number of its unit: `1500 millis`, not `1.5 seconds`");
                        }
                        let span = start.to(unit.span);
                        return Ok(Expr { kind: ExprKind::Duration { value: text, unit }, span });
                    }
                }
                Expr { kind: if bin { ExprKind::Bin(text) } else { ExprKind::Int(text) }, span: start }
            }
            Tok::LParen => {
                self.bump();
                let inner = self.expr(0)?;
                self.expect(&Tok::RParen, "`)`")?;
                let span = start.to(self.prev_span());
                Expr { kind: ExprKind::Paren(Box::new(inner)), span }
            }
            Tok::LBracket => {
                self.bump();
                let mut items = Vec::new();
                let mut trailing = false;
                while !self.is(&Tok::RBracket) {
                    items.push(self.expr(0)?);
                    trailing = false;
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                    trailing = true;
                }
                self.expect(&Tok::RBracket, "`]` closing the list")?;
                let span = start.to(self.prev_span());
                if items.len() == 1 && !trailing {
                    // `[expr]` is grouping: it builds nothing.
                    let inner = items.pop().unwrap();
                    Expr { kind: ExprKind::Paren(Box::new(inner)), span }
                } else {
                    Expr { kind: ExprKind::List(items), span }
                }
            }
            Tok::Word(_) | Tok::Op(_) | Tok::Ellipsis => return self.prefix(),
            Tok::Raw(_) => return self.fail("only `inline.ASM` holds raw text"),
            other => return self.fail(format!("expected a value, found {}", other.describe())),
        };
        self.postfix_on(e)
    }

    fn postfix_on(&mut self, mut e: Expr) -> Result<Expr> {
        loop {
            match self.peek().clone() {
                Tok::Dot => {
                    // `'x'.field`; `'x'.'y'` is an error
                    match self.peek_at(1).clone() {
                        Tok::Word(_) => {
                            self.bump();
                            let field = self.ident("a field name")?;
                            let span = e.span.to(field.span);
                            e = Expr { kind: ExprKind::Field { base: Box::new(e), field }, span };
                        }
                        Tok::Name(n) => {
                            self.bump();
                            return self.fail(format!("a field's name is bare: `.{}`, not `.'{}'` — quotes mark the value you hold, the route into it is bare", n, n));
                        }
                        _ => break,
                    }
                }
                Tok::LBracket => {
                    // `'xs'[2]` reads a list and `'f'[2]` calls a function
                    // (owner, 2026-09-27: `pos` is removed). One plain value
                    // is read as an index here; the checker makes it a call
                    // when the value holds a function.
                    match &e.kind {
                        ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::CallValue { .. } | ExprKind::Index { .. } => {}
                        _ => break,
                    }
                    self.bump();
                    let mut args = self.args()?;
                    let span = e.span.to(self.prev_span());
                    // `pos` on every position (owner, 2026-10-07), so a read
                    // is `[pos …]` and anything else is a call.
                    let one = args.len() == 1 && matches!(&args[0], Arg::Positional(x) if matches!(x.kind, ExprKind::Pos(_)));
                    if one {
                        let Some(Arg::Positional(index)) = args.pop() else { unreachable!() };
                        let index = match index.kind {
                            ExprKind::Pos(inner) => *inner,
                            _ => index,
                        };
                        e = Expr { kind: ExprKind::Index { base: Box::new(e), index: Box::new(index) }, span };
                    } else {
                        e = Expr { kind: ExprKind::CallValue { callee: Box::new(e), args, otherwise: None }, span };
                    }
                }
                _ => break,
            }
        }
        // Two values side by side with nothing between.
        match self.peek() {
            Tok::Str(_) | Tok::Char(_) | Tok::Newline | Tok::Num { .. } | Tok::Name(_) => {
                let here = self.span();
                return self.fail_at(e.span.to(here), "two values next to each other with nothing between: join text with `++`, separate values with `,`");
            }
            _ => {}
        }
        Ok(e)
    }

    /// The contents of a `[ … ]` after a path: positional values, `name = value`
    /// fields, or `key = value` pairs. Consumes the closing `]`.
    fn args(&mut self) -> Result<Vec<Arg>> {
        let mut args = Vec::new();
        while !self.is(&Tok::RBracket) {
            if self.at_eof() {
                return self.fail("this `[` is never closed");
            }
            // `field = value`. A keyword names a field too — `layout` (a
            // no-OS target's linker script, owner 2026-10-09), a C struct's
            // `size` (owner, 2026-10-10: `buf[size = 42]`) — as no keyword
            // stands before `=` otherwise.
            if let Tok::Word(_) = self.peek() {
                if self.is_op_at(1, "=") {
                    let name = self.ident("a field")?;
                    self.bump(); // =
                    let value = self.expr(0)?;
                    args.push(Arg::Named(name, value));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                    continue;
                }
            }
            if let Tok::Name(n) = self.peek().clone() {
                if self.is_op_at(1, "=") {
                    return self.fail(format!("a field's name is bare in a construction: `{} = …`, not `'{}' = …`", n, n));
                }
            }
            // The labels (owner, 2026-10-07): `pos 2` where a position
            // goes, `list.fill[value 0, count 10]`, `simd.fill[value 0]` —
            // the words, not a function someone named so.
            let label = ["pos", "count", "value"].into_iter().find(|w| self.is_word(w));
            if let Some(w) = label.filter(|_| !matches!(self.peek_at(1), Tok::LBracket | Tok::Dot | Tok::ColonColon | Tok::Comma | Tok::RBracket)) {
                let start = self.span();
                self.bump();
                let inner = Box::new(self.expr(0)?);
                let span = start.to(self.prev_span());
                let kind = match w {
                    "pos" => ExprKind::Pos(inner),
                    "count" => ExprKind::Count(inner),
                    _ => ExprKind::Value(inner),
                };
                args.push(Arg::Positional(Expr { kind, span }));
                if !self.eat(&Tok::Comma) {
                    break;
                }
                continue;
            }
            let e = self.expr(0)?;
            if self.is(&Tok::Walrus) {
                return self.fail("a field's value is given with `=`, not `:=`: the blueprint fixed the types");
            }
            if self.is_op("=") {
                self.bump();
                let v = self.expr(0)?;
                args.push(Arg::Pair(e, v));
            } else {
                args.push(Arg::Positional(e));
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the arguments")?;
        Ok(args)
    }

    fn fn_lit(&mut self, start: Span) -> Result<Expr> {
        let params = self.params(ParamContext::Fn)?;
        let (ret, fails) = self.return_spec()?;
        if let Some(r) = self.ret_lent.take() {
            return self.fail_at(r.span, "not yet: only a named `fn` hands back a borrow (`fn f [lent 'a': T] -> lent T`)");
        }
        let captures = if self.eat_word("with") { self.captures()? } else { Vec::new() };
        let outer = std::mem::replace(&mut self.returns_value, false);
        let body = self.block("the function")?;
        if self.returns_value && ret.is_none() {
            return self.fail_at(start, "this function returns a value, so it says its type: `-> int32`");
        }
        self.returns_value = outer;
        let span = start.to(self.prev_span());
        Ok(Expr { kind: ExprKind::FnLit(Box::new(FnLit { params, ret, fails, captures, body })), span })
    }

    /// After `to`: `.type[value]`, with an optional bound bracket first.
    fn conv(&mut self, start: Span) -> Result<Expr> {
        if !self.is(&Tok::Dot) {
            return self.fail("the target type follows `to` after a dot: `to.int32[…]`, `to.str.utf8[…]`");
        }
        self.bump();
        // `to.list[give 'a']`, `to.array[copy 'l']`: the element type stays,
        // so it is not written and the bracket is the value (`lists.md`).
        // They take ownership like a call, so they are parsed as one.
        if let Tok::Word(w) = self.peek().clone() {
            if (w == "list" || w == "array") && matches!(self.peek_at(1), Tok::LBracket) {
                let wspan = self.bump().span;
                self.bump();
                let args = self.args()?;
                let path = Path { limb: None, segs: vec![Ident { name: "to".into(), span: start }, Ident { name: w, span: wspan }], span: start.to(wspan) };
                if self.is(&Tok::LBracket) && !self.bracket_is_bound() {
                    let what = path.spelled();
                    return self.fail(format!("`{}` keeps the element type, so it is not written: `{}[copy 'a']`", what, what));
                }
                let span = start.to(self.prev_span());
                let e = Expr { kind: ExprKind::Call { path, args, otherwise: None }, span };
                return self.postfix_on(e);
            }
        }
        let target = self.type_in(true)?;
        if !self.is(&Tok::LBracket) {
            return self.fail(format!("`to.{}` takes the value to convert in brackets: `to.{}[…]`", crate::names::type_spelling(&target), crate::names::type_spelling(&target)));
        }
        self.bump();
        let e = self.expr(0)?;
        if self.is(&Tok::Comma) {
            return self.fail("a conversion takes exactly one value");
        }
        self.expect(&Tok::RBracket, "`]` closing the conversion")?;
        let span = start.to(self.prev_span());
        let e = Expr { kind: ExprKind::Conv { target, expr: Box::new(e), otherwise: None }, span };
        self.postfix_on(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParamContext {
    Fn,
    Op,
    Extern,
    Iface,
}

pub fn is_device_name(n: &str) -> bool {
    matches!(n, "con" | "nul" | "aux" | "prn")
        || (n.len() == 4 && (n.starts_with("com") || n.starts_with("lpt")) && n.as_bytes()[3].is_ascii_digit() && n.as_bytes()[3] != b'0')
}

#[allow(dead_code)]
const _: &str = NOT_A_COMMENT;

/// An item into its file's or module's list: an `extern.C { … }` block's
/// lines, each with the block's visibility.
fn push_item(items: &mut Vec<Item>, it: Item) {
    match it.kind {
        ItemKind::ExternBlock(lines) => items.extend(lines.into_iter().map(|(v, kind)| Item { vis: if it.vis == Vis::Private { v } else { it.vis }, kind, span: it.span, bare: it.bare, case: it.case })),
        kind => items.push(Item { vis: it.vis, kind, span: it.span, bare: it.bare, case: it.case }),
    }
}
