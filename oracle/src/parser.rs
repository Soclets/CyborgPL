//! Recursive descent over the token list. Grammar is in the comments of each fn.
//!
//! Syntax revised 2026-09-19 (design/syntax-sketch.md): `let`/`fn`, the type
//! after the name (`'x': int32`), `-> T` for a return type, bare `if`/`else if`/
//! `while`/`for`, and `[ ]` only where it builds a value (text join, list, call,
//! construction, index, `range[…]`).

use crate::ast::*;
use crate::lexer::{Tok, Token};
use crate::value::ty_name;

const DURATION_UNITS: &[(&str, i128)] = &[("nano", 1), ("nanos", 1), ("micro", 1_000), ("micros", 1_000), ("milli", 1_000_000), ("millis", 1_000_000), ("second", 1_000_000_000), ("seconds", 1_000_000_000), ("minute", 60_000_000_000), ("minutes", 60_000_000_000), ("hour", 3_600_000_000_000), ("hours", 3_600_000_000_000)];

pub const KEYWORDS: &[&str] = &[
    "fn", "let", "change", "giveS", "gives", "return", "if", "else", "loop", "while", "for", "in", "break", "continue",
    "give", "copy", "lend", "lent", "op", "check", "nocheck", "NO_FUCKING_CHECKS", "true", "false", "mut", "immut",
    "temp", "perm", "range", "seq", "para", "list", "array", "map", "simd", "channel", "concur", "whichever", "from", "struct", "pub", "const", "module", "limb", "my", "on", "fail", "otherwise", "to", "extern", "layout", "ptr", "size", "align", "inline", "asm", "ASM", "enum", "match", "interface", "satisfies", "xor", "shl", "shr", "with", "test", "must",
    // retired 2026-09-19, still reserved so the errors can point at the new spelling
    "var", "func",
];

/// A core is listed by its registry ID and version (owner, 2026-10-06: "ID", then "`a`"); the
/// official ones, from `openssl rand -hex 32` (owner: "yea"), all first released at `CORE_VERSION`
pub const CORE_IDS: [(&str, &str); 10] = [
    ("IEG", "5d0e846b2a0f57f6fae20216e8d856d41b6257e0a6104aaf0bc55b57b44b4085"),
    ("ASP", "4882d1f3bf7b11d59879e4921672f15e91369e25d55a10e3dcd24e9b3fbe676f"),
    ("Genny", "c3e299d99a13567b2039c13fc17d0ca562be9d6f657f8c40a4774081983b20c9"),
    ("MCPS", "7d0ccfe6e41eed3a4d8ffe00ccaada9b89a4b9a3ea553a9d869cb6902e2ca75d"),
    ("AutoGPU", "67b2514aab0ffa5ca9c3ae48144498880466d68aa3b8b8f8e1082e6032c24c4d"),
    ("Metal", "3e7b81fec072c86d58538d10978aff1d42814c9e907f1259766851795cded9b7"),
    ("Nvidia", "beefcce9ed7cccf0d62f5a6e004c4bd53c6ae9e3a45c48883d3b1dc4be913361"),
    ("Vulkan", "63ff7ed8da5e0eead0164488e89553b240501f16d4be863ba6be1e46b51ec3ec"),
    ("WSTN", "e86b0df56e35b02b2ee91df6d89e63011564364d404ed7f5ea25d768d6d854d4"),
    ("MTM", "9c6c773027ae873a736d2e9b2dc398fd4807a7df6cf43b7e233c3447af3d6a56"),
];
/// WSTN and MTM came later the same day, each first released when its ID was made (owner, 2026-10-06)
pub const LATER_CORE_VERSIONS: [(&str, &str); 2] = [("WSTN", "2026-10-06T18:48:22+07:00"), ("MTM", "2026-10-06T19:18:54+07:00")];
pub fn core_version(core: &str) -> &'static str { LATER_CORE_VERSIONS.iter().find(|(n, _)| *n == core).map(|(_, v)| *v).unwrap_or(CORE_VERSION) }
pub const CORE_VERSION: &str = "2026-10-06T18:04:29+07:00";

/// The manifest line that lists an official core, for a message to show
pub fn core_line(core: &str) -> String {
    let id = CORE_IDS.iter().find(|(n, _)| *n == core).map(|(_, i)| *i).unwrap_or("");
    format!("cca::Core[id = \"{id}\", version = \"{}\"]", core_version(core))
}

/// The target the oracle stands in for (cca/targets.txt has only this one)
pub const ORACLE_TARGET: &str = "aarch64-macos";

/// `on.machine.chip X`: what the machine the program runs on can do, asked once (owner, 2026-10-06:
/// IEG's, "the OS's `sysctl` on an ARM Mac"); the oracle asks the machine it runs on
fn machine_has(f: &str) -> Result<bool, String> {
    #[cfg(target_arch = "aarch64")]
    let r = match f {
        "aarch64" => Some(true), "x86_64" | "x86" | "riscv64" | "arm" => Some(false),
        "neon" => Some(std::arch::is_aarch64_feature_detected!("neon")),
        "sve" => Some(std::arch::is_aarch64_feature_detected!("sve")),
        "sve2" => Some(std::arch::is_aarch64_feature_detected!("sve2")),
        "aes" => Some(std::arch::is_aarch64_feature_detected!("aes")),
        "sha2" => Some(std::arch::is_aarch64_feature_detected!("sha2")),
        "sha3" => Some(std::arch::is_aarch64_feature_detected!("sha3")),
        "crc" => Some(std::arch::is_aarch64_feature_detected!("crc")),
        "lse" => Some(std::arch::is_aarch64_feature_detected!("lse")),
        "dotprod" => Some(std::arch::is_aarch64_feature_detected!("dotprod")),
        "fp16" => Some(std::arch::is_aarch64_feature_detected!("fp16")),
        "bf16" => Some(std::arch::is_aarch64_feature_detected!("bf16")),
        "i8mm" => Some(std::arch::is_aarch64_feature_detected!("i8mm")),
        _ => None,
    };
    #[cfg(not(target_arch = "aarch64"))]
    let r = match f { "x86_64" => Some(cfg!(target_arch = "x86_64")), "aarch64" => Some(false), _ => None };
    r.ok_or_else(|| format!("`{f}` is not a chip or feature the oracle can ask about: aarch64, x86_64, neon, sve, sve2, aes, sha2, sha3, crc, lse, dotprod, fp16, bf16, i8mm"))
}

/// std's registry ID (owner, 2026-10-06: design claude's `openssl rand -hex 32`) and the version the
/// oracle stands in for (its release's date and time with the offset, owner: "date and time of update")
pub const STD_ID: &str = "fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936";
pub const STD_VERSION: &str = "2026-10-06T12:57:14+07:00";

/// The operators that are std's (owner, 2026-10-05/06), as the oracle spells them inside; `and.then` and
/// `or.else` stay Signatar's, and a Limb's own operators are that Limb's.
fn is_std_op(op: &str) -> bool {
    matches!(op, "+" | "-" | "x" | "+~" | "-~" | "x~" | "+.ieee" | "-.ieee" | "x.ieee" | "pow" | "pow.wrap" | "pow.ieee" | "==" | "!==" | "<" | ">" | "</==" | ">/==" | "and" | "or" | "++")
        || BIT_OPS.contains(&op) || op.starts_with("div.") || op.starts_with("mod.")
}

/// std's names the oracle emulates, which may be written without `std::` since std marks everything
/// `bare.optional` (owner, 2026-10-06: "optional for all")
const STD_BARE_HEADS: &[&str] = &["print", "printu", "printf", "exit", "time", "input", "round", "floor", "ceil", "trunc"];
const STD_STR_OPS: &[&str] = &["split", "find", "contains", "trim", "upper", "lower", "replace", "starts_with", "ends_with", "at", "slice", "join"];

/// An operator as code writes it (`+` inside is `plus`).
fn op_shown(op: &str) -> String {
    match op { "+" => "plus".into(), "-" => "minus".into(), "x" => "times".into(), "+.ieee" => "plus.ieee".into(), "-.ieee" => "minus.ieee".into(), "x.ieee" => "times.ieee".into(), "+~" => "plus.wrap".into(), "-~" => "minus.wrap".into(), "x~" => "times.wrap".into(), o => o.into() }
}

const OP_WORDS: &[&str] = &["x", "xx", "div", "mod", "and", "or"];
/// Bitwise operators (decided 2026-09-21): the boolean words narrowed with `.bit`, plus xor/shl/shr;
/// no precedence — parentheses whenever one meets another operator.
const BIT_OPS: &[&str] = &["and.bit", "or.bit", "xor", "shl", "shr", "and.bit.wrap", "or.bit.wrap", "xor.wrap", "shl.masked", "shr.masked"];

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    /// the module being parsed: [] for limb.cpls, ["shapes"] for shapes.cpls,
    /// extended by each `module name { … }` block
    module: Vec<String>,
    /// set while parsing the type in `to.<type>[…]`: the bracket belongs to the
    /// conversion, not to the type
    to_target: bool,
    /// `#strict` at the top of this file: types, `pos`, `mut`/`immut`, `temp`/`perm` written out
    strict: bool,
    /// a `return <value>` seen in the body being parsed (a `#strict` fn must then write `-> T`)
    saw_value_return: bool,
    /// inside a `match` / `otherwise` case's block: `giveS` may appear
    arm_depth: usize,
    /// set while parsing a map literal's key: the `=` after it pairs, it is not an assignment
    in_map_key: bool,
    /// set while parsing an extern signature or a layout.C struct: `ptr[T]` is allowed
    at_boundary: bool,
    /// set while parsing an interface's entries: signatures end with `;`, no body
    in_interface: bool,
    /// `<T: Ordered>` bounds seen while parsing the current signature
    bounds: Vec<(String, String)>,
    /// the generics announced by the item being parsed (owner, 2026-10-02); `None` outside one
    declared: Option<Vec<String>>,
    /// the lifetime groups it announced
    groups: Vec<String>,
    /// captures of the anonymous function just parsed
    pending_captures: Vec<(Verb, String)>,
    /// where `my.` points: `[]` for the Limb being built, `["::geometry"]` for a dependency
    /// loaded beside it (a segment no source can write, so the two never meet)
    limb_root: Vec<String>,
    /// inside a `test fn`'s body: `must` may appear (owner, 2026-09-29)
    in_test: bool,
    /// how many tokens the operator `binop` just read takes (`plus` 1, `times.wrap` 3, `std::pow` 3)
    op_len: std::cell::Cell<usize>,
    /// it was a retired symbol (`+`, `x`, `xx`, `x~` …), refused outside the oracle's own stand-ins
    op_old: std::cell::Cell<bool>,
    /// the oracle's own stand-in functions (`op plus` as a value) are written with the symbols
    pub internal_ops: bool,
    /// this Limb's operator levels from its `precedence` block: name → (level, how a chain reads:
    /// 0 left first, 1 right first, 2 refused), on the language's scale (`base_level`)
    pub user_levels: std::collections::HashMap<String, (u16, u8)>,
    /// this file's Limb does not list std in its `CCA.cplr` (owner, 2026-10-06)
    no_std: bool,
    /// the official cores this file's Limb does not list
    lacks: Vec<String>,
}

type PResult<T> = Result<T, ParseError>;

thread_local! {
    /// Every `precedence` block of the program, all its Limbs' (owner, 2026-10-03: a Limb's levels
    /// apply wherever its operators are used); filled by a first pass over every file (`main.rs`)
    /// What each Limb's `CCA.cplr` does not list, by the Limb's root (`[]` for the one being built,
    /// `["::areas"]` for a dependency): `std`, and the official cores by name (owner, 2026-10-05/06:
    /// "Limbs should say what cores they are built for"). A Limb with no `CCA.cplr` lacks nothing.
    pub static LACKS: std::cell::RefCell<Vec<(Vec<String>, Vec<String>)>> = std::cell::RefCell::new(Vec::new());
    /// Other Limbs' structs and enums that may be written bare (`bare.optional` / `bare.only`): the path
    /// as written, the canonical name, the Limb's root, and whether it is `bare.only` (owner, 2026-10-06)
    pub static BARE_TYPES: std::cell::RefCell<Vec<(Vec<String>, String, String, bool)>> = std::cell::RefCell::new(Vec::new());
    /// Bare type names that resolved to another Limb's: (this Limb's would-be name, theirs, as written,
    /// their root), checked once every file is read, since this Limb's own does not win
    pub static BARE_TYPE_USES: std::cell::RefCell<Vec<(String, String, String, String)>> = std::cell::RefCell::new(Vec::new());
    /// Every call written with no path: (the module it is in, the path as written, whether std may be
    /// meant, its line), checked for clashes once the program is read, whether or not it runs
    pub static BARE_CALLS: std::cell::RefCell<Vec<(Vec<String>, Vec<String>, bool, usize)>> = std::cell::RefCell::new(Vec::new());
    /// Power uses seen while parsing (owner, 2026-10-07): the Limb's root, the power's path, the thing
    /// named, the line. C calls and C variables are found once the program is read (CALL_SITES, VAR_SITES).
    pub static POWER_USES: std::cell::RefCell<Vec<(Vec<String>, String, Option<String>, usize)>> = std::cell::RefCell::new(Vec::new());
    pub static CALL_SITES: std::cell::RefCell<Vec<(Vec<String>, Vec<String>, usize)>> = std::cell::RefCell::new(Vec::new());
    pub static VAR_SITES: std::cell::RefCell<Vec<(Vec<String>, String, usize)>> = std::cell::RefCell::new(Vec::new());
    pub static SHARED_LEVELS: std::cell::RefCell<std::collections::HashMap<String, (u16, u8)>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self { Parser { toks, pos: 0, module: Vec::new(), to_target: false, strict: false, saw_value_return: false, arm_depth: 0, in_map_key: false, at_boundary: false, in_interface: false, bounds: Vec::new(), declared: None, groups: Vec::new(), pending_captures: Vec::new(), limb_root: Vec::new(), in_test: false, op_len: std::cell::Cell::new(1), op_old: std::cell::Cell::new(false), internal_ops: false, no_std: false, lacks: Vec::new(), user_levels: SHARED_LEVELS.with(|l| l.borrow().clone()) } }
    /// A parser for a file that is the module `module` (its path from the root).
    pub fn new_in(toks: Vec<Token>, module: Vec<String>) -> Self { Parser { toks, pos: 0, module, to_target: false, strict: false, saw_value_return: false, arm_depth: 0, in_map_key: false, at_boundary: false, in_interface: false, bounds: Vec::new(), declared: None, groups: Vec::new(), pending_captures: Vec::new(), limb_root: Vec::new(), in_test: false, op_len: std::cell::Cell::new(1), op_old: std::cell::Cell::new(false), internal_ops: false, no_std: false, lacks: Vec::new(), user_levels: SHARED_LEVELS.with(|l| l.borrow().clone()) } }

    /// A path written in this module, made canonical (from the root):
    /// `my.x.f` → x.f; a relative `x.f` → <this module>.x.f  (decided 2026-09-19)
    /// A parser for a dependency Limb's file: its modules sit under `limb_root`.
    pub fn new_in_limb(toks: Vec<Token>, module: Vec<String>, limb_root: Vec<String>) -> Self {
        let mut p = Parser::new_in(toks, module);
        p.lacks = LACKS.with(|l| l.borrow().iter().find(|(r, _)| *r == limb_root).map(|(_, v)| v.clone()).unwrap_or_default());
        p.no_std = p.lacks.iter().any(|c| c == "std");
        p.limb_root = limb_root;
        p
    }

    /// What a core brings is refused in a Limb whose `CCA.cplr` does not list that core (owner, 2026-10-05:
    /// "Limbs should say what cores they are built for").
    fn core_missing<T>(&self, core: &str, what: &str) -> PResult<T> {
        self.err(format!("{what} is {core}'s, and this Limb's CCA.cplr does not list the core {core}: add `{}` to its `cores` ({core})", core_line(core)))
    }

    /// A use of a power, for the check once every file is read (owner, 2026-10-07)
    fn use_power(&self, path: &str, thing: Option<String>, line: usize) {
        POWER_USES.with(|u| u.borrow_mut().push((self.limb_root.clone(), path.to_string(), thing, line)));
    }

    /// `bare.never`, `bare.optional` or `bare.only`, at the cursor's `bare` (owner, 2026-10-06); a plain
    /// `bare` does not say whether the path is optional, so it is refused, as a plain `div` is.
    fn bare_mode(&mut self) -> PResult<Bare> {
        self.expect_word("bare")?;
        if *self.peek() == Tok::Dot {
            if let Tok::Word(m) = self.peek_at(1).clone() {
                let mode = match m.as_str() { "never" => Some(Bare::Never), "optional" => Some(Bare::Optional), "only" => Some(Bare::Only), _ => None };
                if let Some(mode) = mode { self.advance(); self.advance(); return Ok(mode); }
                return self.err(format!("no `bare.{m}`: a path is `bare.never` (required), `bare.optional` or `bare.only` (forbidden)"));
            }
        }
        self.err("say how other Limbs write the path: `bare.never` (required), `bare.optional` or `bare.only` (forbidden); a plain `bare` does not say")
    }

    /// Everything std brings is refused in a Limb whose `CCA.cplr` does not list it (owner, 2026-10-06:
    /// the operators, printing and the rest are std's, and a manifest lists std like any limb).
    fn std_missing<T>(&self, what: &str) -> PResult<T> {
        self.err(format!("{what} is std's, and this Limb's CCA.cplr does not list std: add `cca::Dep[id = \"{STD_ID}\", version = \"{STD_VERSION}\"]` to its `dependencies`"))
    }

    fn resolve(&self, segs: &[String]) -> Vec<String> {
        if segs.first().map(|s| s == "my").unwrap_or(false) { let mut out = self.limb_root.clone(); out.extend(segs[1..].iter().cloned()); return out; }
        let mut out = self.module.clone();
        out.extend(segs.iter().cloned());
        out
    }

    fn canonical(&self, segs: &[String]) -> String { self.resolve(segs).join(".") }

    /// A struct or enum name as written: this Limb's, or another Limb's `bare.optional` / `bare.only` one
    /// (owner, 2026-10-06); two other Limbs' are an error where written
    fn type_name(&self, segs: &[String]) -> PResult<String> {
        let own = self.canonical(segs);
        if segs.first().map_or(true, |s| s == "my") { return Ok(own); }
        let mine = self.limb_root.first().cloned();
        let hits: Vec<(String, String)> = BARE_TYPES.with(|t| t.borrow().iter().filter(|(rel, _, root, _)| rel.as_slice() == segs && Some(root) != mine.as_ref()).map(|(_, c, r, _)| (c.clone(), r.clone())).collect());
        match hits.as_slice() {
            [] => Ok(own),
            [(theirs, root)] => {
                BARE_TYPE_USES.with(|u| u.borrow_mut().push((own, theirs.clone(), segs.join("."), root.clone())));
                Ok(theirs.clone())
            }
            many => self.err(format!("`{}` could be any of {}: write the path", segs.join("."), many.iter().map(|(_, r)| format!("`{}::{}`", r.trim_start_matches("::"), segs.join("."))).collect::<Vec<_>>().join(", "))),
        }
    }

    /// `greet::Tally` where `greet` marks `Tally` `bare.only`: the path is forbidden (owner, 2026-10-06)
    fn path_forbidden(&self, canonical: &str) -> PResult<()> {
        if let Some((rel, _, root, _)) = BARE_TYPES.with(|t| t.borrow().iter().find(|(_, c, _, only)| *only && c == canonical).cloned()) {
            let l = root.trim_start_matches("::");
            return self.err(format!("`{l}::{}` is `bare.only`: `{l}` forbids its path, so write `{}`", rel.join("."), rel.join(".")));
        }
        Ok(())
    }

    fn peek(&self) -> &Tok { &self.toks[self.pos].tok }
    fn peek_at(&self, n: usize) -> &Tok { &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok }
    fn line(&self) -> usize { self.toks[self.pos].line }
    fn advance(&mut self) -> Token { let t = self.toks[self.pos].clone(); if self.pos < self.toks.len() - 1 { self.pos += 1; } t }

    fn err<T>(&self, msg: impl Into<String>) -> PResult<T> {
        let t = &self.toks[self.pos];
        Err(ParseError { line: t.line, col: t.col, msg: msg.into() })
    }

    fn expect(&mut self, want: Tok, what: &str) -> PResult<Token> {
        if *self.peek() == want { Ok(self.advance()) } else { self.err(format!("expected {what}, found {}", self.peek())) }
    }

    fn is_word(&self, w: &str) -> bool { matches!(self.peek(), Tok::Word(s) if s == w) }

    fn expect_word(&mut self, w: &str) -> PResult<()> {
        if self.is_word(w) { self.advance(); Ok(()) } else { self.err(format!("expected `{w}`, found {}", self.peek())) }
    }

    /// `CCA.cplr`: one value, `cca::Manifest[ … ];` and nothing else (owner, 2026-10-06). `None` when the
    /// file holds something else; the fields and the line of the value otherwise.
    pub fn manifest(&mut self) -> PResult<Option<(Vec<(String, Bracket)>, usize)>> {
        // `cca::Power.c_calls.functions["write"].except["x"]` is read whole, as text the manifest reader
        // takes apart (owner, 2026-10-07: powers narrow with `.` and name things in `[ ]`)
        let mut i = 0;
        while i + 2 < self.toks.len() {
            if matches!(&self.toks[i].tok, Tok::Word(w) if w == "cca") && self.toks[i + 1].tok == Tok::ColonColon && matches!(&self.toks[i + 2].tok, Tok::Word(w) if w == "Power") {
                let mut j = i + 3;
                let mut text = String::new();
                loop {
                    match (&self.toks[j].tok, self.toks.get(j + 1).map(|t| &t.tok)) {
                        (Tok::Dot, Some(Tok::Word(w))) => { if !text.is_empty() { text.push('.'); } text.push_str(w); j += 2; }
                        (Tok::LBracket, _) => {
                            j += 1;
                            let mut things = Vec::new();
                            loop {
                                match &self.toks[j].tok {
                                    Tok::Text(t, _) => { things.push(t.clone()); j += 1; }
                                    Tok::Comma => j += 1,
                                    Tok::RBracket => { j += 1; break; }
                                    other => { self.pos = j; return self.err(format!("a power names things as texts: `cca::Power.c_calls[\"write\"]`, found {other}")); }
                                }
                            }
                            text.push('[');
                            text.push_str(&things.iter().map(|t| format!("{t:?}")).collect::<Vec<_>>().join(","));
                            text.push(']');
                        }
                        _ => break,
                    }
                }
                let line = self.toks[i].line;
                self.toks.splice(i..j, [Token { tok: Tok::Text(format!("power:{text}"), format!("power:{text}")), line, col: 0, mtm: None }]);
            }
            i += 1;
        }
        let b = self.rhs()?;
        self.expect(Tok::Semi, "`;` after the manifest")?;
        if *self.peek() != Tok::Eof { return self.err("CCA.cplr holds one value, the `cca::Manifest[ … ];`, and nothing after it"); }
        match b.values.as_slice() {
            [Value::Expr(Expr::Construct(shape, fields, line))] if shape == "::cca.Manifest" => Ok(Some((fields.clone(), *line))),
            _ => Ok(None),
        }
    }

    // program := item* Eof
    // item := ('pub' ('.' 'limb')?)? (fn | struct | const | module) | MAIN block
    pub fn program(&mut self) -> PResult<Program> {
        let mut prog = Program { funcs: Vec::new(), structs: Vec::new(), enums: Vec::new(), interfaces: Vec::new(), claims: Vec::new(), consts: Vec::new(), externs: Vec::new(), modules: Vec::new(), main: None, main_args: None, strict_modules: Vec::new(), bare_marks: Vec::new() };
        // every quoted name, for finding C's variables once the program is read (owner, 2026-10-07)
        for t in &self.toks { if let Tok::Var(v) = &t.tok { VAR_SITES.with(|vs| vs.borrow_mut().push((self.limb_root.clone(), v.clone(), t.line))); } }
        // the root is `my.` (owner, 2026-10-06), no longer `limb.`; `pub.limb` stays
        if let Some(i) = (0..self.toks.len().saturating_sub(1)).find(|&i| matches!(&self.toks[i].tok, Tok::Word(w) if w == "limb") && self.toks[i + 1].tok == Tok::Dot && (i == 0 || self.toks[i - 1].tok != Tok::Dot)) {
            self.pos = i;
            return self.err("the root of this Limb is `my.` since 2026-10-06, no longer `limb.`: `my.helper[]`, `my.'pi'`");
        }
        // MTM's spellings (owner, 2026-10-06: other spellings, each standing for a main one)
        if self.lacks.iter().any(|c| c == "MTM") {
            for i in 0..self.toks.len() {
                let after_path = i > 0 && matches!(self.toks[i - 1].tok, Tok::Dot | Tok::ColonColon);
                let found: Option<(String, String)> = match (&self.toks[i].mtm, &self.toks[i].tok) {
                    (Some((w, m)), _) => Some((w.clone(), m.to_string())),
                    (None, Tok::Word(w)) if !after_path && rust_alias(w).is_some() => Some((w.clone(), rust_alias(w).unwrap().to_string())),
                    // `!std.i24` / `!std.u24`, Rust's way, read as `!`, `std`, `.`, the width
                    (None, Tok::Word(w)) if i > 2 && self.toks[i - 3].tok == Tok::Op("!".into()) && matches!(&self.toks[i - 2].tok, Tok::Word(s) if s == "std") && self.toks[i - 1].tok == Tok::Dot
                        && (w.starts_with('i') && !w.starts_with("int") || w.starts_with('u') && !w.starts_with("uint")) => {
                        Some((format!("!std.{w}"), format!("!std.{}", w.replacen('i', "int", 1).replacen('u', "uint", 1))))
                    }
                    _ => None,
                };
                if let Some((w, m)) = found {
                    self.pos = i;
                    return self.err(format!("`{w}` is MTM's spelling of `{m}`: write `{m}`, or add the core MTM to this Limb's `cores` (`{}`)", core_line("MTM")));
                }
            }
        }
        // Signatar's names are English letters, digits and `_`, bare or quoted (owner, 2026-10-06: "Bare
        // names should be English only", "English only too, crazy ones from the core"); WSTN allows every
        // script and emoji ("`b`"). Text values are not names.
        if self.lacks.iter().any(|c| c == "WSTN") {
            for i in 0..self.toks.len() {
                let (name, shown) = match &self.toks[i].tok { Tok::Word(w) => (w, w.clone()), Tok::Var(v) => (v, format!("'{v}'")), _ => continue };
                if name.is_ascii() { continue; }
                self.pos = i;
                return self.err(format!("`{shown}` is not an English name: Signatar's names are English letters, digits and `_`; a name in another script, or an emoji, needs the core WSTN in this Limb's `cores` (`{}`)", core_line("WSTN")));
            }
        }
        // the words a core brings (design/cores.md): ASP `to.` and `check`; IEG `nocheck` and
        // `NO_FUCKING_CHECKS`; Genny `interface` and `satisfies`; MCPS `concur`, `whichever`, `channel`
        // and `for para`. A word after `.` or `::` is a name in a path, not the keyword.
        if !self.lacks.is_empty() {
            for i in 0..self.toks.len() {
                let Tok::Word(w) = &self.toks[i].tok else { continue };
                if i > 0 && matches!(self.toks[i - 1].tok, Tok::Dot | Tok::ColonColon) { continue; }
                let core = match w.as_str() {
                    "to" if self.toks.get(i + 1).map(|t| &t.tok) == Some(&Tok::Dot) => "ASP",
                    "check" => "ASP",
                    "nocheck" | "NO_FUCKING_CHECKS" => "IEG",
                    "interface" | "satisfies" => "Genny",
                    "concur" | "whichever" | "channel" | "para" => "MCPS",
                    _ => continue,
                };
                if self.lacks.iter().any(|c| c == core) {
                    self.pos = i;
                    let what = if w == "to" { "`to.`".to_string() } else if w == "para" { "`for para`".to_string() } else if w == "nocheck" { "`!check`".to_string() } else { format!("`{w}`") };
                    return self.core_missing(core, &what);
                }
            }
        }
        // growing lists, maps and text operations are std's; fixed arrays and text literals are
        // Signatar's (owner, 2026-10-06)
        if self.no_std {
            for i in 0..self.toks.len() {
                let Tok::Word(w) = &self.toks[i].tok else { continue };
                if i > 0 && matches!(self.toks[i - 1].tok, Tok::Dot | Tok::ColonColon) { continue; }
                let next = self.toks.get(i + 1).map(|t| &t.tok);
                let what = match w.as_str() {
                    "list" if matches!(next, Some(Tok::Dot | Tok::LBracket)) => "a growing list (`list`)",
                    "map" if matches!(next, Some(Tok::Dot | Tok::LBracket)) => "a map (`map`)",
                    "str" if next == Some(&Tok::Dot) && matches!(self.toks.get(i + 2).map(|t| &t.tok), Some(Tok::Word(u)) if ["bytes", "points", "graphemes", "each"].contains(&u.as_str()) || STD_STR_OPS.contains(&u.as_str())) => "a text operation (`str.…`)",
                    _ => continue,
                };
                self.pos = i;
                return self.std_missing(what);
            }
        }
        if self.no_std {
            if let Some(i) = (0..self.toks.len().saturating_sub(1)).find(|&i| matches!(&self.toks[i].tok, Tok::Word(w) if w == "std") && self.toks[i + 1].tok == Tok::ColonColon) {
                self.pos = i;
                let what = match &self.toks.get(i + 2).map(|t| &t.tok) { Some(Tok::Word(w)) => format!("`std::{w}`"), _ => "`std::`".into() };
                return self.std_missing(&what);
            }
        }
        // `std::list.append[…]` is `list.append[…]`: std marks everything `bare.optional` (owner,
        // 2026-10-06), and its lists, maps and text units are the oracle's built-ins
        let mut i = 0;
        while i + 3 < self.toks.len() {
            let std_head = matches!(&self.toks[i].tok, Tok::Word(w) if w == "std") && self.toks[i + 1].tok == Tok::ColonColon
                && (i == 0 || self.toks[i - 1].tok != Tok::Dot)
                && match (&self.toks[i + 2].tok, &self.toks[i + 3].tok, self.toks.get(i + 4).map(|t| &t.tok)) {
                    (Tok::Word(h), Tok::Dot, _) if h == "list" || h == "map" => true,
                    (Tok::Word(h), Tok::Dot, Some(Tok::Word(u))) if h == "str" => ["bytes", "points", "graphemes", "each"].contains(&u.as_str()),
                    _ => false,
                };
            if std_head { self.toks.drain(i..i + 2); }
            i += 1;
        }
        // a `precedence` block applies to the whole file, wherever it is written: read it first
        self.scan_levels()?;
        self.items(&mut prog, 0)?;
        Ok(prog)
    }

    /// One branch of a top-level `on.target` chain, after its `{`: its items are read either way, and
    /// kept only if `keep`.
    fn on_branch(&mut self, prog: &mut Program, depth: usize, keep: bool) -> PResult<()> {
        let n = (prog.funcs.len(), prog.structs.len(), prog.enums.len(), prog.interfaces.len(), prog.claims.len(), prog.consts.len(), prog.externs.len(), prog.modules.len(), prog.strict_modules.len(), prog.bare_marks.len(), prog.main.is_some());
        self.items(prog, depth + 1)?;
        if !keep {
            prog.funcs.truncate(n.0); prog.structs.truncate(n.1); prog.enums.truncate(n.2); prog.interfaces.truncate(n.3); prog.claims.truncate(n.4);
            prog.consts.truncate(n.5); prog.externs.truncate(n.6); prog.modules.truncate(n.7); prog.strict_modules.truncate(n.8); prog.bare_marks.truncate(n.9);
            if !n.10 { prog.main = None; prog.main_args = None; }
        }
        Ok(())
    }

    /// Parse items until `}` (depth > 0) or end of file (depth 0).
    fn items(&mut self, prog: &mut Program, depth: usize) -> PResult<()> {
        loop {
            if depth > 0 && *self.peek() == Tok::RBrace { self.advance(); return Ok(()); }
            if *self.peek() == Tok::Eof {
                if depth > 0 { return self.err("unclosed `module {`"); }
                return Ok(());
            }
            if *self.peek() == Tok::Strict {
                let started = !(prog.funcs.is_empty() && prog.structs.is_empty() && prog.enums.is_empty() && prog.consts.is_empty() && prog.externs.is_empty() && prog.modules.is_empty() && prog.main.is_none() && prog.interfaces.is_empty());
                if depth > 0 || started || self.strict { return self.err("`#strict` goes once, at the top of the file, before anything else"); }
                self.advance();
                self.strict = true;
                prog.strict_modules.push(self.module.clone());
                continue;
            }
            // `bare.optional;` at the top of a file marks the file's module; `bare.never [fn, op];` in
            // `limb.cpls` marks kinds across the Limb (owner, 2026-10-06)
            if self.is_word("bare") {
                let line = self.line();
                let started = !(prog.funcs.is_empty() && prog.structs.is_empty() && prog.enums.is_empty() && prog.consts.is_empty() && prog.externs.is_empty() && prog.modules.is_empty() && prog.main.is_none() && prog.interfaces.is_empty());
                let mode = self.bare_mode()?;
                if *self.peek() == Tok::Semi {
                    self.advance();
                    if depth > 0 || started || prog.bare_marks.iter().any(|m| matches!(m.target, BareTarget::File(_))) {
                        return Err(ParseError { line, col: 0, msg: "a file's `bare` mark is its first line, before anything else; a block module takes `pub bare.… module name { … }`".into() });
                    }
                    prog.bare_marks.push(BareMark { target: BareTarget::File(self.module.clone()), mode, line, limb_root: self.limb_root.clone() });
                    continue;
                }
                if *self.peek() == Tok::LBracket {
                    if depth > 0 || self.module != self.limb_root { return self.err("a `bare` mark on kinds (`bare.never [fn, op];`) covers the whole Limb, so it is written in `limb.cpls`"); }
                    self.advance();
                    loop {
                        let kind = match self.advance().tok { Tok::Word(k) => k, other => return self.err(format!("expected a kind (`fn`, `op`, `struct`, `enum`, `interface`), found {other}")) };
                        if kind == "const" { return self.err("a const takes no `bare` mark: `'pi'` alone is a variable, so another Limb's const always has its path (owner, 2026-10-06)"); }
                        if !["fn", "op", "struct", "enum", "interface"].contains(&kind.as_str()) {
                            return self.err(format!("`{kind}` is not a kind a `bare` mark covers: `fn`, `op`, `struct`, `enum`, `interface` (a module is marked on itself)"));
                        }
                        prog.bare_marks.push(BareMark { target: BareTarget::Kind(kind), mode, line, limb_root: self.limb_root.clone() });
                        if *self.peek() == Tok::Comma { self.advance(); if *self.peek() == Tok::RBracket { break; } continue; }
                        break;
                    }
                    self.expect(Tok::RBracket, "`]` after the kinds")?;
                    self.expect(Tok::Semi, "`;` after the `bare` mark")?;
                    continue;
                }
                return self.err("a `bare` mark is `bare.optional;` at the top of a file, `bare.optional [fn, op];` in `limb.cpls`, or `pub bare.optional fn …` on an item");
            }
            // `on.target … { items } else { items }` at the top level: only the chosen branch's items exist
            // (owner, 2026-10-06: a compile-time choice around any code)
            if self.is_word("on") {
                let mut chosen = false;
                loop {
                    let (machine, held) = self.on_test()?;
                    if machine { return self.err("`on.machine` chooses when running, so it goes inside a body; around items, `on.target`"); }
                    self.expect(Tok::LBrace, "`{` after the `on.target` test")?;
                    let keep = held && !chosen;
                    self.on_branch(prog, depth, keep)?;
                    chosen |= keep;
                    if !self.is_word("else") { break; }
                    self.advance();
                    if self.is_word("on") { continue; }
                    self.expect(Tok::LBrace, "`{` after `else`")?;
                    self.on_branch(prog, depth, !chosen)?;
                    break;
                }
                continue;
            }
            let mut pending_bare: Option<(Bare, usize)> = None;
            let before = (prog.funcs.len(), prog.structs.len(), prog.enums.len(), prog.consts.len(), prog.interfaces.len(), prog.modules.len());
            // visibility (decided 2026-09-19): nothing, `pub.limb`, `pub`
            let mut vis = Vis::Private;
            if self.is_word("pub") {
                self.advance();
                vis = Vis::Pub;
                if *self.peek() == Tok::Dot {
                    self.advance();
                    match self.advance().tok {
                        Tok::Word(w) if w == "limb" => vis = Vis::Limb,
                        Tok::Word(w) if w == "bare" => return self.err("`bare` is its own word after `pub`, not a narrowing: `pub bare.optional fn …`"),
                        other => return self.err(format!("the only narrowing of `pub` is `pub.limb` (visible in this Limb only), found {other}")),
                    }
                }
                // `pub fn area = shapes.circle.area;` — a shorter name for other Limbs (owner,
                // 2026-09-23). The oracle runs one Limb, where it has no outside to serve.
                if matches!(self.peek(), Tok::Word(k) if ["fn", "struct", "enum", "interface", "const", "module"].contains(&k.as_str()))
                    && matches!(self.peek_at(1), Tok::Word(_) | Tok::Var(_)) && *self.peek_at(2) == Tok::Eq {
                    return self.err("`pub fn area = shapes.circle.area;` gives other Limbs a shorter name; the oracle runs one Limb, so it is not in the oracle yet");
                }
                // `pub bare.optional fn …`: how other Limbs write the path to this item (owner, 2026-10-06)
                if self.is_word("bare") {
                    let line = self.line();
                    if vis == Vis::Limb { return self.err("a `bare` mark says how other Limbs write the path, and a `pub.limb` thing is never reached from them"); }
                    pending_bare = Some((self.bare_mode()?, line));
                }
                match self.peek().clone() {
                    Tok::Word(w) if w == "fn" || w == "struct" || w == "const" || w == "module" || w == "extern" || w == "layout" || w == "enum" || w == "op" || w == "interface" => {}
                    Tok::Word(w) if w == "let" => return self.err("a Limb-level value is a `const`, not a `let`: `pub const 'x' := 1;`"),
                    Tok::Var(_) => return self.err("a Limb-level value is a `const`, not a variable: `pub const 'x' := 1;`"),
                    Tok::Word(w) if w == "MAIN" => return self.err("MAIN is the program's entry, never `pub`"),
                    other => return self.err(format!("`pub` goes before `fn`, `struct`, `const`, or `module`, found {other}")),
                }
            }
            match self.peek().clone() {
                // `test fn name [] { … }` (owner, 2026-09-29): run by `cca test`, never called
                Tok::Word(w) if w == "test" && matches!(self.peek_at(1), Tok::Word(f) if f == "fn") => {
                    if vis != Vis::Private { return self.err("a `test fn` is never `pub`: nothing calls it, `cca test` runs it"); }
                    self.advance();
                    self.in_test = true;
                    let f = self.func();
                    self.in_test = false;
                    let mut f = f?;
                    if !f.params.is_empty() { return Err(ParseError { line: f.line, col: 0, msg: format!("`test fn {}` takes no parameters: `test fn {} [] {{ … }}`", f.name, f.name) }); }
                    if f.ret.is_some() || f.ret_lent.is_some() || !f.fails.is_empty() { return Err(ParseError { line: f.line, col: 0, msg: format!("`test fn {}` returns nothing and cannot fail; it passes unless a `must` does not hold", f.name) }); }
                    f.is_test = true;
                    prog.funcs.push(f)
                }
                Tok::Word(w) if w == "fn" => { let mut f = self.func()?; f.vis = vis; prog.funcs.push(f) }
                Tok::Word(w) if w == "struct" => { let mut d = self.struct_def()?; d.vis = vis; prog.structs.push(d) }
                Tok::Word(w) if w == "enum" => {
                    // enum Name { case  case: type … }   (design/enums.md, 2026-09-20)
                    let line = self.line();
                    self.advance();
                    let name = match self.advance().tok {
                        Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot name an enum")),
                        Tok::Word(n) if rust_alias(&n).is_some() => return self.err(format!("`{n}` is another name for `{}`, so it cannot name an enum", rust_alias(&n).unwrap())),
                        Tok::Word(n) => n,
                        Tok::Var(v) => return self.err(format!("an enum's name is bare — it is a type: `enum {v}`")),
                        other => return self.err(format!("expected an enum name, found {other}")),
                    };
                    let gens = self.announce("enum")?;
                    self.expect(Tok::LBrace, "`{`")?;
                    let mut cases: Vec<FailCase> = Vec::new();
                    while *self.peek() != Tok::RBrace {
                        let cname = match self.advance().tok {
                            Tok::Word(c) if KEYWORDS.contains(&c.as_str()) => return self.err(format!("`{c}` is a keyword and cannot name a case")),
                            Tok::Word(c) => c,
                            Tok::Var(v) => return self.err(format!("a case's name is bare, like a field: `{v}` — it is a path into the type")),
                            Tok::Eof => return self.err("unclosed `enum {`"),
                            other => return self.err(format!("expected a case name, found {other}")),
                        };
                        if cases.iter().any(|c| c.name == cname) { return self.err(format!("case `{cname}` is declared twice")); }
                        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
                        if *self.peek() == Tok::Semi { return self.err("no `;` after a case"); }
                        if *self.peek() == Tok::Comma { return self.err("no `,` between cases — one per line, like struct fields"); }
                        cases.push(FailCase { name: cname, ty });
                    }
                    self.advance();
                    if cases.is_empty() { return self.err(format!("enum `{name}` has no cases")); }
                    let params: Vec<String> = gens.iter().map(|(g, _)| g.clone()).collect();
                    let bounds = Self::flat_bounds(&gens);
                    self.declared = None;
                    prog.enums.push(EnumDef { name: self.canonical(&[name]), module: self.module.clone(), vis, cases, params, bounds, c_values: None, line });
                }
                Tok::Word(w) if w == "extern" => {
                    // extern.C fn name [params] -> T;  or a block of them, `extern.C { … }`  (design/ffi.md;
                    // the block and the `.C`, owner, 2026-10-02: "both", "`extern.C`, and one line too")
                    self.advance();
                    if *self.peek() != Tok::Dot {
                        return self.err("`extern` names its language, as `layout.C` does: `extern.C fn write [...] -> int64;`, `extern.C 'errno': int32;`, or a block, `extern.C { … }`");
                    }
                    self.advance();
                    match self.advance().tok {
                        Tok::Word(l) if l == "C" => {}
                        Tok::Word(l) if l == "c" => return self.err("C is written capitalized here, as every language's name is: `extern.C`"),
                        other => return self.err(format!("the only language `extern` reaches is C: `extern.C`, found {other}")),
                    }
                    if *self.peek() == Tok::LBrace {
                        self.advance();
                        while *self.peek() != Tok::RBrace {
                            if *self.peek() == Tok::Eof { return self.err("an `extern.C { … }` block is missing its `}`"); }
                            // a line may say its own `pub` / `pub.limb`; a `pub` written on the block
                            // overrides every line's (owner, 2026-10-02: "allow mixing, but block overrides")
                            let mut line_vis = Vis::Private;
                            if self.is_word("pub") {
                                self.advance();
                                line_vis = Vis::Pub;
                                if *self.peek() == Tok::Dot {
                                    self.advance();
                                    match self.advance().tok {
                                        Tok::Word(w) if w == "limb" => line_vis = Vis::Limb,
                                        other => return self.err(format!("the only narrowing of `pub` is `pub.limb` (visible in this Limb only), found {other}")),
                                    }
                                }
                            }
                            let item_vis = if vis != Vis::Private { vis } else { line_vis };
                            if self.is_word("extern") { return self.err("inside `extern.C { … }`, each line starts with `fn` or a quoted name, without `extern.C`"); }
                            if !(self.is_word("fn") || self.is_word("mut") || self.is_word("immut") || self.is_word("let") || matches!(self.peek(), Tok::Var(_))) {
                                return self.err(format!("an `extern.C {{ … }}` block holds `fn` lines and quoted names, found {}: is its `}}` missing?", self.peek()));
                            }
                            self.extern_item(prog, item_vis)?;
                        }
                        self.advance();
                        continue;
                    }
                    self.extern_item(prog, vis)?;
                }
                Tok::Word(w) if w == "layout" => {
                    // layout.C struct Name { … }   (design/ffi.md)
                    self.advance();
                    self.expect(Tok::Dot, "`.C` after `layout` (`layout.C struct …`)")?;
                    match self.advance().tok {
                        Tok::Word(l) if l == "C" => {},
                        // a language's name is capitalized, as `inline.ASM` (owner, 2026-10-02: "BIG")
                        Tok::Word(l) if l == "c" => return self.err("C is written capitalized here, as every language's name is: `layout.C`".to_string()),
                        other => return self.err(format!("the only layout is `layout.C`, found {other}")),
                    }
                    if self.is_word("enum") {
                        // layout.C enum Key { a = 65  b = 66 }  — C's enum: a number per case, no data,
                        // every number written (owner, 2026-09-27: a)
                        let line = self.line();
                        self.advance();
                        let name = match self.advance().tok {
                            Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot name an enum")),
                            Tok::Word(n) => n,
                            other => return self.err(format!("expected an enum name, found {other}")),
                        };
                        self.expect(Tok::LBrace, "`{`")?;
                        let (mut cases, mut values): (Vec<FailCase>, Vec<i128>) = (Vec::new(), Vec::new());
                        while *self.peek() != Tok::RBrace {
                            let cname = match self.advance().tok {
                                Tok::Word(c) if KEYWORDS.contains(&c.as_str()) => return self.err(format!("`{c}` is a keyword and cannot name a case")),
                                Tok::Word(c) => c,
                                Tok::Eof => return self.err("unclosed `layout.C enum {`"),
                                other => return self.err(format!("expected a case name, found {other}")),
                            };
                            if *self.peek() == Tok::Colon { return self.err(format!("a C enum's case carries no data; C's enum is a number: `{cname} = 1`")); }
                            if *self.peek() != Tok::Eq { return self.err(format!("a C enum's case says its number, as the C header does: `{cname} = 1`")); }
                            self.advance();
                            let neg = matches!(self.peek(), Tok::Op(o) if o == "-");
                            if neg { self.advance(); }
                            let v = match self.advance().tok { Tok::Int(n) => if neg { -n } else { n }, other => return self.err(format!("a C enum's number is a whole number, found {other}")) };
                            if v < i32::MIN as i128 || v > i32::MAX as i128 { return self.err(format!("{v} does not fit C's enum (an `int`, 32 bits)")); }
                            if cases.iter().any(|c| c.name == cname) { return self.err(format!("case `{cname}` is declared twice")); }
                            if let Some(i) = values.iter().position(|x| *x == v) { return self.err(format!("`{cname}` and `{}` are both {v}; each case has its own number", cases[i].name)); }
                            cases.push(FailCase { name: cname, ty: None });
                            values.push(v);
                        }
                        self.advance();
                        if cases.is_empty() { return self.err(format!("enum `{name}` has no cases")); }
                        prog.enums.push(EnumDef { name: self.canonical(&[name]), module: self.module.clone(), vis, cases, params: Vec::new(), bounds: Vec::new(), c_values: Some(values), line });
                        continue;
                    }
                    if !self.is_word("struct") { return self.err("`layout.C` goes before `struct` or `enum`: `layout.C struct Timeval { … }`, `layout.C enum Key { a = 65 }`"); }
                    self.at_boundary = true;
                    let d = self.struct_def();
                    self.at_boundary = false;
                    let mut d = d?;
                    d.vis = vis; d.layout_c = true;
                    prog.structs.push(d);
                }
                Tok::Word(w) if w == "asm" || w == "ASM" || w == "inline" => return self.err("an `inline.Assembly` block is a statement; it goes inside a body, under `!check`"),
                Tok::Word(w) if w == "const" => { let mut c = self.const_def()?; c.vis = vis; prog.consts.push(c) }
                Tok::Word(w) if w == "module" => {
                    let line = self.line();
                    self.advance();
                    let name = match self.advance().tok {
                        Tok::Word(w) => w,
                        Tok::Var(v) => return self.err(format!("a module's name is bare: `module {v}`")),
                        other => return self.err(format!("expected a module name, found {other}")),
                    };
                    if let Err(m) = module_name_ok(&name) { return self.err(format!("`{name}` cannot name a module: {m}")); }
                    if *self.peek() == Tok::Semi { return self.err(format!("a module is a file (`{name}.cpls`) or a block (`module {name} {{ … }}`); there is no `module {name};` include line")); }
                    self.expect(Tok::LBrace, "`{` to open the module")?;
                    self.module.push(name);
                    prog.modules.push(ModuleDef { path: self.module.clone(), vis, line });
                    if self.strict { prog.strict_modules.push(self.module.clone()); }
                    self.items(prog, depth + 1)?;
                    self.module.pop();
                }
                Tok::Word(w) if w == "MAIN" => {
                    if self.module != self.limb_root { return self.err("MAIN lives at the root of the Limb (`limb.cpls`, outside any module)"); }
                    if prog.main.is_some() { return self.err("a program has exactly one MAIN"); }
                    self.advance();
                    // the command line, as C hands it over: a count and the words (owner, 2026-09-29:
                    // "low-level shit, decided"); anything friendlier is written in CyborgPL on top
                    if *self.peek() == Tok::LBracket {
                        self.advance();
                        let mut names = Vec::new();
                        let want = ["int32", "ptr[ptr[uint8]]"];
                        for (k, w) in want.iter().enumerate() {
                            let name = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("MAIN's parameters are C's: `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]]`; found {other}")) };
                            self.expect(Tok::Colon, "`:` and the type")?;
                            self.at_boundary = true;
                            let t = self.ty();
                            self.at_boundary = false;
                            let t = t?;
                            if ty_name(&t) != *w { return self.err(format!("MAIN's parameters are C's: '{name}' is `{w}`, not `{}`", ty_name(&t))); }
                            names.push(name);
                            if k == 0 { self.expect(Tok::Comma, "`,` and the second parameter, `'argv': ptr[ptr[uint8]]`")?; }
                        }
                        self.expect(Tok::RBracket, "`]` after MAIN's two parameters")?;
                        prog.main_args = Some((names[0].clone(), names[1].clone()));
                    }
                    prog.main = Some(self.block()?);
                }
                Tok::Word(w) if w == "op" => { let mut f = self.op_decl()?; f.vis = vis; prog.funcs.push(f); }
                // read already, before the items (`program`); passed over here
                Tok::Word(w) if w == "precedence" => { self.precedence_block()?; }
                Tok::Word(w) if w == "interface" => {
                    // interface Name { fn …;  op …; }   (design/interfaces.md, 2026-09-20)
                    let line = self.line();
                    self.advance();
                    let name = match self.advance().tok {
                        Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot name an interface")),
                        Tok::Word(n) if rust_alias(&n).is_some() => return self.err(format!("`{n}` is another name for `{}`, so it cannot name an interface", rust_alias(&n).unwrap())),
                        Tok::Word(n) if n == "any" => return self.err("`any` is the built-in \"any type\" (owner, 2026-10-02); an interface needs another name"),
                        Tok::Word(n) => n,
                        Tok::Var(v) => return self.err(format!("an interface's name is bare: `interface {v}`")),
                        other => return self.err(format!("expected an interface name, found {other}")),
                    };
                    // `interface Ordered <T> { … }` — its `<T>` announced, as every generic is (owner, 2026-10-02)
                    self.announce("interface")?;
                    self.expect(Tok::LBrace, "`{`")?;
                    let mut entries = Vec::new();
                    // a list of types, one per line (owner, 2026-10-02: "just let it fucking go down"),
                    // beside or instead of the operations
                    let mut types: Vec<Ty> = Vec::new();
                    let mut last_line = 0;
                    self.in_interface = true;
                    while *self.peek() != Tok::RBrace {
                        let r = match self.peek().clone() {
                            Tok::Word(w) if w == "fn" => self.func().map(Some),
                            Tok::Word(w) if w == "op" => self.op_decl().map(Some),
                            Tok::Eof => Err(ParseError { line: self.line(), col: 0, msg: "unclosed `interface {`".into() }),
                            Tok::Comma => Err(ParseError { line: self.line(), col: 0, msg: "no `,` between the types: one per line".into() }),
                            _ => {
                                let tline = self.line();
                                if tline == last_line { Err(ParseError { line: tline, col: 0, msg: "an interface's types go one per line".into() }) }
                                else {
                                    last_line = tline;
                                    match self.ty() {
                                        Ok(Ty::Var(v)) => Err(ParseError { line: tline, col: 0, msg: format!("`<{v}>` is the type being checked; list real types, or promise operations about it") }),
                                        Ok(t) => { types.push(t); Ok(None) }
                                        Err(e) => Err(e),
                                    }
                                }
                            }
                        };
                        match r { Ok(Some(e)) => entries.push(e), Ok(None) => {}, Err(e) => { self.in_interface = false; self.declared = None; return Err(e); } }
                    }
                    self.in_interface = false;
                    self.declared = None;
                    self.advance();
                    if entries.is_empty() && types.is_empty() { return self.err(format!("interface `{name}` promises nothing")); }
                    prog.interfaces.push(InterfaceDef { name: self.canonical(&[name]), module: self.module.clone(), vis, entries, types, line });
                }
                Tok::Word(_) if matches!(self.peek_at(1), Tok::Word(s) if s == "satisfies") || (matches!(self.peek_at(1), Tok::Dot) && false) => {
                    // Vec satisfies Ordered;   — an optional, verified claim
                    let line = self.line();
                    let ty = self.ty()?;
                    self.expect_word("satisfies")?;
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), other => return self.err(format!("expected an interface's name after `satisfies`, found {other}")) }
                        if *self.peek() == Tok::Dot { self.advance(); continue; }
                        break;
                    }
                    self.expect(Tok::Semi, "`;` after the claim")?;
                    prog.claims.push(Claim { ty, iface: self.type_name(&segs)?, line });
                }
                Tok::Word(w) if w == "func" => return self.err("`func` became `fn` (2026-09-19): `fn add ['a': int32] -> int32 { … }`"),
                Tok::Word(w) if w == "let" || w == "var" => return self.err("a variable cannot sit at the top level; it goes inside MAIN or a fn — a Limb-level value is a `const`: `const 'x' := 1;`"),
                Tok::Var(_) => return self.err("a variable cannot sit at the top level; it goes inside MAIN or a fn — a Limb-level value is a `const`: `const 'x' := 1;`"),
                Tok::Word(w) if w == "mod" => return self.err("`mod` is the remainder operator; a module block is `module name { … }`"),
                Tok::Word(w) if w == "check" || w == "nocheck" || w == "NO_FUCKING_CHECKS" => return self.err(format!("`{w}` goes in front of a statement inside a body; there is no file-wide or Limb-wide tier")),
                Tok::RBrace => return self.err("`}` with nothing open"),
                other => return self.err(format!("expected `fn`, `struct`, `const`, `module`, or `MAIN` at the top level, found {other}")),
            }
            // the item a `pub bare.…` mark was written on (owner, 2026-10-06)
            if let Some((mode, line)) = pending_bare {
                let target = if prog.modules.len() > before.5 && prog.modules[before.5].line == line {
                    BareTarget::Block(prog.modules[before.5].path.clone())
                } else if prog.funcs.len() > before.0 {
                    let f = &prog.funcs[before.0];
                    // a glyph operator has nowhere to write a path (2026-10-02), so nothing to mark
                    if f.is_op && !f.name.rsplit('.').next().unwrap_or("").chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        return Err(ParseError { line, col: 0, msg: format!("`{}` is a glyph operator, which has nowhere to write a path, so there is nothing for `bare` to say", f.name.rsplit('.').next().unwrap_or("")) });
                    }
                    BareTarget::Item(if f.is_op { "op".into() } else { "fn".into() }, f.module.iter().cloned().chain(std::iter::once(f.name.clone())).collect::<Vec<_>>().join("."))
                } else if prog.structs.len() > before.1 { BareTarget::Item("struct".into(), prog.structs[before.1].name.clone()) }
                else if prog.enums.len() > before.2 { BareTarget::Item("enum".into(), prog.enums[before.2].name.clone()) }
                else if prog.consts.len() > before.3 { return Err(ParseError { line, col: 0, msg: "a const takes no `bare` mark: `'pi'` alone is a variable, so another Limb's const always has its path (owner, 2026-10-06)".into() }); }
                else if prog.interfaces.len() > before.4 { BareTarget::Item("interface".into(), prog.interfaces[before.4].name.clone()) }
                else { return Err(ParseError { line, col: 0, msg: "a `bare` mark goes on a `fn`, `op`, `struct`, `enum`, `interface` or `module`".into() }); };
                prog.bare_marks.push(BareMark { target, mode, line, limb_root: self.limb_root.clone() });
            }
        }
    }

    // const := 'const' Var (':' type)? (':=' | '=') rhs ';'   — top level only (decided 2026-09-19)
    fn const_def(&mut self) -> PResult<ConstDef> {
        let line = self.line();
        self.expect_word("const")?;
        if self.is_word("mut") || self.is_word("immut") { return self.err("a `const` is neither `mut` nor `!mut`; it is a constant"); }
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            Tok::Word(w) => return self.err(format!("a const is named like a variable, quoted: `const '{w}'`")),
            other => return self.err(format!("expected a quoted const name, found {other}")),
        };
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
        match (self.peek().clone(), &ty) {
            (Tok::Walrus, None) | (Tok::Eq, Some(_)) => { self.advance(); }
            (Tok::Walrus, Some(_)) => return self.err("`:=` means Precog chooses the type; with an explicit type write `=`"),
            (Tok::Eq, None) => return self.err("`=` means you said everything; with no type write `:=`"),
            (Tok::ShadowWalrus | Tok::ShadowEq, _) => return self.err("a const cannot shadow anything; there is nothing above the top level"),
            (other, _) => return self.err(format!("expected `:=` or `=` after '{name}', found {other}")),
        }
        let value = self.rhs()?;
        self.expect(Tok::Semi, "`;`")?;
        Ok(ConstDef { name, module: self.module.clone(), vis: Vis::Private, ty, value, line })
    }


    // op NAME ['a': T, 'b': T] -> R { … }   (decided 2026-09-20)
    // The language's own operators may be defined for a user type (2026-09-20), never
    // for the language's types; an interface entry may name them for a `<T>`.
    fn op_decl(&mut self) -> PResult<Func> {
        let line = self.line();
        self.bounds.clear();
        self.expect_word("op")?;
        let name = match self.advance().tok {
            Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot be an operator")),
            Tok::Word(n) if n == "not" => return self.err("`not` is the language's; it cannot be redefined"),
            // a type's own `plus` / `minus` / `times` (owner, 2026-10-03)
            Tok::Word(n) if n == "plus" => "+".into(),
            Tok::Word(n) if n == "minus" => "-".into(),
            Tok::Word(n) if n == "times" => "x".into(),
            Tok::Word(n) if (n == "x" || n == "xx") && !self.internal_ops => { let now = if n == "x" { "times" } else { "pow" }; return self.err(format!("`op {n}` is `op {now}` now (owner, 2026-10-03)")); }
            Tok::Op(o) if matches!(o.as_str(), "+" | "-") && !self.internal_ops => { let now = if o == "+" { "plus" } else { "minus" }; return self.err(format!("`op {o}` is `op {now}` now (owner, 2026-10-03)")); }
            // a narrowed form, `op dot.fast […]` (owner, 2026-10-02/03)
            Tok::Word(n) if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(_)) => { self.advance(); let Tok::Word(b) = self.advance().tok else { unreachable!() }; format!("{n}.{b}") }
            Tok::Word(n) => n,
            Tok::Op(o) => o,
            Tok::Var(v) => return self.err(format!("an operator's name is bare — a word or a run of operator characters, not `'{v}'`")),
            Tok::Arrow | Tok::LeftArrow | Tok::BothArrow => return self.err("an arrow cannot name an operator: `-->`, `->`, `>>`, `>>>`, `<--`, `<-`, `<<`, `<<<`, and `<-->`, `<->`, `<<>>`, `<<<>>>`, `<---->`, `<--|-->`, `<-|->`, `<<|>>`, `<<<|>>>` are arrows (owner, 2026-10-06)"),
            other => return self.err(format!("expected an operator name after `op`, found {other}")),
        };
        let gens = if self.in_interface { Vec::new() } else { self.announce("op")? };
        self.expect(Tok::LBracket, "`[` to open the two operands")?;
        let mut params = Vec::new();
        loop {
            let pline = self.line();
            let kind = match self.peek().clone() {
                Tok::Word(w) if w == "give" => { self.advance(); ParamKind::Owned }
                Tok::Word(w) if w == "lent" || w == "lend" => return self.err("an operator's operands are lent by default; write just `'a': T` (or `give 'a': T` to take it by value)"),
                Tok::Word(w) if w == "mut" || w == "immut" || w == "copy" => return self.err(format!("`{w}` has no meaning on an operand; an operand is lent, or `give`")),
                Tok::Ellipsis => return self.err("an operator takes exactly two operands; no `...`"),
                _ => ParamKind::Lent,
            };
            let pname = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a quoted operand name, found {other}")) };
            let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
            params.push(Param { kind, mutability: None, ty, name: pname, variadic: false, life: None, line: pline });
            if *self.peek() == Tok::Comma { self.advance(); continue; }
            break;
        }
        self.expect(Tok::RBracket, "`]`")?;
        if params.len() != 2 { return self.err(format!("an operator takes exactly two operands, `op {name}` has {}", params.len())); }
        let builtin = (Parser::base_level(&name) != 0 && name != "pow" && name != "pow.wrap" && name != "pow.ieee") || OP_WORDS.contains(&name.as_str());
        if builtin && !self.in_interface {
            let user_type = matches!(params[0].ty, Some(Ty::Struct(_)) | Some(Ty::Inst(..)));
            if !user_type { return self.err(format!("`{name}` is the language's operator; it can be defined for a struct or enum of yours, never changed for the language's own types")); }
        }
        let ret = if *self.peek() == Tok::Arrow { self.advance(); Some(self.ty()?) } else { return self.err("an operator produces a value: `-> T` is required"); };
        if self.is_word("or") { return self.err("an operator cannot fail; make it a `fn` if it can"); }
        let body = if self.in_interface {
            if *self.peek() == Tok::LBrace { return self.err("an interface lists promises, not code: end the signature with `;`"); }
            self.expect(Tok::Semi, "`;` to end the promise")?;
            Vec::new()
        } else { self.block()? };
        let bounds = Self::flat_bounds(&gens);
        if !self.in_interface { self.declared = None; self.groups.clear(); }
        Ok(Func { name, module: self.module.clone(), vis: Vis::Private, ret, ret_lent: None, fails: Vec::new(), is_extern: false, is_op: true, is_test: false, bounds, params, body, line })
    }

    // struct := 'struct' Word '{' (('mut' | 'immut')? Word ':' type)* '}'   — no `;`, no `let`
    // Field names are bare (decided 2026-09-19): a field is a path into the struct, not a
    // variable you hold; the quotes go on the whole (`'l'`), the route into it is bare.
    fn struct_def(&mut self) -> PResult<StructDef> {
        let line = self.line();
        self.expect_word("struct")?;
        let name = match self.advance().tok {
            Tok::Word(w) => w,
            Tok::Var(v) => return self.err(format!("a struct's name is bare — it is a type: `struct {v}`")),
            other => return self.err(format!("expected a struct name, found {other}")),
        };
        if KEYWORDS.contains(&name.as_str()) { return self.err(format!("`{name}` is a keyword and cannot name a struct")); }
        if let Some(own) = rust_alias(&name) { return self.err(format!("`{name}` is another name for `{own}`, so it cannot name a struct")); }
        let gens = self.announce("struct")?;
        self.expect(Tok::LBrace, "`{`")?;
        let mut fields = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof { return self.err("unclosed `struct {`"); }
            if self.is_word("let") || self.is_word("var") { return self.err("a struct field has no `let`: a struct always owns its fields — write `number: int32`"); }
            let mut immut = false;
            if self.is_word("immut") { self.advance(); immut = true; }
            else if self.is_word("mut") { self.advance(); }
            if *self.peek() == Tok::Dot { return self.err("`!mut` is a bare word before the field's name: `!mut number: int32`"); }
            let fname = match self.advance().tok {
                Tok::Word(w) if KEYWORDS.contains(&w.as_str()) => return self.err(format!("`{w}` is a keyword and cannot name a field")),
                Tok::Word(w) => w,
                Tok::Var(v) => return self.err(format!("a field's name is bare: `{v}: int32` — a field is a path into the struct, not a variable (decided 2026-09-19)")),
                other => return self.err(format!("expected a field name, found {other}")),
            };
            if let Tok::Var(v) = self.peek().clone() { return self.err(format!("a field is `name: type` — the name first, bare: `{v}: {fname}`")); }
            if *self.peek() != Tok::Colon { return self.err(format!("a field's type follows its name: `{fname}: int32`")); }
            self.advance();
            let ty = self.ty()?;
            if fields.iter().any(|f: &Field| f.name == fname) { return self.err(format!("field '{fname}' is declared twice")); }
            if *self.peek() == Tok::Semi { return self.err("no `;` after a struct field — fields are not statements"); }
            if *self.peek() == Tok::Comma { return self.err("no `,` between struct fields — one per line"); }
            fields.push(Field { ty, name: fname, immut });
        }
        self.advance();
        if fields.is_empty() { return self.err(format!("struct `{name}` has no fields")); }
        // the blanks' order is the announcement's (owner, 2026-10-02), no longer first appearance
        let params: Vec<String> = gens.iter().map(|(g, _)| g.clone()).collect();
        let bounds = Self::flat_bounds(&gens);
        self.declared = None;
        Ok(StructDef { name: self.canonical(&[name]), module: self.module.clone(), vis: Vis::Private, layout_c: false, params, bounds, fields, line })
    }

    // fn := 'fn' Word '[' params ']' ('->' type)? block
    fn func(&mut self) -> PResult<Func> {
        let line = self.line();
        self.bounds.clear();
        self.expect_word("fn")?;
        if *self.peek() == Tok::Dot { return self.err("`fn` takes no chain; the return type goes after the parameters: `fn add [...] -> int32`"); }
        let name = match self.advance().tok {
            Tok::Word(w) => w,
            Tok::Var(v) => return self.err(format!("function names are bare: `fn {v}`, not `fn '{v}'` (quotes mean variable)")),
            other => return self.err(format!("expected a function name, found {other}")),
        };
        // A keyword may name a function (decided 2026-09-19) — it is then never called
        // bare, only through a path: `my.to[…]` inside, `std::to[…]` outside.
        if name == "MAIN" { return self.err("MAIN is the program's block, not a function: `MAIN { … }`"); }
        let gens = if self.in_interface { Vec::new() } else { self.announce("fn")? };
        if *self.peek() == Tok::LParen { return self.err("parameters sit in `[ ]`; `( )` is arithmetic grouping"); }
        self.expect(Tok::LBracket, "`[` to open the parameter list")?;
        let mut params = Vec::new();
        if *self.peek() != Tok::RBracket {
            loop {
                params.push(self.param()?);
                if *self.peek() == Tok::Comma { self.advance(); continue; }
                break;
            }
        }
        self.expect(Tok::RBracket, "`]` to close the parameter list")?;
        // -> T | -> T or fail[…] | -> fail[…]   (decided 2026-09-19)
        let mut ret = None;
        let mut ret_lent = None;
        let mut fails = Vec::new();
        match self.peek() {
            Tok::Arrow => {
                self.advance();
                // `-> lent@x str.utf8` — hands back a borrow (owner, 2026-09-29)
                if self.is_word("lent") {
                    self.advance();
                    let m = if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(w) if w == "mut") { self.advance(); self.advance(); true } else { false };
                    ret_lent = Some((m, self.life_label()?));
                } else if let Some(at) = self.amp() {
                    // `-> &T`, `-> &mut T`, `-> &@x T` — `&` wherever `lent` goes (owner, 2026-10-02: "Yes.")
                    let (m, life) = self.amp_rest(at)?;
                    ret_lent = Some((m, life));
                }
                if self.is_word("fail") { fails = self.fail_list()?; }
                else {
                    let saved = self.at_boundary; self.at_boundary = true;
                    let r = self.ty();
                    self.at_boundary = saved;
                    ret = Some(r?);
                    if self.is_word("or") {
                        self.advance();
                        if !self.is_word("fail") { return self.err("after `or` comes `fail[…]`: `-> bool or fail[full, bad_item: str.utf8]`"); }
                        fails = self.fail_list()?;
                    }
                }
            }
            Tok::Colon => return self.err("a return type is written `-> int32`, not `: int32` (`:` attaches a type to a name)"),
            Tok::Word(w) if w == "or" || w == "fail" => return self.err("a function that can fail says so after `->`: `-> bool or fail[full]` or `-> fail[full]`"),
            Tok::Word(_) => return self.err("a return type is written `-> int32` after the parameter list"),
            _ => {}
        }
        if self.strict {
            if let Some(p) = params.iter().find(|p| p.ty.is_none()) { return self.err(format!("`#strict`: parameter '{}' says its type", p.name)); }
        }
        self.saw_value_return = false;
        let body = if self.at_boundary {
            if *self.peek() == Tok::LBrace { return self.err("an `extern.C fn` has no body — it lives somewhere else; end it with `;`"); }
            if !fails.is_empty() { return self.err("an `extern.C fn` cannot `fail`; C has no failures — a wrapper turns its return value into one"); }
            self.expect(Tok::Semi, "`;` to end the extern declaration")?;
            Vec::new()
        } else if self.in_interface {
            if *self.peek() == Tok::LBrace { return self.err("an interface lists promises, not code: end the signature with `;`"); }
            self.expect(Tok::Semi, "`;` to end the promise")?;
            Vec::new()
        } else { self.block()? };
        if self.strict && ret.is_none() && self.saw_value_return { return self.err(format!("`#strict`: `{name}` returns a value, so it says its type: `-> int32`")); }
        // a returned borrow names its group in a strict file: callers there see only signatures
        // (owner, 2026-09-29); elsewhere Precog works it out from the body
        if self.strict && matches!(ret_lent, Some((_, None))) { return self.err(format!("`#strict`: `{name}` hands back a borrow, so it names what the borrow depends on: `-> lent@x …` with `lent@x` on those parameters")); }
        let bounds = Self::flat_bounds(&gens);
        if !self.in_interface { self.declared = None; self.groups.clear(); }
        Ok(Func { name, module: self.module.clone(), vis: Vis::Private, ret, ret_lent, fails, is_extern: false, is_op: false, is_test: false, bounds, params, body, line })
    }

    // fail list := 'fail' '[' Word (':' type)? (',' Word (':' type)?)* ']'
    fn fail_list(&mut self) -> PResult<Vec<FailCase>> {
        self.expect_word("fail")?;
        self.expect(Tok::LBracket, "`[` after `fail` to list the ways this function can fail")?;
        let mut cases = Vec::new();
        loop {
            let name = match self.advance().tok {
                Tok::Word(w) if KEYWORDS.contains(&w.as_str()) => return self.err(format!("`{w}` is a keyword and cannot name a failure")),
                Tok::Word(w) => w,
                Tok::Var(v) => return self.err(format!("a failure's name is bare: `fail[{v}]` — it names a reason, not a variable")),
                other => return self.err(format!("expected a failure name, found {other}")),
            };
            if cases.iter().any(|c: &FailCase| c.name == name) { return self.err(format!("failure `{name}` is listed twice")); }
            let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
            cases.push(FailCase { name, ty });
            if *self.peek() == Tok::Comma { self.advance(); continue; }
            break;
        }
        self.expect(Tok::RBracket, "`]` to close the failure list")?;
        Ok(cases)
    }

    // param := ('lent' ('.' 'mut')? | 'mut' | 'immut')? Var (':' type)?
    // An owning parameter has no keyword; `lent` / `lent.mut` mirror the caller's `lend` / `lend.mut`.
    /// `@x` after `lent` / `lent.mut`: the lifetime group (owner, 2026-09-29) — introduced by
    /// first use, like `<T>`
    /// The announcement after an item's name (owner, 2026-10-02): every generic, one bracket each,
    /// with what it accepts after a `:` — `<T>: Ordered`, `<T>: Number and Hashable`, `<T>: any` —
    /// then, after a `|` when there are generics, the lifetime groups: `<T>: any | @x @y`.
    /// An interface announces its `<T>` with no bound (it is the type being checked).
    /// One C declaration after `extern.C`, or one line of an `extern.C { … }` block:
    /// `fn name [params] -> T;` or `['mut'] 'name': T;`.
    fn extern_item(&mut self, prog: &mut Program, vis: Vis) -> PResult<()> {
        let line = self.line();
        if self.is_word("let") { return self.err("C's variable takes no `let`: `extern.C 'errno': int32;` (or `extern.C mut 'x': T;`)"); }
        if self.is_word("mut") || self.is_word("immut") || matches!(self.peek(), Tok::Var(_)) {
            // a C global (decided 2026-09-21; no `let` since 2026-09-25)
            let mutable = if self.is_word("mut") { self.advance(); true } else { false };
            if self.is_word("immut") { return self.err("an extern variable is C's; it is read-only unless `mut`, never `!mut`"); }
            let name = match self.advance().tok { Tok::Var(v) => v, Tok::Word(w) => return self.err(format!("an extern variable is named like a variable, quoted: `extern.C '{w}': …`")), other => return self.err(format!("expected a quoted name, found {other}")) };
            if *self.peek() != Tok::Colon { return self.err(format!("the extern variable '{name}' needs a type; C has no inference: `extern.C '{name}': int32;`")); }
            self.advance();
            self.at_boundary = true;
            let ty = self.ty();
            self.at_boundary = false;
            let ty = ty?;
            if matches!(self.peek(), Tok::Walrus | Tok::Eq) { return self.err(format!("the extern variable '{name}' has no value here; C owns it. A value of your own is a `const`")); }
            self.expect(Tok::Semi, "`;` after the extern variable's type")?;
            prog.externs.push(ExternVar { name, module: self.module.clone(), vis, ty, mutable, line });
            return Ok(());
        }
        if !self.is_word("fn") { return self.err("after `extern.C` comes `fn` or a quoted name: `extern.C fn write [...] -> int64;`, `extern.C 'errno': int32;`"); }
        self.at_boundary = true;
        let f = self.func();
        self.at_boundary = false;
        let mut f = f?;
        f.vis = vis; f.is_extern = true;
        prog.funcs.push(f);
        Ok(())
    }

    fn announce(&mut self, what: &str) -> PResult<Vec<(String, Vec<String>)>> {
        let mut gens: Vec<(String, Vec<String>)> = Vec::new();
        let mut groups: Vec<String> = Vec::new();
        if matches!(self.peek(), Tok::Op(o) if o == "<") && self.lacks.iter().any(|c| c == "Genny") { return self.core_missing("Genny", "a generic"); }
        while matches!(self.peek(), Tok::Op(o) if o == "<") {
            self.advance();
            let name = match self.advance().tok {
                Tok::Word(n) => n,
                Tok::Var(v) => return self.err(format!("a generic's name is bare: `<{v}>`")),
                other => return self.err(format!("expected a generic's name after `<`, found {other}")),
            };
            if *self.peek() == Tok::Colon { return self.err(format!("what `<{name}>` accepts goes after the bracket: `<{name}>: any`")); }
            match self.advance().tok { Tok::Op(o) if o == ">" => {}, other => return self.err(format!("expected `>` to close `<{name}`, found {other}")) }
            if gens.iter().any(|(g, _)| *g == name) { return self.err(format!("`<{name}>` is announced twice")); }
            let mut bound = Vec::new();
            if what == "interface" {
                if *self.peek() == Tok::Colon { return self.err(format!("an interface's `<{name}>` is the type being checked; it accepts nothing in particular")); }
            } else {
                if *self.peek() != Tok::Colon { return self.err(format!("`<{name}>` says what it accepts: `<{name}>: any`, or an interface, `<{name}>: Ordered`")); }
                self.advance();
                loop {
                    if self.is_word("any") { self.advance(); if !bound.is_empty() || self.is_word("and") { return self.err("`any` stands alone: it already accepts every type"); } break; }
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), other => return self.err(format!("expected an interface's name, or `any`, after `<{name}>:`, found {other}")) }
                        if *self.peek() == Tok::Dot { self.advance(); continue; }
                        break;
                    }
                    let iface = self.type_name(&segs)?;
                    if bound.contains(&iface) { return self.err(format!("`{iface}` is named twice for `<{name}>`")); }
                    bound.push(iface);
                    if self.is_word("and") { self.advance(); continue; }
                    break;
                }
            }
            gens.push((name, bound));
        }
        let mut bar = false;
        if matches!(self.peek(), Tok::Op(o) if o == "|") {
            if gens.is_empty() { return self.err("a `|` separates generics from lifetime groups; with no generics, write the groups alone: `@x`"); }
            self.advance(); bar = true;
        }
        if matches!(self.peek(), Tok::Op(o) if o == "|@") { return self.err("a space after the `|`: `| @x`"); }
        while matches!(self.peek(), Tok::Op(o) if o == "@") {
            if what != "fn" { return self.err(format!("only a `fn` has lifetime groups; a{} `{what}` cannot hand back a borrow", if what == "op" || what == "enum" || what == "interface" { "n" } else { "" })); }
            if !gens.is_empty() && !bar { return self.err("a `|` separates the generics from the lifetime groups: `<T>: any | @x`"); }
            self.advance();
            match self.advance().tok {
                Tok::Word(g) => { if groups.contains(&g) { return self.err(format!("`@{g}` is announced twice")); } groups.push(g) }
                other => return self.err(format!("a lifetime group is named after `@`: `@x`, found {other}")),
            }
        }
        if bar && groups.is_empty() { return self.err("after the `|` come the lifetime groups: `<T>: any | @x`"); }
        self.declared = Some(gens.iter().map(|(g, _)| g.clone()).collect());
        self.groups = groups;
        Ok(gens)
    }

    fn flat_bounds(gens: &[(String, Vec<String>)]) -> Vec<(String, String)> {
        gens.iter().flat_map(|(g, b)| b.iter().map(move |i| (g.clone(), i.clone()))).collect()
    }

    /// A lending `&` here? Returns whether it came glued to `@` (`&@x` lexes as one operator).
    fn amp(&mut self) -> Option<bool> {
        match self.peek() {
            Tok::Op(o) if o == "&" => { self.advance(); Some(false) }
            Tok::Op(o) if o == "&@" => { self.advance(); Some(true) }
            _ => None,
        }
    }

    /// After `&`: `mut`, then a lifetime group (`&mut@x`); or the group's name straight after `&@`.
    fn amp_rest(&mut self, glued_at: bool) -> PResult<(bool, Option<String>)> {
        if glued_at {
            return match self.advance().tok {
                Tok::Word(w) => {
                    if !self.groups.contains(&w) { return self.err(format!("`@{w}` was never announced: announce it after the function's name, `fn f @{w} […]`")); }
                    Ok((false, Some(w)))
                }
                other => self.err(format!("a lifetime group is named after `@`: `&@x`, found {other}")),
            };
        }
        let m = if self.is_word("mut") { self.advance(); true } else { false };
        Ok((m, self.life_label()?))
    }

    fn life_label(&mut self) -> PResult<Option<String>> {
        if !matches!(self.peek(), Tok::Op(o) if o == "@") { return Ok(None); }
        self.advance();
        match self.advance().tok {
            Tok::Word(w) => {
                if !self.groups.contains(&w) { return self.err(format!("`@{w}` was never announced: announce it after the function's name, `fn f @{w} […]`")); }
                Ok(Some(w))
            }
            other => self.err(format!("a lifetime group is named after `@`: `lent@x`, found {other}")),
        }
    }

    fn param(&mut self) -> PResult<Param> {
        let line = self.line();
        let mut life = None;
        let mut kind = ParamKind::Owned;
        let mut mutability = None;
        match self.peek().clone() {
            Tok::Word(w) if w == "lent" => {
                self.advance();
                kind = if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(m) if m == "mut") {
                    self.advance(); self.advance(); ParamKind::LentMut
                } else { ParamKind::Lent };
                life = self.life_label()?;
            }
            Tok::Op(o) if o == "&" || o == "&@" => {
                // `&'text': T` / `&mut 'buf': T` — an alternative to `lent` / `lent.mut` (owner, 2026-09-21);
                // `&@x 'a': T` / `&mut@x 'a': T` with a lifetime group (owner, 2026-10-02)
                let glued = self.amp().unwrap_or(false);
                let (m, l) = self.amp_rest(glued)?;
                kind = if m { ParamKind::LentMut } else { ParamKind::Lent };
                life = l;
            }
            Tok::Word(w) if w == "mut" => { self.advance(); mutability = Some(Mutability::Mut); }
            Tok::Word(w) if w == "immut" => { self.advance(); mutability = Some(Mutability::Immut); }
            Tok::Word(w) if w == "var" || w == "let" => return self.err(format!("an owning parameter has no `{w}`: `'a': int32`")),
            Tok::Word(w) if w == "lend" => return self.err("a parameter is `lent` (what it receives); `lend` is what the caller does"),
            _ => {}
        }
        if *self.peek() == Tok::Dot { return self.err("a parameter's type follows its name: `lent 'text': str.utf8`"); }
        let mut variadic = false;
        if *self.peek() == Tok::Ellipsis { self.advance(); variadic = true; }
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            Tok::Word(w) => return self.err(format!("a parameter is `'name': type` — the name first, quoted (found `{w}`)")),
            Tok::Ellipsis => return self.err("`...` goes right before the name: `...'values'`"),
            other => return self.err(format!("expected a quoted parameter name, found {other}")),
        };
        // a signature may mention `ptr[T]` (a callback C calls receives pointers); a `let` may not
        let saved = self.at_boundary; self.at_boundary = true;
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()) } else { None };
        self.at_boundary = saved;
        let ty = match ty { Some(t) => Some(t?), None => None };
        if variadic && *self.peek() == Tok::Comma { return self.err(format!("`...'{name}'` takes the rest of the arguments, so it must be the last parameter")); }
        Ok(Param { kind, mutability, ty, name, variadic, life, line })
    }

    // type := Word ('.' Word)* ('[' type (',' type)* ']')?
    // Decided 2026-09-19: `.` picks a fixed variant (`str.utf8`, `char.grapheme`);
    // `[ ]` supplies type parameters (`list[int32]`, `list[list[int32]]`). A struct
    // through modules is a dotted path (`shapes.Locker`, `my.Locker`).
    fn ty(&mut self) -> PResult<Ty> {
        // `!std.int24` — an integer of a width that is not standard (owner, 2026-10-03; the `.` since
        // 2026-10-06: "not standard", narrowed to one): read as one name inside
        if matches!(self.peek(), Tok::Op(o) if o == "!") && matches!(self.peek_at(1), Tok::Word(w) if w.starts_with("std_")) {
            let Tok::Word(w) = self.peek_at(1).clone() else { unreachable!() };
            let fixed = w.replacen("std_", "std.", 1);
            return self.err(format!("`!{w}` is `!{fixed}` since 2026-10-06: \"not standard\", narrowed with `.`"));
        }
        if matches!(self.peek(), Tok::Op(o) if o == "!") && matches!(self.peek_at(1), Tok::Word(w) if w == "std") && *self.peek_at(2) == Tok::Dot {
            let Tok::Word(w) = self.peek_at(3).clone() else { return self.err("`!std.` is followed by a width: `!std.int24`, `!std.uint4`") };
            self.toks.drain(self.pos..self.pos + 3);
            self.toks[self.pos].tok = Tok::Word(format!("!std_{w}"));
        }
        // `<T>` — a type variable (design/generics.md, 2026-09-20)
        if matches!(self.peek(), Tok::Op(o) if o == "<") {
            self.advance();
            let name = match self.advance().tok {
                Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot name a type variable")),
                Tok::Word(n) => n,
                Tok::Var(v) => return self.err(format!("a type variable is a bare name: `<{v}>`")),
                other => return self.err(format!("expected a type variable's name after `<`, found {other}")),
            };
            if *self.peek() == Tok::Colon {
                return self.err(format!("what `<{name}>` accepts is said once, where it is announced after the name: `fn f <{name}>: Ordered […]`"));
            }
            if !self.declared.as_ref().map_or(false, |d| d.contains(&name)) {
                return self.err(format!("`<{name}>` was never announced: announce it after the name, `fn f <{name}>: any […]` (or `struct S <{name}>: any {{ … }}`)"));
            }
            match self.advance().tok { Tok::Op(o) if o == ">" => {}, other => return self.err(format!("expected `>` to close `<{name}`, found {other}")) }
            return Ok(Ty::Var(name));
        }
        if matches!(self.peek(), Tok::Op(o) if o.starts_with('<')) { return self.err("a type variable is `<T>` — with a space before it after `->` (`-> <T>`), since a run of operator characters is one token"); }
        // `fn[lent T, U] -> R or fail[…]` — a function's type (2026-09-21)
        if self.is_word("fn") {
            self.advance();
            self.expect(Tok::LBracket, "`[` after `fn` in a type: `fn[lent Locker, int32] -> bool`")?;
            let mut ps = Vec::new();
            if *self.peek() != Tok::RBracket {
                loop {
                    let kind = match self.peek().clone() {
                        Tok::Word(w) if w == "lent" => { self.advance(); if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(m) if m == "mut") { self.advance(); self.advance(); ParamKind::LentMut } else { ParamKind::Lent } }
                        Tok::Op(o) if o == "&" => { self.advance(); if self.is_word("mut") { self.advance(); ParamKind::LentMut } else { ParamKind::Lent } }
                        Tok::Var(v) => return self.err(format!("a function type has no parameter names: `fn[lent Locker]`, not `fn[lent '{v}': Locker]`")),
                        _ => ParamKind::Owned,
                    };
                    let t = self.ty()?;
                    ps.push((kind, t));
                    if *self.peek() == Tok::Comma { self.advance(); continue; }
                    break;
                }
            }
            self.expect(Tok::RBracket, "`]`")?;
            let mut ret = None;
            let mut fails = Vec::new();
            if *self.peek() == Tok::Arrow {
                self.advance();
                if self.is_word("fail") { fails = self.fail_list()?.into_iter().map(|c| c.name).collect(); }
                else {
                    ret = Some(Box::new(self.ty()?));
                    if self.is_word("or") { self.advance(); if !self.is_word("fail") { return self.err("after `or` comes `fail[…]`"); } fails = self.fail_list()?.into_iter().map(|c| c.name).collect(); }
                }
            }
            return Ok(Ty::Fn(ps, ret, fails));
        }
        // `std::duration` — the one std type the oracle knows (design/concurrency.md)
        if self.is_word("std") && *self.peek_at(1) == Tok::ColonColon {
            if matches!(self.peek_at(2), Tok::Word(w) if w == "duration") { self.advance(); self.advance(); self.advance(); return Ok(Ty::Duration); }
            return self.err("the only std type the oracle knows is `std::duration`");
        }
        // another Limb's type: `geometry::Circle` (limbs.md)
        if matches!(self.peek(), Tok::Word(w) if w != "std") && *self.peek_at(1) == Tok::ColonColon {
            let limb = match self.advance().tok { Tok::Word(w) => w, _ => unreachable!() };
            self.advance();
            let mut segs = vec![format!("::{limb}")];
            loop {
                match self.advance().tok { Tok::Word(w) => segs.push(w), other => return self.err(format!("expected a type name after `{limb}::`, found {other}")) }
                if *self.peek() == Tok::Dot { self.advance(); continue; }
                break;
            }
            self.path_forbidden(&segs.join("."))?;
            return Ok(Ty::Struct(segs.join(".")));
        }
        let mut segs = Vec::new();
        loop {
            match self.advance().tok {
                Tok::Word(w) => segs.push(w),
                other => return self.err(format!("expected a type segment, found {other}")),
            }
            if *self.peek() == Tok::Dot { self.advance(); continue; }
            break;
        }
        // Rust's number names are aliases (owner, 2026-10-02): `i32` is `int32`, `f64` is `bin64`
        if let Some(own) = rust_alias(&segs[0]) { segs[0] = own.to_string(); }
        // Rust names that are not aliases say why (owner, 2026-10-02: only the number names)
        if segs.len() == 1 {
            match segs[0].as_str() {
                "usize" | "isize" => return self.err(format!("`{}` is as big as the machine's pointer, which no CyborgPL type is: say the width, `{}`", segs[0], if segs[0] == "usize" { "uint64" } else { "int64" })),
                "String" => return self.err("text is `str.utf8`; Rust's `String` is not an alias"),
                "f128" => return self.err("there is no `bin128` (2026-09-23), so no `f128`: `std::uenr` is exact rather than wider"),
                _ => {}
            }
        }
        // `int32[0, 100]` — a range type (design/input.md): a numeric head with two bounds
        {
            let head = segs[0].as_str();
            let numeric = ["int", "uint", "bin", "deci"].iter().any(|p| head.strip_prefix(p).map(|w| w.parse::<u32>().is_ok()).unwrap_or(false));
            let range_ahead = *self.peek() == Tok::LBracket && (matches!(self.peek_at(1), Tok::Int(_) | Tok::Bin(..))
                || matches!(self.peek_at(1), Tok::Op(o) if o == "-")
                || matches!(self.peek_at(1), Tok::Word(w) if w == "min" || w == "max"));
            if numeric && range_ahead && (!self.to_target || self.bracket_then_bracket()) {
                self.advance();
                let lo = self.bound()?;
                self.expect(Tok::Comma, "`,` between the bounds")?;
                let hi = self.bound()?;
                self.expect(Tok::RBracket, "`]` after the bounds")?;
                let base = Parser::new_in(vec![Token { tok: Tok::Word(head.to_string()), line: 0, col: 0, mtm: None }, Token { tok: Tok::Eof, line: 0, col: 0, mtm: None }], self.module.clone()).ty().map_err(|e| ParseError { line: self.line(), col: 0, msg: e.msg })?;
                match (&base, &lo, &hi) {
                    (Ty::Int(_) | Ty::Uint(_), Bound::Bin(_), _) | (Ty::Int(_) | Ty::Uint(_), _, Bound::Bin(_)) => return self.err("an integer's bounds are integers"),
                    (Ty::Bin(_) | Ty::Deci(_), Bound::Int(_), _) | (Ty::Bin(_) | Ty::Deci(_), _, Bound::Int(_)) => return self.err("a bin's bounds are written as bins: `bin64[0.0, 1.0]`"),
                    _ => {}
                }
                // `min` / `max` are the base type's limits, so they order like any number
                let (rlo, rhi) = (crate::interp::resolve_bound(&base, &lo), crate::interp::resolve_bound(&base, &hi));
                // a bound the type cannot hold says nothing, or is a mistake (owner, 2026-09-23)
                let (tlo, thi) = (crate::interp::resolve_bound(&base, &Bound::Min), crate::interp::resolve_bound(&base, &Bound::Max));
                for b in [&rlo, &rhi] {
                    let fits = match (b, &tlo, &thi) {
                        (Bound::Int(v), Bound::Int(a), Bound::Int(z)) => a <= v && v <= z,
                        (Bound::Bin(v), Bound::Bin(a), Bound::Bin(z)) => a <= v && v <= z,
                        _ => true,
                    };
                    if !fits { return self.err(format!("{} does not fit {head}: a range's bounds are values of its type", crate::value::bound_name(b))); }
                }
                let ok = match (&rlo, &rhi) { (Bound::Int(a), Bound::Int(b)) => a <= b, (Bound::Bin(a), Bound::Bin(b)) => a <= b, _ => true };
                if !ok { return self.err("a range's low bound comes first: `int32[0, 100]`"); }
                // `int32[min, max]` and `uint8[0, 255]` are the plain type written long: harmless,
                // so allowed (owner, 2026-09-23: "they're harmless. So, no problem")
                return Ok(Ty::Ranged(Box::new(base), lo, hi));
            }
        }
        let mut params = Vec::new();
        // `[` followed by a number is a bound (`str.utf8[1, 40 graphemes]`), not type parameters
        // `simd[T, N]` — a type and a count, not two types (design/simd.md, 2026-09-22)
        if segs[0] == "simd" {
            if segs.len() > 1 { return self.err("`simd` takes its type and count in brackets: `simd[uint8, 16]`"); }
            if *self.peek() != Tok::LBracket { return self.err("`simd` says what and how many: `simd[uint8, 16]`"); }
            self.advance();
            let t = self.ty()?;
            self.expect(Tok::Comma, "`,` between a simd's type and its lane count")?;
            let n = match self.advance().tok { Tok::Int(i) if i > 0 => i as usize, other => return self.err(format!("a simd's lane count is a whole number: `simd[uint8, 16]`, found {other}")) };
            self.expect(Tok::RBracket, "`]` after the lane count")?;
            if !matches!(t, Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Bool) { return self.err(format!("a simd holds plain numbers or bools, not {}", ty_name(&t))); }
            // Only the target's own register shapes. The oracle's target is the owner's
            // machine: ARM64, 128-bit NEON (design/simd.md; a real target list is the compiler's).
            // a mask (`simd[bool, N]`, owner 2026-09-27: a) has as many lanes as the vectors it
            // compares, so any count the target has for some element type: 2, 4, 8 or 16
            // a lane is a standard width (owner, 2026-10-03: a): no chip has a `!std_` lane, and an
            // emulated one is slower than plain code
            if let Ty::Int(IntWidth::Other(_)) | Ty::Uint(IntWidth::Other(_)) = &t {
                return self.err(format!("a vector's lanes are a standard width (8, 16, 32 or 64 bits), not `{}` (owner, 2026-10-03)", crate::value::ty_name(&t)));
            }
            if matches!(t, Ty::Bool) {
                if ![2, 4, 8, 16].contains(&n) { return self.err(format!("a mask has as many lanes as the vectors it compares: 2, 4, 8 or 16 on this target, not {n}")); }
                return Ok(Ty::Simd(Box::new(t), n));
            }
            let bits = match &t { Ty::Int(w) | Ty::Uint(w) => match w { IntWidth::W8 => 8, IntWidth::W16 => 16, IntWidth::W32 => 32, IntWidth::W64 => 64, IntWidth::W128 => 128, IntWidth::Other(n) => *n as usize }, Ty::Bin(w) => *w as usize, _ => 0 };
            if bits * n != 128 {
                let fits = 128 / bits.max(1);
                return self.err(format!("this target's vector registers are 128 bits, so a `simd[{}, …]` holds {fits} lanes, not {n}", ty_name(&t)));
            }
            return Ok(Ty::Simd(Box::new(t), n));
        }
        if *self.peek() == Tok::LBracket && !self.to_target && !matches!(self.peek_at(1), Tok::Int(_)) {
            self.advance();
            loop {
                params.push(self.ty()?);
                if *self.peek() == Tok::Comma { self.advance(); continue; }
                break;
            }
            self.expect(Tok::RBracket, "`]` to close the type parameters")?;
        }
        let head = segs[0].as_str();
        if head == "ptr" {
            if !self.at_boundary { return bad_ptr(self); }
            let mutable = match segs.len() { 1 => false, 2 if segs[1] == "mut" => true, _ => return self.err("a pointer is `ptr[T]` or `ptr.mut[T]`") };
            if params.len() != 1 { return self.err("a pointer says what it points at: `ptr[uint8]`, `ptr.mut[Timeval]`"); }
            return Ok(Ty::Ptr(Box::new(params.pop().unwrap()), mutable));
        }
        let width = |s: &str, prefix: &str| -> Option<u32> { s.strip_prefix(prefix).and_then(|w| w.parse().ok()) };
        let int_w = |n: u32| match n { 8 => Some(IntWidth::W8), 16 => Some(IntWidth::W16), 32 => Some(IntWidth::W32), 64 => Some(IntWidth::W64), 128 => Some(IntWidth::W128), _ => None };
        let bad = |s: &Self, m: String| -> PResult<Ty> { s.err(m) };
        let no_params = |s: &Self, name: &str| -> PResult<()> { if params.is_empty() { Ok(()) } else { s.err(format!("`{name}` takes no type parameters")) } };
        match head {
            "bool" => { if segs.len() > 1 { return bad(self, "`bool` takes no narrowing".into()); } no_params(self, "bool")?; Ok(Ty::Bool) }
            "str" => {
                if segs.len() != 2 { return bad(self, "the typed form is fully explicit: `str.utf8`, `str.utf16`, `str.utf32`, or `str.ascii` — never bare `str`".into()); }
                no_params(self, "str")?;
                let enc = match segs[1].as_str() {
                    "utf8" => Encoding::Utf8, "utf16" => Encoding::Utf16, "utf32" => Encoding::Utf32, "ascii" => Encoding::Ascii,
                    e => return bad(self, format!("unknown text encoding `{e}`")),
                };
                // `str.utf8[1, 40 graphemes]` — a length bound in a named unit (decided 2026-09-22):
                // text has three lengths, so the bound says which it counts.
                if *self.peek() == Tok::LBracket && matches!(self.peek_at(1), Tok::Int(_)) && (!self.to_target || self.bracket_then_bracket()) {
                    self.advance();
                    let lo = match self.advance().tok { Tok::Int(i) => i, _ => unreachable!() };
                    self.expect(Tok::Comma, "`,` between the length bounds")?;
                    let hi = match self.advance().tok { Tok::Int(i) => i, other => return bad(self, format!("a text's length bounds are whole numbers, found {other}")) };
                    let unit = match self.advance().tok {
                        Tok::Word(w) if w == "bytes" => CharUnit::Byte,
                        Tok::Word(w) if w == "points" => CharUnit::Point,
                        Tok::Word(w) if w == "graphemes" => CharUnit::Grapheme,
                        Tok::RBracket => return bad(self, "a text has three lengths: say which the bound counts — `str.utf8[1, 40 graphemes]`, `[…bytes]`, or `[…points]`".into()),
                        other => return bad(self, format!("the unit after a text's bounds is `bytes`, `points`, or `graphemes`, found {other}")),
                    };
                    self.expect(Tok::RBracket, "`]` after the unit")?;
                    if lo < 0 { return bad(self, "a length is never negative: the low bound is 0 or more".into()); }
                    if lo > hi { return bad(self, "a range's low bound comes first: `str.utf8[0, 100 bytes]`".into()); }
                    return Ok(Ty::TextRanged(enc, lo, hi, unit));
                }
                Ok(Ty::Str(enc))
            }
            "char" => {
                if segs.len() != 2 { return bad(self, "the typed form is fully explicit: `char.point`, `char.grapheme`, or `char.byte` — never bare `char`".into()); }
                no_params(self, "char")?;
                Ok(Ty::Char(match segs[1].as_str() {
                    "point" => CharUnit::Point, "grapheme" => CharUnit::Grapheme, "byte" => CharUnit::Byte,
                    u => return bad(self, format!("unknown character unit `{u}`")),
                }))
            }
            "channel" => {
                // channel[T] / channel[T][0, N]  (design/concurrency.md)
                if segs.len() > 1 { return bad(self, "`channel` takes its element type in brackets: `channel[str.utf8]`".into()); }
                if params.len() != 1 { return bad(self, "`channel` takes exactly one type: `channel[str.utf8]`".into()); }
                let base = Ty::Channel(Box::new(params.pop().unwrap()));
                if *self.peek() == Tok::LBracket && matches!(self.peek_at(1), Tok::Int(_)) {
                    self.advance();
                    let lo = self.bound()?; self.expect(Tok::Comma, "`,` between the bounds")?; let hi = self.bound()?; self.expect(Tok::RBracket, "`]` after the bounds")?;
                    let (Bound::Int(a), Bound::Int(b)) = (&lo, &hi) else { return bad(self, "a channel's bound is whole numbers: `channel[T][0, 100]`".into()) };
                    if *a != 0 { return bad(self, "a channel's low bound is 0: it may be empty".into()); }
                    if *b < 1 { return bad(self, "a channel that holds nothing is a hand-off: write `channel[T]` with no bound".into()); }
                    return Ok(Ty::Ranged(Box::new(base), lo, hi));
                }
                Ok(base)
            }
            "map" => {
                // map[K, V]  (decided 2026-09-22: a language collection beside list and array)
                // `map.ordered` / `map.unordered` — the variant, when the writer chooses (owner, 2026-09-22)
                let order = match segs.len() {
                    1 => MapOrder::Any,
                    2 if segs[1] == "ordered" => MapOrder::Ordered,
                    2 if segs[1] == "unordered" => MapOrder::Unordered,
                    _ => return bad(self, format!("`map` is `map[K, V]`, `map.ordered[K, V]`, or `map.unordered[K, V]`; `map.{}` is not a variant", segs[1..].join("."))),
                };
                if params.len() != 2 { return bad(self, "`map` takes a key type and a value type: `map[str.utf8, int32]`".into()); }
                let v = params.pop().unwrap(); let k = params.pop().unwrap();
                match &k { Ty::Fn(..) | Ty::Ptr(..) | Ty::Map(..) => return bad(self, format!("{} cannot be a map key; keys are plain values, text, chars, structs, enums and lists of those, or a type that satisfies `Key`", ty_name(&k))), _ => {} }
                let base = Ty::Map(Box::new(k), Box::new(v), order);
                // `map[K, V][0, 100]` — an entry-count bound, as a list's (decided 2026-09-22)
                if *self.peek() == Tok::LBracket && matches!(self.peek_at(1), Tok::Int(_) | Tok::Bin(..)) {
                    self.advance();
                    let lo = self.bound()?;
                    self.expect(Tok::Comma, "`,` between the count bounds")?;
                    let hi = self.bound()?;
                    self.expect(Tok::RBracket, "`]` after the count bounds")?;
                    let (Bound::Int(a), Bound::Int(b)) = (&lo, &hi) else { return bad(self, "a map's count bounds are whole numbers: `map[K, V][0, 100]`".to_string()) };
                    if *a < 0 { return bad(self, "a count is never negative: the low bound is 0 or more".to_string()); }
                    if a > b { return bad(self, "a range's low bound comes first: `map[K, V][0, 100]`".to_string()); }
                    return Ok(Ty::Ranged(Box::new(base), lo, hi));
                }
                Ok(base)
            }
            "list" | "array" => {
                if segs.len() > 1 { return bad(self, format!("`{head}` takes its element type in brackets: `{head}[{}]` (decided 2026-09-19), not `{}`", segs[1..].join("."), segs.join("."))); }
                if params.len() != 1 { return bad(self, format!("`{head}` takes exactly one type parameter: `{head}[int32]`, `{head}[str.utf8]`, `{head}[list[int32]]`")); }
                let inner = params.pop().unwrap();
                let base = if head == "list" { Ty::List(Box::new(inner)) } else { Ty::Array(Box::new(inner)) };
                // `list[int32][0, 1000]` — a length bound, the thing then its bounds like `int32[0, 100]` (decided 2026-09-21)
                if *self.peek() == Tok::LBracket && matches!(self.peek_at(1), Tok::Int(_) | Tok::Bin(..)) {
                    if head == "array" { return bad(self, "an `array` has one fixed length, so a bound on it says nothing; a length bound belongs on a `list`".to_string()); }
                    self.advance();
                    let lo = self.bound()?;
                    self.expect(Tok::Comma, "`,` between the length bounds")?;
                    let hi = self.bound()?;
                    self.expect(Tok::RBracket, "`]` after the length bounds")?;
                    let (Bound::Int(a), Bound::Int(b)) = (&lo, &hi) else { return bad(self, "a length bound is a pair of whole numbers: `list[int32][0, 1000]`".to_string()) };
                    if *a < 0 { return bad(self, "a length is never negative: the low bound is 0 or more".to_string()); }
                    if a > b { return bad(self, "a range's low bound comes first: `list[int32][0, 1000]`".to_string()); }
                    return Ok(Ty::Ranged(Box::new(base), lo, hi));
                }
                Ok(base)
            }
            "int" | "uint" | "bin" | "deci" => bad(self, format!("the typed form is fully explicit: `{head}` needs a width (e.g. `{head}32`)")),
            _ if KEYWORDS.contains(&head) => bad(self, format!("`{head}` is a keyword, not a type")),
            _ => {
                if !params.is_empty() {
                    let builtin = ["int", "uint", "bin", "deci"].iter().any(|p| width(head, p).is_some());
                    if builtin { return bad(self, format!("`{head}` takes no type parameters")); }
                    // `Stack[int32]` — a generic struct or enum with its blanks filled
                    return Ok(Ty::Inst(self.type_name(&segs)?, params));
                }
                if segs.len() > 1 {
                    // a struct reached through modules: `shapes.Locker`, `my.Locker`
                    if segs[..segs.len() - 1].iter().any(|m| m == "my" || module_name_ok(m).is_ok()) {
                        return Ok(Ty::Struct(self.type_name(&segs)?));
                    }
                    return bad(self, format!("`{}` takes no narrowing", segs.join(".")));
                }
                if let Some(rest) = head.strip_prefix("!std_") {
                    // our spelling or Rust's (MTM's): `!std.int24` / `!std.i24`, `!std.uint4` / `!std.u4`, held inside as `!std_…`
                    let (signed, digits) = if let Some(d) = rest.strip_prefix("uint") { (false, d) } else if let Some(d) = rest.strip_prefix("int") { (true, d) }
                        else if let Some(d) = rest.strip_prefix('u') { (false, d) } else if let Some(d) = rest.strip_prefix('i') { (true, d) } else { (true, "") };
                    let n: u32 = match digits.parse() { Ok(n) if n >= 1 => n, _ => return bad(self, format!("`{}` is not a type: a non-standard integer is `!std.int24` or `!std.uint24` (MTM's `!std.i24`, `!std.u24`)", head.replacen("!std_", "!std.", 1))) };
                    let std_name = if signed { "int" } else { "uint" };
                    if int_w(n).is_some() { return bad(self, format!("`{std_name}{n}` is standard: write `{std_name}{n}` (owner, 2026-10-03)")); }
                    // LLVM's widest integer is 2^23 bits (checked with `llvm-as`, 2026-10-03)
                    if n > 8_388_608 { return bad(self, format!("`{}` is past the widest integer there is: LLVM's integers go up to 8388608 bits", head.replacen("!std_", "!std.", 1))); }
                    return Ok(if signed { Ty::Int(IntWidth::Other(n)) } else { Ty::Uint(IntWidth::Other(n)) });
                }
                if let Some(n) = width(head, "int") { return int_w(n).map(Ty::Int).ok_or(()).or_else(|_| bad(self, format!("no such width `{head}`; int is 8/16/32/64/128"))); }
                if let Some(n) = width(head, "uint") { return int_w(n).map(Ty::Uint).ok_or(()).or_else(|_| bad(self, format!("no such width `{head}`; uint is 8/16/32/64/128"))); }
                // bin128 was dropped (owner, 2026-09-23): no hardware anywhere, and `std::uenr`
                // answers "more precision than 64 bits" by being exact rather than wider.
                // the brain float (owner, 2026-10-02): 16 bits only, a family of its own
                if head == "brain16" { return Ok(Ty::Brain); }
                if let Some(n) = width(head, "brain") { return bad(self, format!("no such width `brain{n}`; the brain float is `brain16` only")); }
                if let Some(n) = width(head, "bin") {
                    if n == 128 { return bad(self, "there is no `bin128`: quad precision has no hardware, and `std::uenr` is exact rather than merely wider".into()); }
                    return if [16, 32, 64].contains(&n) { Ok(Ty::Bin(n as u8)) } else { bad(self, format!("no such width `{head}`; bin is 16/32/64")) };
                }
                if let Some(n) = width(head, "deci") { return if [32, 64, 128].contains(&n) { Ok(Ty::Deci(n as u8)) } else { bad(self, format!("no such width `{head}`; deci is 32/64/128 (IEEE 754 defines only these)")) }; }
                // anything else is a struct name in this module; the interpreter checks it was declared
                Ok(Ty::Struct(self.type_name(&segs)?))
            }
        }
    }

    // block := '{' stmt* '}'
    fn block(&mut self) -> PResult<Vec<Stmt>> {
        self.expect(Tok::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof { return self.err("unclosed `{`"); }
            stmts.push(self.stmt()?);
        }
        self.advance();
        Ok(stmts)
    }

    fn stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        match self.peek().clone() {
            Tok::Word(w) => match w.as_str() {
                // `let` is back (owner, 2026-10-02): every declaration starts with it; `change` changes
                "let" => {
                    self.advance();
                    if *self.peek() == Tok::Dot { return self.err("`let` is a word of its own, then `mut`: `let mut 'x' := …;`"); }
                    self.decl()
                }
                "var" => self.err("`var` is gone: declare with `let 'x' := 1;` or `let 'x': int32 = 1;`"),
                "mut" | "immut" => self.err(format!("a declaration starts with `let`: `let {w} 'x' := …;`")),
                _ if matches!(self.peek_at(1), Tok::Var(_)) && !KEYWORDS.contains(&w.as_str()) => self.err(format!("a type goes after the name: `'x': {w} = …;`")),
                "change" => { self.advance(); self.change_stmt(line) }
                // `must …;` — a test's assertion (owner, 2026-09-29): `must`, `must.fail[.reason]`, `must.stop`
                "must" => {
                    if !self.in_test { return self.err("`must` belongs in a `test fn`; in a program, Precog proves what must hold (`check`)"); }
                    self.advance();
                    let (mut kind, mut reason) = (None, None);
                    if *self.peek() == Tok::Dot {
                        self.advance();
                        match self.advance().tok {
                            Tok::Word(w) if w == "fail" => {
                                kind = Some("fail");
                                if *self.peek() == Tok::Dot {
                                    self.advance();
                                    match self.advance().tok { Tok::Word(r) => reason = Some(r), other => return self.err(format!("expected a failure's name after `must.fail.`, found {other}")) }
                                }
                            }
                            Tok::Word(w) if w == "stop" => kind = Some("stop"),
                            other => return self.err(format!("`must` narrows to `must.fail` or `must.stop`, not `must.{other}`")),
                        }
                    }
                    let mut e = self.expr()?;
                    self.expect(Tok::Semi, "`;`")?;
                    match kind {
                        None => Ok(Stmt::Must { cond: e, line }),
                        Some("fail") => {
                            // the failure climbs out to `must.fail` instead of needing an `otherwise`
                            let can = match &mut e {
                                Expr::Call(c) if c.handler.is_none() => { c.handler = Some(Handler::Propagate); true }
                                Expr::Convert(_, _, h, _, _) | Expr::MapFail(_, _, h, _) | Expr::CharFromCode(_, _, h, _) | Expr::PtrReadChecked(_, h, _) | Expr::CallValue(_, _, h, _) if h.is_none() => { *h = Some(Handler::Propagate); true }
                                _ => false,
                            };
                            if !can { return self.err("`must.fail` takes something that can fail, with no `otherwise`: a call, or `to.…[…]`"); }
                            Ok(Stmt::MustFail { expr: e, reason, line })
                        }
                        _ => Ok(Stmt::MustStop { expr: e, line }),
                    }
                }
                "if" => self.if_stmt(),
                "on" => self.on_stmt(),
                "loop" => self.loop_stmt(),
                "while" => self.while_stmt(),
                "for" => self.for_stmt(),
                "break" => { self.advance(); self.expect(Tok::Semi, "`;`")?; Ok(Stmt::Break { line }) }
                "continue" => { self.advance(); self.expect(Tok::Semi, "`;`")?; Ok(Stmt::Continue { line }) }
                "return" => {
                    self.advance();
                    let value = if *self.peek() == Tok::Semi { None } else { Some(self.rhs()?) };
                    if value.is_some() { self.saw_value_return = true; }
                    self.expect(Tok::Semi, "`;` after return")?;
                    Ok(Stmt::Return { value, line })
                }
                "fn" | "func" => self.err("a `fn` cannot be defined inside a body; it goes at the top level"),
                "const" => self.err("a `const` sits at the top level of the Limb; inside a body use `let` (Precog finds the constants itself)"),
                "pub" => self.err("`pub` goes at the top level, before `fn`, `struct`, `const`, or `module`"),
                "module" => self.err("a `module` block sits at the top level of a file, not inside a body"),
                "MAIN" => self.err("MAIN cannot be nested or called"),
                "else" => self.err("`else` without an `if`"),
                "check" | "nocheck" | "NO_FUCKING_CHECKS" => {
                    // decided 2026-09-19: a word before a statement (or a `{ }` block).
                    // `NO_FUCKING_CHECKS` (owner, 2026-09-27): `nocheck`, and C's answers trusted too
                    self.advance();
                    let tier = match w.as_str() { "check" => Tier::Check, "nocheck" => Tier::NoCheck, _ => Tier::NoChecksAtAll };
                    // vouching is a power IEG grants (owner, 2026-10-07: "`a`")
                    match tier { Tier::NoCheck => self.use_power("vouch", None, line), Tier::NoChecksAtAll => self.use_power("vouch.c", None, line), _ => {} }
                    match self.peek().clone() {
                        Tok::Semi => return self.err(format!("`{w}` goes in front of a statement: `{w} 'xs'[pos 'i'] = 5;` or `{w} {{ … }}`")),
                        Tok::Word(k) if k == "fn" || k == "struct" || k == "const" || k == "module" => return self.err(format!("`{w}` covers a statement, not a `{k}` — a function's safety is the sum of its statements")),
                        Tok::Word(k) if k == "check" || k == "nocheck" || k == "NO_FUCKING_CHECKS" => return self.err(format!("`{w} {k}`: one tier per statement; the innermost wins when they nest")),
                        _ => {}
                    }
                    let body = self.stmt()?;
                    Ok(Stmt::Tier { tier, body: Box::new(body), line })
                }
                "give" | "copy" | "lend" => self.err(format!("`{w}` belongs inside a value: `'s' = {w} 't';`")),
                "list" | "array" | "map" | "simd" => {
                    let (head, op, args) = self.builtin()?;
                    if head == "map" && (op == "get" || op == "remove") {
                        // a call that can fail, as a statement: `map.remove[…] otherwise …;` (the value is dropped)
                        let handler = if self.is_word("otherwise") { self.handler()? } else { None };
                        self.expect(Tok::Semi, "`;`")?;
                        return Ok(Stmt::MapFailStmt { op, args, handler, line });
                    }
                    self.expect(Tok::Semi, "`;`")?;
                    Ok(Stmt::BuiltinStmt { head, op, args, line })
                }
                "channel" => {
                    let (head, op, args) = self.builtin()?;
                    if op == "take" {
                        let handler = if self.is_word("otherwise") { self.handler()? } else { None };
                        self.expect(Tok::Semi, "`;`")?;
                        return Ok(Stmt::MapFailStmt { op, args, handler, line });
                    }
                    self.expect(Tok::Semi, "`;`")?;
                    Ok(Stmt::BuiltinStmt { head, op, args, line })
                }
                "concur" => {
                    // concur [.detached] [with [captures]] { … } [otherwise …];   |   concur f[…] [otherwise …];
                    self.advance();
                    let detached = if *self.peek() == Tok::Dot {
                        self.advance();
                        match self.advance().tok { Tok::Word(d) if d == "detached" => true, other => return self.err(format!("the only variant of `concur` is `concur.detached`, found {other}")) }
                    } else { false };
                    let caps = self.captures()?;
                    let body = if *self.peek() == Tok::LBrace { self.block()? } else {
                        // a call, run as a task
                        let cline = self.line();
                        let c = self.call()?;
                        vec![Stmt::Call { call: c, line: cline }]
                    };
                    let handler = if self.is_word("otherwise") { self.handler()? } else { None };
                    if *self.peek() == Tok::Semi { self.advance(); }
                    Ok(Stmt::Concur { detached, caps, body, handler, line })
                }
                "whichever" => {
                    // whichever { 'v' from lend 'c' { … } … } [otherwise …];
                    self.advance();
                    self.expect(Tok::LBrace, "`{` after `whichever`")?;
                    let mut arms = Vec::new();
                    while *self.peek() != Tok::RBrace {
                        let aline = self.line();
                        let name = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("a `whichever` arm names its value: `'msg' from lend 'inbox' {{ … }}`, found {other}")) };
                        if !self.is_word("from") { return self.err("after the arm's name comes `from`: `'msg' from lend 'inbox' { … }`"); }
                        self.advance();
                        let chan = Bracket { values: vec![self.value()?], line: aline, one_list: false };
                        let body = self.block()?;
                        arms.push(WhicheverArm { name, chan, body, line: aline });
                    }
                    self.advance();
                    if arms.is_empty() { return self.err("`whichever` needs at least one arm"); }
                    let handler = if self.is_word("otherwise") { self.handler()? } else { None };
                    if *self.peek() == Tok::Semi { self.advance(); }
                    Ok(Stmt::Whichever { arms, handler, line })
                }
                "fail" => {
                    // fail name; | fail name[payload];   (decided 2026-09-19)
                    self.advance();
                    let name = match self.advance().tok {
                        Tok::Word(w) if KEYWORDS.contains(&w.as_str()) => return self.err(format!("`{w}` is a keyword, not a failure name")),
                        Tok::Word(w) => w,
                        Tok::Var(v) => return self.err(format!("a failure's name is bare: `fail {v};`")),
                        Tok::Semi => return self.err("`fail` names its reason: `fail full;` — the reasons are listed in the signature's `fail[…]`"),
                        other => return self.err(format!("expected a failure name after `fail`, found {other}")),
                    };
                    let payload = if *self.peek() == Tok::LBracket { Some(self.bracket()?) } else { None };
                    self.expect(Tok::Semi, "`;` after fail")?;
                    Ok(Stmt::Fail { name, payload, line })
                }
                "otherwise" => self.err("`otherwise` follows a call that can fail: `stash[…] otherwise false;`"),
                "to" => self.err("`to.…[…]` is a value; use it: `let 'n' := to.int32[\"42\"] otherwise 0;`"),
                "ptr" => {
                    // ptr.write[…]; as a statement
                    let (head, op, args) = self.ptr_op()?;
                    self.expect(Tok::Semi, "`;`")?;
                    Ok(Stmt::BuiltinStmt { head, op, args, line })
                }
                "extern" | "layout" => self.err(format!("`{w}` sits at the top level of a file")),
                "asm" | "ASM" => self.err("inline assembly is `inline.Assembly { … }` (decided 2026-09-20; the whole name since 2026-10-06)"),
                "gives" => return self.err("`gives` is spelled `giveS` (owner, 2026-09-29: the capital S keeps it apart from `give`)"),
                "giveS" => {
                    if !self.strict { return self.err("`giveS` belongs to `#strict` files; elsewhere a case's value is written alone: `empty { \"free\" }`"); }
                    if self.arm_depth == 0 { return self.err("`giveS` gives a `match` or `otherwise` case its value; there is no case here"); }
                    self.advance();
                    let value = self.rhs()?;
                    self.expect(Tok::Semi, "`;` after the value")?;
                    Ok(Stmt::Gives { value, line })
                }
                "match" => {
                    // match v { case ['bind'] { … } … }   (design/enums.md, 2026-09-20)
                    self.advance();
                    let value = self.rhs()?;
                    if *self.peek() != Tok::LBrace { return self.err(format!("expected `{{` after the value to match, found {}", self.peek())); }
                    let arms = self.arms("match")?;
                    Ok(Stmt::Match { value, arms, line })
                }
                "enum" => self.err("an `enum` sits at the top level of a file"),
                "inline" => {
                    // nocheck inline.ASM { … } with [ … ];   (design/ffi.md, 2026-09-20)
                    // The language's name is in capitals, like MAIN: easy to spot (owner).
                    self.advance();
                    self.expect(Tok::Dot, "`.Assembly` after `inline` (`inline.Assembly { … }`)")?;
                    match self.advance().tok {
                        // `inline.S` is another name for it, as assembly's `.S` files (owner, 2026-10-02)
                        // `inline.Assembly`, the language's whole name (owner, 2026-10-06); `ASM` and `S` are MTM's
                        Tok::Word(l) if l == "Assembly" || l == "ASM" || l == "S" => { self.use_power("inline_assembly", None, self.line()); }
                        Tok::Word(l) if l == "asm" || l == "s" || l == "assembly" => return self.err("a language's name is capitalized, written whole: `inline.Assembly` (MTM's `inline.ASM`, `inline.S`)".to_string()),
                        Tok::Word(l) => return self.err(format!("`inline.{l}` is not decided; only `inline.Assembly` exists (MTM's `inline.ASM`, `inline.S`) (`inline.C` is possible later — LLVM ships clang)")),
                        other => return self.err(format!("expected `ASM` after `inline.`, found {other}")),
                    }
                    let text = match self.advance().tok {
                        Tok::Raw(t) => t,
                        Tok::LBracket => return self.err("assembly goes in braces, verbatim: `inline.Assembly { mov x16, #4 … }` — not a list of strings"),
                        other => return self.err(format!("expected `{{` after `inline.Assembly`, found {other}")),
                    };
                    let mut binds = Vec::new();
                    if self.is_word("with") {
                        self.advance();
                        self.expect(Tok::LBracket, "`[` after `with`")?;
                        if *self.peek() != Tok::RBracket {
                            loop {
                                match self.advance().tok {
                                    Tok::Var(v) => {
                                        let dir = self.advance().tok;
                                        let reg = match self.advance().tok { Tok::Word(r) => r, other => return self.err(format!("expected a register name, found {other}")) };
                                        binds.push(match dir {
                                            Tok::Arrow => AsmBind::In(v, reg),
                                            Tok::LeftArrow => AsmBind::Out(v, reg),
                                            Tok::BothArrow => AsmBind::InOut(v, reg),
                                            other => return self.err(format!("after '{v}' comes an arrow: `-> reg` (in), `<- reg` (out), `<-> reg` (both); found {other}")),
                                        });
                                    }
                                    // a lend satisfies a pointer, as in an extern call (owner, 2026-09-30: a)
                                    Tok::Word(w) if w == "lend" => {
                                        let mutable = if *self.peek() == Tok::Dot {
                                            self.advance();
                                            match self.advance().tok { Tok::Word(m) if m == "mut" => true, other => return self.err(format!("the only narrowing of `lend` is `.mut`, found {other}")) }
                                        } else { false };
                                        let lend = if mutable { "lend.mut" } else { "lend" };
                                        let v = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a variable after `{lend}`, found {other}")) };
                                        match self.advance().tok {
                                            Tok::Arrow => {}
                                            Tok::LeftArrow | Tok::BothArrow => return self.err(format!("a lend goes into a register only: `{lend} '{v}' -> x1` puts the address of '{v}' there")),
                                            other => return self.err(format!("after `{lend} '{v}'` comes `-> register`, found {other}")),
                                        }
                                        let reg = match self.advance().tok { Tok::Word(r) => r, other => return self.err(format!("expected a register name, found {other}")) };
                                        binds.push(AsmBind::Lend(v, reg, mutable));
                                    }
                                    Tok::Word(w) if w == "clobber" => {
                                        let reg = match self.advance().tok { Tok::Word(r) => r, other => return self.err(format!("expected a register after `clobber`, found {other}")) };
                                        binds.push(AsmBind::Clobber(reg));
                                    }
                                    Tok::Word(w) if w == "in" || w == "out" || w == "inout" => return self.err(format!("no `{w}` word: the arrow says the direction — `'v' -> x0`, `'v' <- x0`, `'v' <-> x0`")),
                                    other => return self.err(format!("expected `'variable' -> register`, `'variable' <- register`, `lend 'variable' -> register`, or `clobber register`, found {other}")),
                                }
                                if *self.peek() == Tok::Comma { self.advance(); continue; }
                                break;
                            }
                        }
                        self.expect(Tok::RBracket, "`]` to close the `with` list")?;
                    }
                    self.expect(Tok::Semi, "`;` after the asm block")?;
                    Ok(Stmt::Asm { text, binds, line })
                }
                _ => {
                    let mut call = self.call()?;
                    call.handler = self.handler()?;
                    self.expect(Tok::Semi, "`;` after the call")?;
                    Ok(Stmt::Call { call, line })
                }
            },
            Tok::LBrace => Ok(Stmt::Block { body: self.block()?, line }),
            Tok::NoShadow => self.err("a declaration starts with `let`: `let !$ 'x' := …;`"),
            Tok::Strict => self.err("`#strict` goes once, at the top of the file, before anything else"),
            Tok::Var(name) => {
                // a declaration says `let` (owner, 2026-10-02; from 2026-09-25 to then it started with its name)
                if matches!(self.peek_at(1), Tok::Walrus | Tok::ShadowWalrus | Tok::Colon | Tok::Semi) {
                    return self.err(format!("a declaration starts with `let`: `let '{name}' := …;` (or `let '{name}': T = …;`)"));
                }
                self.advance();
                if *self.peek() == Tok::LBracket && matches!(self.peek_at(1), Tok::Word(w) if w == "pos") {
                    return self.err(format!("'{name}'[…] on its own reads an item and drops it; to change it, `change '{name}'[…] = …;`"));
                }
                if *self.peek() == Tok::LBracket {
                    // 'f'[args];  — a function value called as a statement (2026-09-21); a list read
                    // on its own line (`'xs'[2];`) is refused when it runs, since it does nothing
                    let args = self.bracket()?;
                    let handler = self.handler()?;
                    self.expect(Tok::Semi, "`;` after the call")?;
                    return Ok(Stmt::CallValue { name, args, handler, line });
                }
                match self.peek() {
                    Tok::Word(w) if w == "mut" || w == "immut" => return self.err(format!("`{w}` goes before the name: `let {w} '{name}' := …;`")),
                    Tok::LBracket | Tok::Dot | Tok::Eq | Tok::ShadowEq => return self.err(format!("changing a variable says so: `change '{name}' = …;` (a declaration is `let '{name}' := …;`)")),
                    Tok::Op(o) if o == "++" => return self.err(format!("`++` joins text; to add one write `change '{name}' = '{name}' + 1;`")),
                    other => return self.err(format!("expected `:=` to declare '{name}', `[…]` to call it, or `change '{name}' = …;` to change it; found {other}")),
                }
            }
            other => self.err(format!("expected a statement, found {other}")),
        }
    }

    // change := 'change' Var ( '[' pos … ']' | '.' field )? '=' rhs ';'   (owner, 2026-09-25)
    fn change_stmt(&mut self, line: usize) -> PResult<Stmt> {
        match self.peek().clone() {
            Tok::Var(name) => {
                self.advance();
                if *self.peek() == Tok::LBracket {
                    // 'xs'[pos i] = v;  (decided 2026-09-17)
                    let idx = self.index_bracket(&name)?;
                    self.expect(Tok::Eq, "`=` after the element")?;
                    let value = self.rhs()?;
                    self.expect(Tok::Semi, "`;`")?;
                    return Ok(Stmt::AssignElem { name, index: idx, value, line });
                }
                if *self.peek() == Tok::Dot {
                    // 'mine'.field = v;  (decided 2026-09-18)
                    self.advance();
                    let field = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a field name after `'{name}'.`, found {other}")) };
                    if *self.peek() == Tok::LBracket { return self.err(format!("assigning into an element of a field (`'{name}'.{field}[pos …] = …`) is not modelled by the oracle; lend the list to a name first: `let 'xs' := lend.mut '{name}'.{field};`")); }
                    self.expect(Tok::Eq, "`=` after the field")?;
                    let value = self.rhs()?;
                    self.expect(Tok::Semi, "`;`")?;
                    return Ok(Stmt::AssignField { name, field, value, line });
                }
                match self.peek() {
                    Tok::Eq => {}
                    Tok::Walrus | Tok::ShadowWalrus | Tok::ShadowEq | Tok::Colon => return self.err(format!("`change` changes what exists, with `=`: `change '{name}' = …;` (a new '{name}' is `let '{name}' := …;`, a shadow `let '{name}' $:= …;`)")),
                    other => return self.err(format!("expected `=` after `change '{name}'`, found {other}")),
                }
                self.advance();
                let value = self.rhs()?;
                self.expect(Tok::Semi, "`;`")?;
                Ok(Stmt::Assign { name, value, line })
            }
            other => self.err(format!("`change` is followed by the variable it changes: `change 'x' = …;`, found {other}")),
        }
    }

    // decl := 'let' '!$'? ('mut' | 'immut')? Var (':' type)? (':=' | '=' | '$:=' | '$=') rhs ';'   (`let` back, owner 2026-10-02); `let` already read
    fn decl(&mut self) -> PResult<Stmt> {
        let line = self.line();
        let sealed = if *self.peek() == Tok::NoShadow { self.advance(); true } else { false };
        let mut mutability = None;
        if self.is_word("mut") { self.advance(); mutability = Some(Mutability::Mut); }
        else if self.is_word("immut") { self.advance(); mutability = Some(Mutability::Immut); }
        if self.strict && mutability.is_none() {
            let n = match self.peek() { Tok::Var(v) => v.clone(), _ => "x".into() };
            return self.err(format!("`#strict`: a declaration says whether it changes: `let mut '{n}' := …` or `let !mut '{n}' := …`"));
        }
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            Tok::Word(w) if w == "mut" || w == "immut" => return self.err("say `mut` or `!mut` once, before the name"),
            Tok::Word(w) if w == "let" => return self.err("say `let` once: `let 'x' := …;`"),
            Tok::Word(w) => return self.err(format!("variable names are quoted: `'{w}'` (a type goes after the name: `'x': {w}`)")),
            other => return self.err(format!("expected a quoted variable name, found {other}")),
        };
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
        let shadow;
        match (self.peek().clone(), &ty) {
            (Tok::Walrus, None) => { self.advance(); shadow = false; }
            (Tok::Eq, Some(_)) => { self.advance(); shadow = false; }
            (Tok::ShadowWalrus, None) => { self.advance(); shadow = true; }
            (Tok::ShadowEq, Some(_)) => { self.advance(); shadow = true; }
            (Tok::Walrus | Tok::ShadowWalrus, Some(_)) => return self.err("`:=` means Precog chooses the type; with an explicit type write `=` (or `$=` to shadow)"),
            (Tok::Eq | Tok::ShadowEq, None) => return self.err("`=` means you said everything; with no type write `:=` (or `$:=` to shadow)"),
            (Tok::Semi, _) => return self.err(format!("'{name}' needs a value; there are no implicit zero values")),
            (other, _) => return self.err(format!("expected `:=` or `=`, found {other}")),
        }
        let value = self.rhs()?;
        if let ([Value::Expr(Expr::OpValue(op, ..))], None) = (value.values.as_slice(), &ty) {
            // design claude's reading, not ruled: nothing says which `+` (int32? bin64? a Vec?)
            return self.err(format!("`op {op}` alone does not say which `{op}`: give '{name}' a function type, `'{name}': fn[int32, int32] -> int32 = op {op};`"));
        }
        self.expect(Tok::Semi, "`;`")?;
        Ok(Stmt::Decl { name, mutability, ty, value, shadow, sealed, line })
    }

    // if := 'if' expr block ('else' 'if' expr block)* ('else' block)?
    /// One test of an `on` chain (owner, 2026-10-06): `on.target aarch64-macos`, `on.target.chip
    /// aarch64`, `on.target.os macos` are decided when compiling, against the target built for (the
    /// oracle's is `aarch64-macos`); `on.machine.chip sve` when running, on the actual machine, and it is
    /// IEG's. Returns whether it is `on.machine`, and whether it holds.
    fn on_test(&mut self) -> PResult<(bool, bool)> {
        self.expect_word("on")?;
        if *self.peek() != Tok::Dot { return self.err("`on` is narrowed: `on.target …` (chosen when compiling) or `on.machine …` (when running)"); }
        self.advance();
        let which = match self.advance().tok { Tok::Word(w) if w == "target" || w == "machine" => w, other => return self.err(format!("`on.target` or `on.machine`, found {other}")) };
        let narrow = if *self.peek() == Tok::Dot {
            self.advance();
            match self.advance().tok { Tok::Word(w) if w == "chip" || w == "os" => Some(w), other => return self.err(format!("`on.{which}` narrows to `.chip` or `.os`, found {other}")) }
        } else { None };
        let word = |p: &mut Self, what: &str| -> PResult<String> {
            match p.advance().tok { Tok::Word(w) => Ok(w), other => p.err(format!("expected {what}, found {other}")) }
        };
        if which == "machine" {
            if self.lacks.iter().any(|c| c == "IEG") { return self.core_missing("IEG", "`on.machine`"); }
            if narrow.as_deref() != Some("chip") { return self.err("`on.machine` asks the chip what it can do: `on.machine.chip sve { … }`"); }
            let f = word(self, "a chip or a feature (`aarch64`, `sve`, `neon`, …)")?;
            return Ok((true, machine_has(&f).map_err(|m| ParseError { line: self.line(), col: 0, msg: m })?));
        }
        const CHIPS: &[&str] = &["aarch64", "x86_64", "x86", "arm", "riscv64", "riscv32", "wasm32", "wasm64", "powerpc64", "powerpc64le", "s390x", "mips64", "loongarch64"];
        const OSES: &[&str] = &["macos", "ios", "linux", "android", "windows", "freebsd", "netbsd", "openbsd", "wasi", "none"];
        let (chip, os) = ORACLE_TARGET.split_once('-').unwrap();
        let held = match narrow.as_deref() {
            Some("chip") => {
                let c = word(self, "a chip (`aarch64`, `x86_64`, …)")?;
                if !CHIPS.contains(&c.as_str()) { return self.err(format!("`{c}` is not a chip `on.target.chip` knows: {}", CHIPS.join(", "))); }
                if matches!(self.peek(), Tok::Word(_)) { return self.err(format!("a chip's minimum (`on.target.chip {c} sve`) is not built in the oracle: how a build names the chip features it needs is not decided; `on.machine.chip` checks when running")); }
                c == chip
            }
            Some(_) => {
                let o = word(self, "an OS (`macos`, `linux`, `windows`, …)")?;
                if !OSES.contains(&o.as_str()) { return self.err(format!("`{o}` is not an OS `on.target.os` knows: {}", OSES.join(", "))); }
                if matches!(self.peek(), Tok::Int(_) | Tok::Bin(..)) { return self.err(format!("an OS minimum (`on.target.os {o} 15`) is not built in the oracle: how a build names the oldest version it runs on is not decided")); }
                o == os
            }
            None => {
                let mut t = word(self, "a target (`aarch64-macos`)")?;
                while *self.peek() == Tok::Op("-".into()) { self.advance(); t.push('-'); t.push_str(&word(self, "the rest of the target")?); }
                let Some((c, o)) = t.split_once('-') else { return self.err(format!("`{t}` is a chip; a whole target is chip-os (`aarch64-macos`), or narrow: `on.target.chip {t}`")) };
                if !CHIPS.contains(&c) || !OSES.contains(&o) { return self.err(format!("`{t}` is not a target: chip-os, from {} and {}", CHIPS.join(", "), OSES.join(", "))); }
                t == ORACLE_TARGET
            }
        };
        Ok((false, held))
    }

    /// `on.target … { } else on.machine … { } else { }` as a statement (owner, 2026-10-06): a branch
    /// `on.target` rules out is not compiled at all; `on.machine` branches are, and one runs
    fn on_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        let mut arms: Vec<(bool, bool, Vec<Stmt>)> = Vec::new();
        let (m, h) = self.on_test()?;
        arms.push((m, h, self.block()?));
        let mut otherwise = None;
        while self.is_word("else") {
            self.advance();
            if self.is_word("on") { let (m, h) = self.on_test()?; arms.push((m, h, self.block()?)); }
            else if self.is_word("if") { return self.err("an `on` chain goes on with `else on.…` or ends with `else { }`; an `if` goes inside a branch"); }
            else { otherwise = Some(self.block()?); break; }
        }
        if !arms.iter().any(|(m, ..)| *m) {
            let body = arms.into_iter().find(|(_, h, _)| *h).map(|(_, _, b)| b).or(otherwise).unwrap_or_default();
            return Ok(Stmt::Block { body, line });
        }
        let arms: Vec<(Bracket, Vec<Stmt>)> = arms.into_iter().filter(|(m, h, _)| *m || *h)
            .map(|(_, h, b)| (Bracket { values: vec![Value::Expr(Expr::Bool(h, line))], line, one_list: false }, b)).collect();
        Ok(Stmt::If { arms, otherwise, line })
    }

    fn if_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("if")?;
        let mut arms = vec![(self.cond()?, self.block()?)];
        let mut otherwise = None;
        while self.is_word("else") {
            self.advance();
            if *self.peek() == Tok::Dot {
                return self.err("`else.if` became `else if` (2026-09-19)");
            } else if self.is_word("if") {
                self.advance();
                arms.push((self.cond()?, self.block()?));
            } else {
                otherwise = Some(self.block()?);
                break;
            }
        }
        Ok(Stmt::If { arms, otherwise, line })
    }

    // A condition is a bare expression followed by `{`; `[ ]` around it is
    // accepted as grouping but builds nothing.
    fn cond(&mut self) -> PResult<Bracket> {
        let b = self.rhs()?;
        if *self.peek() != Tok::LBrace { return self.err(format!("expected `{{` after the condition, found {}", self.peek())); }
        Ok(b)
    }

    // loop := 'loop' block
    fn loop_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("loop")?;
        if *self.peek() == Tok::Dot {
            self.advance();
            return match self.peek().clone() {
                Tok::Word(w) if w == "while" => self.err("`loop.while` became `while` (2026-09-19): `while 'i' < 3 { }`"),
                Tok::Word(w) if w == "temp" || w == "perm" || w == "for" || w == "range" => self.err("`loop.temp.for` / `loop.temp.range` became `for` (2026-09-19): `for 'x' in lend 'xs' { }`, `for 'i' in range[1, 10] { }`"),
                other => self.err(format!("`loop` takes no chain; loops are `loop`, `while`, and `for` (found `.{other}`)")),
            };
        }
        Ok(Stmt::Loop { body: self.block()?, line })
    }

    // while := 'while' expr block
    fn while_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("while")?;
        Ok(Stmt::While { cond: self.cond()?, body: self.block()?, line })
    }

    // for := 'for' ('seq')? ('temp' | 'perm')? Var (':' type)? 'in' ('range' bounds | rhs) block
    // `temp` is the default (decided 2026-09-19); `perm` keeps the variable after the loop;
    // modifiers are bare words before the name, `seq` first.
    fn for_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("for")?;
        if *self.peek() == Tok::Dot { return self.err("`for` takes no chain: `for 'x' in …`, `for seq perm 'i' in …`"); }
        // `for !cheat` — "no cheat" (owner, 2026-10-01; `!c` until 2026-10-06): every round's work really happens; the compiler
        // may not replace the loop with its result or drop rounds, though it may still use SIMD or
        // cores. The strongest modifier comes first, so `!cheat` precedes `seq` / `para`. The oracle
        // runs every round of every loop already, so it only reads the word.
        if self.is_word("!c") { return self.err("`!c` is `!cheat` now: `for !cheat 'i' in …` (owner, 2026-10-06)"); }
        if self.is_word("!cheat") { self.advance(); }
        let mut seq = false;
        if self.is_word("seq") { self.advance(); seq = true; }
        // `for para` forces a loop parallel, as `for seq` forces it in order (owner, 2026-09-30); the
        // oracle runs every loop in order, one schedule of the design's, so it only reads the word
        else if self.is_word("para") { self.advance(); }
        if self.is_word("seq") || self.is_word("para") { return self.err("a `for` is `seq` or `para`, not both"); }
        if self.is_word("!cheat") || self.is_word("!c") { return self.err("`!cheat` comes first, the strongest modifier: `for !cheat seq 'i' in …`"); }
        let perm = match self.peek().clone() {
            Tok::Word(w) if w == "temp" => { self.advance(); false }
            Tok::Word(w) if w == "perm" => { self.advance(); true }
            Tok::Word(w) if w == "seq" || w == "para" => return self.err(format!("`{w}` comes first: `for {w} perm 'x' in …`")),
            _ if self.strict => return self.err("`#strict`: a `for` says whether its variable outlives it: `for temp 'x' in …` or `for perm 'x' in …`"),
            _ => false,
        };
        let mut name = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a quoted loop variable, found {other}")) };
        // `for 'i', 'x' in lend 'xs'` — the position first, then the item (owner, 2026-09-29)
        let mut index = None;
        if *self.peek() == Tok::Comma {
            self.advance();
            let item = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected the item's quoted name after the position's: `for 'i', 'x' in …`, found {other}")) };
            if item == name { return self.err("the position and the item need different names"); }
            index = Some(std::mem::replace(&mut name, item));
        }
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
        if *self.peek() == Tok::LBracket { return self.err("expected `in` before what the loop runs over"); }
        self.expect_word("in")?;
        if self.is_word("range") {
            self.advance();
            if index.is_some() { return self.err("a range's counter already is the position; `for 'i' in range[…]`"); }
            let bounds = self.range_bounds()?;
            let body = self.block()?;
            return Ok(Stmt::Range { name, perm, seq, ty, bounds, body, line });
        }
        if ty.is_some() { return self.err("a `for` over a list takes no type on its variable; the element's type is the list's"); }
        let over = self.rhs()?;
        let body = self.block()?;
        Ok(Stmt::For { name, index, perm, seq, over, body, line })
    }

    // range bounds := '[' expr ',' expr (',' expr)? ']'   — start, inclusive stop, step
    fn range_bounds(&mut self) -> PResult<Bracket> {
        let line = self.line();
        if *self.peek() != Tok::LBracket { return self.err("`range` builds its bounds in `[ ]`: `range[1, 10]` or `range[1, 10, 2]`"); }
        self.advance();
        let mut values = Vec::new();
        // In a `#strict` file each bound says what it is — `range[from 2, to 8, by 2]` — and `by`
        // is always written (owner, 2026-09-27); elsewhere the labels are not used.
        const LABELS: [&str; 3] = ["from", "to", "by"];
        loop {
            if matches!(self.peek(), Tok::Walrus | Tok::Eq) {
                return self.err("range bounds are plain values: `range[0, 9]` — the counter's type goes on its name (`'i': int32`)");
            }
            let k = values.len().min(2);
            let labelled = matches!(self.peek(), Tok::Word(w) if LABELS.contains(&w.as_str())) && !matches!(self.peek_at(1), Tok::Dot | Tok::LBracket | Tok::Comma | Tok::RBracket);
            if self.strict {
                if !self.is_word(LABELS[k]) { return self.err(format!("`#strict`: each bound says what it is: `range[from 2, to 8, by 1]` — this one is `{}`", LABELS[k])); }
                self.advance();
            } else if labelled {
                return self.err("`from` / `to` / `by` label a range in a `#strict` file; here it is `range[2, 8]` or `range[2, 8, 2]`");
            }
            values.push(Value::Expr(self.expr()?));
            if *self.peek() == Tok::Comma { self.advance(); continue; }
            break;
        }
        self.expect(Tok::RBracket, "`]`")?;
        if self.strict && values.len() == 2 { return Err(ParseError { line, col: 0, msg: "`#strict`: a range writes its step too, even when it is 1: `range[from 2, to 8, by 1]`".into() }); }
        if values.len() < 2 || values.len() > 3 {
            return Err(ParseError { line, col: 0, msg: format!("a range is `range[start, stop]` or `range[start, stop, step]`, not {} value(s)", values.len()) });
        }
        Ok(Bracket { values, line, one_list: false })
    }

    // rhs := bracket | expr
    // Decided 2026-09-19: `[ ]` is written only where it builds a value (a text
    // join, a list, a call's arguments…). A single expression stands bare.
    fn rhs(&mut self) -> PResult<Bracket> {
        let line = self.line();
        if *self.peek() == Tok::LBracket { return self.bracket(); }
        let mut e = self.expr()?;
        if self.starts_primary() {
            return self.err(format!("two values next to each other (found {} after the value): join text with `++`", self.peek()));
        }
        // `call[…] otherwise …` — only when the call is the whole right-hand side (decided 2026-09-19)
        if self.is_word("otherwise") {
            match &mut e {
                Expr::Call(c) => c.handler = self.handler()?,
                Expr::Convert(_, _, h, _, _) => *h = self.handler()?,
                Expr::CharFromCode(_, _, h, _) => *h = self.handler()?,
                Expr::MapFail(_, _, h, _) => *h = self.handler()?,
                Expr::CallValue(_, _, h, _) => *h = self.handler()?,
                Expr::Builtin(hd, op, b, l) if hd == "ptr" && op == "read" => { let (b, l) = (b.clone(), *l); e = Expr::PtrReadChecked(b, self.handler()?, l); }
                _ => return self.err("`otherwise` handles a call that can fail, and the call must be the whole value: declare it on its own line first (`'v' := f[…] otherwise …;`)"),
            }
        }
        Ok(Bracket { values: vec![Value::Expr(e)], line, one_list: false })
    }

    // handler := 'otherwise' ( 'fail' ';'-ahead | leaving-stmt | '{' arm* '}' | expr )
    fn handler(&mut self) -> PResult<Option<Handler>> {
        if !self.is_word("otherwise") { return Ok(None); }
        self.advance();
        let h = match self.peek().clone() {
            Tok::Word(w) if w == "fail" && *self.peek_at(1) == Tok::Semi => { self.advance(); Handler::Propagate }
            Tok::Word(w) if w == "fail" || w == "return" || w == "break" || w == "continue"
                || (w == "std" && *self.peek_at(1) == Tok::ColonColon && matches!(self.peek_at(2), Tok::Word(e) if e == "exit")) => {
                // a leaving statement, `;` included; the outer statement's `;` is this one
                let s = self.stmt()?;
                self.pos -= 1; // hand the `;` back to the enclosing statement
                Handler::Leave(Box::new(s))
            }
            Tok::LBrace => {
                let arms = self.arms("otherwise")?;
                if arms.is_empty() { return self.err("`otherwise { }` handles nothing; list the failures, or use `otherwise fail;`"); }
                Handler::Arms(arms)
            }
            Tok::Semi => return self.err("`otherwise` needs what to do: a value, `fail`, `return …`, or `{ arms }`"),
            _ => {
                let line = self.line();
                let e = self.expr()?;
                Handler::Value(Bracket { values: vec![Value::Expr(e)], line, one_list: false })
            }
        };
        Ok(Some(h))
    }

    /// A range type's bound: a numeric literal, possibly negated.
    fn bound(&mut self) -> PResult<Bound> {
        // `min` / `max` are the type's own limits: `int32[1, max]` is "at least 1"
        // (owner, 2026-09-23), so a one-ended range never writes a number that goes
        // stale when the width changes.
        if let Tok::Word(w) = self.peek().clone() {
            if w == "min" { self.advance(); return Ok(Bound::Min); }
            if w == "max" { self.advance(); return Ok(Bound::Max); }
        }
        let neg = if matches!(self.peek(), Tok::Op(o) if o == "-") { self.advance(); true } else { false };
        match self.advance().tok {
            Tok::Int(i) => Ok(Bound::Int(if neg { -i } else { i })),
            Tok::Bin(b, _) => Ok(Bound::Bin(if neg { -b } else { b })),
            other => self.err(format!("a range's bounds are numeric literals, or `min` / `max` for the type's own limit, found {other}")),
        }
    }

    /// `[ … ][` — in `to.int32[0, 100]['text']` the first bracket is the range, the second the value.
    fn bracket_then_bracket(&self) -> bool {
        let mut i = 0; let mut depth = 0;
        loop {
            match self.peek_at(i) {
                Tok::LBracket => depth += 1,
                Tok::RBracket => { depth -= 1; if depth == 0 { return *self.peek_at(i + 1) == Tok::LBracket; } }
                Tok::Eof => return false,
                _ => {}
            }
            i += 1;
        }
    }

    /// A lone Word (not a keyword, no bracket, no path) in value position — a function as a value.
    /// A dotted path without a bracket is an enum case (`is_enum_bare`) unless it ends in a
    /// lowercase segment after an uppercase one… the oracle resolves both at run time, so a
    /// single bare word is treated as a function reference and a dotted one as an enum case.
    fn is_fn_ref(&self) -> bool {
        matches!(self.peek(), Tok::Word(w) if !KEYWORDS.contains(&w.as_str()) && !OP_WORDS.contains(&w.as_str()))
            && !matches!(self.peek_at(1), Tok::LBracket | Tok::ColonColon | Tok::Dot)
    }

    /// The body of an anonymous function: `fn [params] -> T with [captures] { … }`
    fn anon_func(&mut self) -> PResult<Func> {
        let line = self.line();
        self.expect_word("fn")?;
        self.expect(Tok::LBracket, "`[` to open the parameter list")?;
        let mut params = Vec::new();
        if *self.peek() != Tok::RBracket {
            loop { params.push(self.param()?); if *self.peek() == Tok::Comma { self.advance(); continue; } break; }
        }
        self.expect(Tok::RBracket, "`]` to close the parameter list")?;
        let mut ret = None; let mut fails = Vec::new();
        if *self.peek() == Tok::Arrow {
            self.advance();
            if self.is_word("fail") { fails = self.fail_list()?; }
            else { ret = Some(self.ty()?); if self.is_word("or") { self.advance(); if !self.is_word("fail") { return self.err("after `or` comes `fail[…]`"); } fails = self.fail_list()?; } }
        }
        let caps = self.captures()?;
        if self.strict {
            if let Some(p) = params.iter().find(|p| p.ty.is_none()) { return self.err(format!("`#strict`: parameter '{}' says its type", p.name)); }
        }
        let saved_ret = self.saw_value_return;
        self.saw_value_return = false;
        let body = self.block()?;
        if self.strict && ret.is_none() && self.saw_value_return { return self.err("`#strict`: this function returns a value, so it says its type: `-> int32`"); }
        self.saw_value_return = saved_ret;
        self.pending_captures = caps;
        let bounds = std::mem::take(&mut self.bounds);
        Ok(Func { name: String::new(), module: self.module.clone(), vis: Vis::Private, ret, ret_lent: None, fails, is_extern: false, is_op: false, is_test: false, bounds, params, body, line })
    }

    /// `with [lend 'a', give 'b']` — explicit captures, for an anonymous fn and for `concur`.
    fn captures(&mut self) -> PResult<Vec<(Verb, String)>> {
        // captures: nothing is taken from the surrounding scope unless listed here, with a verb
        let mut caps = Vec::new();
        if self.is_word("with") {
            self.advance();
            self.expect(Tok::LBracket, "`[` after `with`: `with [lend 'limit', give 'name']`")?;
            if *self.peek() != Tok::RBracket {
                loop {
                    let verb = match self.advance().tok {
                        Tok::Word(w) if w == "lend" => { if *self.peek() == Tok::Dot { self.advance(); match self.advance().tok { Tok::Word(m) if m == "mut" => Verb::LendMut, other => return self.err(format!("the only narrowing of `lend` is `.mut`, found {other}")) } } else { Verb::Lend } }
                        Tok::Word(w) if w == "give" => Verb::Give,
                        Tok::Word(w) if w == "copy" => Verb::Copy,
                        Tok::Var(v) => return self.err(format!("a capture says how it is taken: `lend '{v}'`, `copy '{v}'`, or `give '{v}'` — nothing is captured silently")),
                        other => return self.err(format!("expected `lend`, `copy`, or `give` before a captured variable, found {other}")),
                    };
                    let v = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a quoted variable to capture, found {other}")) };
                    caps.push((verb, v));
                    if *self.peek() == Tok::Comma { self.advance(); continue; }
                    break;
                }
            }
            self.expect(Tok::RBracket, "`]` to close the captures")?;
        }
        Ok(caps)
    }

    /// Word ('.' Word)+ not followed by '[' or '::'  — an enum case with no data
    fn is_enum_bare(&self) -> bool {
        let mut i = 0;
        let mut dots = 0;
        loop {
            if !matches!(self.peek_at(i), Tok::Word(_)) { return false; }
            match self.peek_at(i + 1) {
                Tok::Dot if matches!(self.peek_at(i + 2), Tok::Word(_)) => { i += 2; dots += 1; }
                Tok::Dot | Tok::LBracket | Tok::ColonColon => return false,   // a const path, a call, a Limb
                _ => return dots >= 1,
            }
        }
    }

    /// Word ('.' Word)* '.' Var  — a const reached by path
    fn is_const_path(&self) -> bool {
        let mut i = 0;
        loop {
            if !matches!(self.peek_at(i), Tok::Word(_)) { return false; }
            if *self.peek_at(i + 1) != Tok::Dot { return false; }
            match self.peek_at(i + 2) { Tok::Var(_) => return true, Tok::Word(_) => i += 2, _ => return false }
        }
    }

    /// Word ('.' Word)* '[' (Word | Var) ('=' | ':=' | ':')  — a struct construction
    /// (a Var here is the pre-2026-09-19 spelling, kept so the error can say so)
    /// The fields of a struct value, after its name and `[`: `radius = 2.0, …]`.
    fn construct_body(&mut self, w: String, line: usize) -> PResult<Expr> {
        let mut fields = Vec::new();
        loop {
            let fname = match self.advance().tok {
                Tok::Word(w) => w,
                Tok::Var(v) => return self.err(format!("a field's name is bare: `{v} = …` (decided 2026-09-19)")),
                other => return self.err(format!("expected a field name, found {other}")),
            };
            match self.peek() {
                Tok::Eq => { self.advance(); }
                Tok::Walrus => return self.err(format!("a field's type is fixed by the struct: `{fname} = …`, not `:=`")),
                Tok::Colon => return self.err(format!("a field is given by name: `{fname} = value` (its type is fixed by the struct)")),
                other => return self.err(format!("expected `=` after `{fname}`, found {other}")),
            }
            // one value; pieces join, `[ ]` builds a list (or just groups)
            let vline = self.line();
            let value = Bracket { values: vec![self.value()?], line: vline, one_list: false };
            fields.push((fname, value));
            if *self.peek() == Tok::Comma { self.advance(); continue; }
            break;
        }
        self.expect(Tok::RBracket, "`]` to close the struct value")?;
        Ok(Expr::Construct(w, fields, line))
    }

    /// `geometry::Circle[radius = …]` or `geometry::'pi'` (not `std::`, whose calls go through `call`)
    fn is_limb_item(&self) -> bool {
        if !matches!(self.peek(), Tok::Word(w) if w != "std") || *self.peek_at(1) != Tok::ColonColon { return false; }
        let mut i = 2;
        loop {
            match self.peek_at(i) {
                Tok::Var(_) => return true,
                Tok::Word(_) => {}
                _ => return false,
            }
            match self.peek_at(i + 1) {
                Tok::Dot => { i += 2; }
                Tok::LBracket => return matches!(self.peek_at(i + 2), Tok::Word(_)) && matches!(self.peek_at(i + 3), Tok::Eq),
                _ => return false,
            }
        }
    }

    fn is_construct(&self) -> bool {
        let mut i = 0;
        loop {
            if !matches!(self.peek_at(i), Tok::Word(_)) { return false; }
            match self.peek_at(i + 1) {
                Tok::Dot => { i += 2; }
                Tok::LBracket => return matches!(self.peek_at(i + 2), Tok::Var(_) | Tok::Word(_)) && matches!(self.peek_at(i + 3), Tok::Eq | Tok::Walrus | Tok::Colon),
                _ => return false,
            }
        }
    }

    // arms := '{' (Word Var? block)* '}'   — shared by `otherwise` and `match`
    fn arms(&mut self, what: &str) -> PResult<Vec<Arm>> {
        self.expect(Tok::LBrace, "`{`")?;
        let mut arms = Vec::new();
        while *self.peek() != Tok::RBrace {
            let line = self.line();
            let name = match self.advance().tok {
                Tok::Word(w) => w,
                Tok::Var(v) => return self.err(format!("an arm starts with the case's bare name; the data's name comes after it: `claimed {v} {{ … }}`")),
                Tok::Eof => return self.err(format!("unclosed `{what} {{`")),
                other => return self.err(format!("expected a case name to handle, found {other}")),
            };
            let bind = if let Tok::Var(v) = self.peek().clone() { self.advance(); Some(v) } else { None };
            // `{ "free" }` — a block holding only a value is that value (owner, 2026-09-29: a); in a
            // `#strict` file the value is said with `giveS` instead (b)
            let mut value = None;
            if *self.peek() == Tok::LBrace && *self.peek_at(1) != Tok::RBrace {
                let save = self.pos;
                self.advance();
                let try_value = self.expr();
                if matches!(try_value, Ok(_)) && *self.peek() == Tok::RBrace {
                    if self.strict {
                        // a `match` alone in a strict case is a statement: a value would say `giveS` (owner, 2026-09-29)
                        if !matches!(try_value, Ok(Expr::Match(..))) { return self.err(format!("`#strict`: a case says its value with `giveS`: `{name} {{ giveS …; }}`")); }
                        self.pos = save;
                    } else {
                        self.advance();
                        value = try_value.ok();
                    }
                } else {
                    self.pos = save;
                }
            }
            let body = if value.is_some() { Vec::new() } else {
                self.arm_depth += 1;
                let b = self.block();
                self.arm_depth -= 1;
                b?
            };
            if arms.iter().any(|a: &Arm| a.name == name) { return self.err(format!("`{name}` is handled twice")); }
            arms.push(Arm { name, bind, value, body, line });
        }
        self.advance();
        Ok(arms)
    }

    // call := (Word '::')? Word ('.' Word)* bracket
    fn call(&mut self) -> PResult<Call> {
        let line = self.line();
        let first = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a name, found {other}")) };
        if KEYWORDS.contains(&first.as_str()) && first != "my" && *self.peek() != Tok::ColonColon {
            return self.err(format!("`{first}` is a keyword; a function named `{first}` is reached through a path, never bare: `my.{first}[…]`"));
        }
        let (limb, mut path) = if *self.peek() == Tok::ColonColon {
            self.advance();
            let seg = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a name after `::`, found {other}")) };
            (Some(first), vec![seg])
        } else {
            (None, vec![first])
        };
        while *self.peek() == Tok::Dot {
            self.advance();
            match self.advance().tok { Tok::Word(w) => path.push(w), other => return self.err(format!("expected a name after `.`, found {other}")) }
        }
        // written with no path at all: another Limb's bare name may be meant (owner, 2026-10-06)
        let bare = limb.is_none() && path[0] != "my";
        let written = if path[0] == "my" { path[1..].to_vec() } else { path.clone() };
        // std's names come bare too (owner, 2026-10-06: std is "optional for all"), where std is listed
        let limb = if bare && !self.no_std && (STD_BARE_HEADS.contains(&path[0].as_str()) || (path[0] == "str" && path.get(1).map_or(false, |o| STD_STR_OPS.contains(&o.as_str())))) { Some("std".to_string()) } else { limb };
        // `std::print.weird.stored.stdout <- "n is " <- 'n' <- \n;` — C++'s `cout <<`, with CyborgPL's
        // arrow pointing where the values go (owner, 2026-10-01): values follow, one per `<-`
        if limb.as_deref() == Some("std") && path.first().map(|s| s.as_str()) == Some("print") && path.iter().skip(2).any(|w| w == "weird") {
            return self.err("`.weird` comes first, the strongest modifier: `std::print.weird.useful.stdout <- …`");
        }
        if limb.as_deref() == Some("std") && path.first().map(|s| s.as_str()) == Some("print") && path.get(1).map(|s| s.as_str()) == Some("weird") {
            if *self.peek() == Tok::LBracket { return self.err(format!("`std::{}` takes its values with arrows, not `[…]`: `std::{} <- \"text\" <- 'n' <- \\n;`", path.join("."), path.join("."))); }
            let bline = self.line();
            let mut values = Vec::new();
            while *self.peek() == Tok::LeftArrow { self.advance(); values.push(self.value()?); }
            if values.is_empty() { return self.err(format!("`std::{}` needs at least one value: `std::{} <- \"text\";`", path.join("."), path.join("."))); }
            return Ok(Call { handler: None, limb, path, from: self.module.clone(), args: Bracket { values, line: bline, one_list: false }, line, bare, written });
        }
        if *self.peek() != Tok::LBracket {
            let shown = match &limb { Some(l) => format!("{l}::{}", path.join(".")), None => path.join(".") };
            if limb.is_none() && path.len() >= 2 { return self.err(format!("`{shown}`: a call needs `[…]`; an enum case with no data is written bare (`Status.empty`) — is `{}` an enum?", path[path.len() - 2])); }
            // `lang:let` is the 2026-09-14 spelling: say that, not "a call needs […]"
            if limb.is_none() && shown == "lang" && *self.peek() == Tok::Colon { return self.err("`lang:` was removed (2026-09-14): keywords are bare — `let`, not `lang:let`"); }
            return self.err(format!("`{shown}` is followed by {}; a call needs `[…]`", self.peek()));
        }
        // `print.format` reads its format text as typed, with its own escapes, in one pass
        // (owner, 2026-10-01: "Format print has its own rules"): `\\` there is one backslash
        let raw_format = match (limb.as_deref(), path.first().map(|s| s.as_str()), path.get(1).map(|s| s.as_str()), self.peek_at(1)) {
            (Some("std"), Some("print"), Some("format"), Tok::Text(_, raw)) => Some(raw.clone()),
            _ => None,
        };
        let mut args = self.bracket()?;
        if let (Some(raw), Some(crate::ast::Value::Expr(Expr::Text(t, _)))) = (raw_format, args.values.first_mut()) { *t = raw; }
        let path = if limb.is_none() { self.resolve(&path) } else { path };
        if bare { BARE_CALLS.with(|b| b.borrow_mut().push((self.module.clone(), written.clone(), limb.as_deref() == Some("std"), line))); }
        if limb.is_none() { CALL_SITES.with(|c| c.borrow_mut().push((self.limb_root.clone(), path.clone(), line))); }
        Ok(Call { handler: None, limb, path, from: self.module.clone(), args, line, bare, written })
    }

    // index := '[' 'pos' expr ']'   — 1-based; `pos` is mandatory (decided 2026-09-17)
    fn index_bracket(&mut self, name: &str) -> PResult<Expr> {
        self.expect(Tok::LBracket, "`[`")?;
        if self.strict {
            if !self.is_word("pos") { return self.err(format!("`#strict`: an index says it is a position: `'{name}'[pos 2]`")); }
            self.advance();
        } else if self.is_word("pos") && !matches!(self.peek_at(1), Tok::LBracket | Tok::Dot | Tok::RBracket) { return self.err(format!("`pos` is gone (2026-09-27): a position is written bare, `'{name}'[2]` (`pos` belongs to `#strict` files)")); }
        let idx = self.expr()?;
        if *self.peek() == Tok::Comma { return self.err(format!("an index is one position: `'{name}'[3]`")); }
        self.expect(Tok::RBracket, "`]` after the position")?;
        Ok(idx)
    }

    // ptr op := 'ptr' '.' ('read' | 'write' | 'at' | 'is_null') bracket   (design/ffi.md)
    fn ptr_op(&mut self) -> PResult<(String, String, Bracket)> {
        self.expect_word("ptr")?;
        self.expect(Tok::Dot, "`.` after `ptr` (`ptr.read[…]`, `ptr.write[…]`, `ptr.at[…]`, `ptr.is_null[…]`)")?;
        let op = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected an operation after `ptr.`, found {other}")) };
        let want = match op.as_str() { "read" => 1, "write" => 2, "at" => 2, "is_null" => 1, "cast" => return self.err("`ptr.cast[T]['p']` is a value, the pointer seen as pointing at a T: `!check 'q' := ptr.cast[Point]['p'];`"), "mut" => return self.err("`ptr.mut[T]` is a type, for an extern signature; here use `ptr.read`, `ptr.write`, `ptr.at`, `ptr.is_null`, `ptr.cast`"), _ => return self.err(format!("`ptr.{op}` does not exist; ptr has read, write, at, is_null, cast")) };
        if *self.peek() != Tok::LBracket { return self.err(format!("`ptr.{op}` is a call and needs `[…]`")); }
        match op.as_str() { "read" | "at" => self.use_power("raw_memory.read", Some(format!("ptr.{op}")), self.line()), "write" => self.use_power("raw_memory.write", Some("ptr.write".into()), self.line()), _ => {} }
        let args = self.bracket()?;
        if args.values.len() != want { return self.err(format!("`ptr.{op}` takes {want} argument(s), given {}", args.values.len())); }
        Ok(("ptr".into(), op, args))
    }

    // builtin := ('list' | 'array') '.' Word bracket
    fn builtin(&mut self) -> PResult<(String, String, Bracket)> {
        let head = match self.advance().tok { Tok::Word(w) => w, _ => unreachable!() };
        self.expect(Tok::Dot, &format!("`.` after `{head}` (`{head}.length[…]`)"))?;
        let mut op = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected an operation after `{head}.`, found {other}")) };
        // `simd.load.some` / `simd.store.some` — only the first k lanes (owner, 2026-09-27: a)
        if head == "simd" && (op == "load" || op == "store") && *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(w) if w == "some") {
            self.advance(); self.advance(); op = format!("{op}.some");
        }
        // `simd.sum.lane` / `simd.sum.wrap`: the sum at the lane's type, checked or wrapped (owner, 2026-10-07: "yea")
        if head == "simd" && op == "sum" && *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(w) if w == "lane" || w == "wrap") {
            self.advance(); let Tok::Word(w) = self.advance().tok else { unreachable!() }; op = format!("sum.{w}");
        }
        let ok = match (head.as_str(), op.as_str()) {
            ("list", "length" | "append" | "remove" | "empty" | "slice" | "fill") | ("array", "length" | "slice") => true,
            ("map", "empty" | "length" | "has" | "set" | "get" | "remove" | "keys" | "values" | "pairs") => true,
            ("channel", "send" | "take") => true,
            ("simd", "fill" | "load" | "store" | "load.some" | "store.some" | "all" | "any" | "max" | "min" | "sum" | "sum.lane" | "sum.wrap" | "shuffle") => true,
            ("array", "empty") => return self.err("an empty array could never be filled; use `list.empty`"),
            _ => false,
        };
        if !ok { return self.err(format!("`{head}.{op}` does not exist; list has length/append/remove/empty/slice/fill, array has length/slice, map has empty/length/has/get/set/remove/keys/values/pairs, simd has fill/load/store/all/any/max/min/sum")); }
        if op == "empty" {
            // `[list.empty]` — a value, not a call (decided 2026-09-17)
            let line = self.line();
            return Ok((head, op, Bracket { values: Vec::new(), line, one_list: false }));
        }
        if *self.peek() != Tok::LBracket { return self.err(format!("`{head}.{op}` is a call and needs `[…]`")); }
        // `simd.load[lend 'xs', pos 'i']` — the position says `pos`, as every index does
        if (head == "list" || head == "array") && op == "slice" {
            // `list.slice[lend 'xs', pos 2, pos 4]` — positions say `pos`, as every index does
            let line = self.line();
            self.advance();
            let mut values = vec![self.value()?];
            for which in ["first", "last"] {
                self.expect(Tok::Comma, &format!("`,` before the {which} position"))?;
                if self.strict { if !self.is_word("pos") { return self.err(format!("`#strict`: positions say `pos`: `{head}.slice[lend 'xs', pos 2, pos 4]`")); } self.advance(); }
                else if self.is_word("pos") { return self.err(format!("`pos` is gone (2026-09-27): `{head}.slice[lend 'xs', 2, 4]` (`pos` belongs to `#strict` files)")); }
                values.push(self.value()?);
            }
            self.expect(Tok::RBracket, "`]`")?;
            return Ok((head, op, Bracket { values, line, one_list: false }));
        }
        // `list.fill[0, 10]`: ten copies of 0 (owner, 2026-09-30); a strict file labels the count,
        // `list.fill[0, count 10]` (owner: "count"), as a position says `pos`
        if head == "list" && op == "fill" {
            let line = self.line();
            self.advance();
            let mut values = vec![self.value()?];
            self.expect(Tok::Comma, "`,` before the count: `list.fill[0, 10]`")?;
            if self.strict { if !self.is_word("count") { return self.err("`#strict`: the count says `count`: `list.fill[0, count 10]`"); } self.advance(); }
            else if self.is_word("count") { return self.err("`count` belongs to `#strict` files: `list.fill[0, 10]`"); }
            values.push(self.value()?);
            self.expect(Tok::RBracket, "`]` after the count: `list.fill[0, 10]`")?;
            return Ok((head, op, Bracket { values, line, one_list: false }));
        }
        if head == "simd" && op.starts_with("load") || head == "simd" && op.starts_with("store") {
            let line = self.line();
            self.advance();
            let mut values = vec![self.value()?];
            self.expect(Tok::Comma, "`,` after the list")?;
            if self.strict { if !self.is_word("pos") { return self.err(format!("`#strict`: the position says `pos`: `simd.{op}[lend 'xs', pos 'i']`")); } self.advance(); }
            else if self.is_word("pos") { return self.err(format!("`pos` is gone (2026-09-27): `simd.{op}[lend 'xs', 'i']` (`pos` belongs to `#strict` files)")); }
            values.push(self.value()?);
            while *self.peek() == Tok::Comma { self.advance(); values.push(self.value()?); }
            self.expect(Tok::RBracket, "`]`")?;
            return Ok((head, op, Bracket { values, line, one_list: false }));
        }
        let args = self.bracket()?;
        // `simd.shuffle['v', [4, 3, 2, 1]]` — the new order is fixed numbers, known when compiling
        // (owner, 2026-09-27: a), so it is one instruction and Precog sees through it
        if head == "simd" && op == "shuffle" {
            let fixed = match args.values.get(1) {
                Some(Value::Expr(Expr::Nested(b))) => b.values.iter().all(|v| matches!(v, Value::Expr(Expr::Int(..) | Expr::RawInt(..)))),
                _ => false,
            };
            if args.values.len() != 2 || !fixed { return self.err("`simd.shuffle` takes a vector and the new order as fixed lane numbers: `simd.shuffle['v', [4, 3, 2, 1]]`"); }
        }
        Ok((head, op, args))
    }

    // bracket := '[' (value (',' value)*)? ']'
    // A `[ ]` builds a value: one value is that value, several are a list,
    // whitespace-separated pieces join into text.
    fn bracket(&mut self) -> PResult<Bracket> {
        let line = self.line();
        self.expect(Tok::LBracket, "`[`")?;
        let mut values = Vec::new();
        let mut trailing_comma = false;
        if *self.peek() != Tok::RBracket {
            loop {
                values.push(self.value()?);
                if *self.peek() == Tok::Comma {
                    self.advance();
                    // `[1,]` — a trailing comma makes a one-element list (decided 2026-09-14)
                    if *self.peek() == Tok::RBracket { trailing_comma = true; break; }
                    continue;
                }
                break;
            }
        }
        self.expect(Tok::RBracket, "`]`")?;
        let one_list = trailing_comma && values.len() == 1;
        Ok(Bracket { values, line, one_list })
    }

    // value := expr   — one expression per comma-separated slot.
    // Decided 2026-09-19: joining is explicit (`++`); two values side by side are an error.
    fn value(&mut self) -> PResult<Value> {
        let e = self.expr()?;
        if self.starts_primary() {
            return self.err(format!("two values next to each other (found {} after the value): join text with `++`, separate values with `,`", self.peek()));
        }
        Ok(Value::Expr(e))
    }

    fn starts_primary(&self) -> bool {
        match self.peek() {
            Tok::Int(_) | Tok::Bin(..) | Tok::Text(..) | Tok::Char(_) | Tok::Newline | Tok::Var(_) | Tok::LParen => true,
            Tok::Word(w) => !OP_WORDS.contains(&w.as_str()) && w != "not" && w != "otherwise",
            _ => false,
        }
    }

    /// The operator at the cursor, spelled as the oracle works with it inside (`plus` is `+`,
    /// `times.wrap` is `x~`); `op_len` says how many tokens it takes, `op_old` whether it is a
    /// retired symbol (owner, 2026-10-03: the operators are words).
    fn binop(&self) -> Option<String> {
        // `'a' std::plus 'b'`: std's word operators with their path (owner, 2026-10-06: std is
        // `bare.optional` for everything; `std::pow` since 2026-10-02)
        if matches!(self.peek(), Tok::Word(w) if w == "std") && *self.peek_at(1) == Tok::ColonColon
            && matches!(self.peek_at(2), Tok::Word(x) if ["plus", "minus", "times", "div", "mod", "and", "or", "xor", "shl", "shr"].contains(&x.as_str())) {
            let r = self.binop_from(2);
            if r.is_some() { self.op_len.set(self.op_len.get() + 2); }
            return r;
        }
        self.binop_from(0)
    }

    fn binop_from(&self, k: usize) -> Option<String> {
        self.op_len.set(1);
        self.op_old.set(false);
        let dot_word = |b: &str| *self.peek_at(k + 1) == Tok::Dot && matches!(self.peek_at(k + 2), Tok::Word(x) if x == b);
        match self.peek_at(k) {
            Tok::Op(o) => { if matches!(o.as_str(), "+" | "-" | "+~" | "-~" | "x~" | "xx~") { self.op_old.set(true); } Some(o.clone()) }
            // `times` / `plus` / `minus`, and `.wrap` for the wrapping ones (owner, 2026-10-03)
            Tok::Word(w) if w == "times" || w == "plus" || w == "minus" => {
                let base = match w.as_str() { "times" => "x", "plus" => "+", _ => "-" };
                if dot_word("wrap") { self.op_len.set(3); Some(format!("{base}~")) }
                // `plus.ieee`: IEEE's own answer, a -0 kept (owner, 2026-10-07: "yea")
                else if dot_word("ieee") { self.op_len.set(3); Some(format!("{base}.ieee")) } else { Some(base.into()) }
            }
            // std's `pow`, bare or with its path, and `pow.wrap` (owner, 2026-10-03)
            Tok::Word(w) if w == "pow" => { if dot_word("wrap") || dot_word("ieee") { self.op_len.set(3); Some(format!("pow.{}", if dot_word("wrap") { "wrap" } else { "ieee" })) } else { Some("pow".into()) } }
            Tok::Word(w) if w == "std" && *self.peek_at(k + 1) == Tok::ColonColon && matches!(self.peek_at(k + 2), Tok::Word(x) if x == "pow") => {
                if *self.peek_at(k + 3) == Tok::Dot && matches!(self.peek_at(k + 4), Tok::Word(x) if x == "wrap" || x == "ieee") { let Tok::Word(x) = self.peek_at(k + 4).clone() else { unreachable!() }; self.op_len.set(5); Some(format!("pow.{x}")) } else { self.op_len.set(3); Some("pow".into()) }
            }
            // `and.bit` / `or.bit` — one operator spelled as a chain
            // `and.bit.wrap`, `or.bit.wrap`, `xor.wrap`: operands taken as their bits (owner, 2026-10-07)
            Tok::Word(w) if (w == "and" || w == "or") && dot_word("bit") && *self.peek_at(k + 3) == Tok::Dot && matches!(self.peek_at(k + 4), Tok::Word(x) if x == "wrap") => { self.op_len.set(5); Some(format!("{w}.bit.wrap")) }
            Tok::Word(w) if w == "xor" && dot_word("wrap") => { self.op_len.set(3); Some("xor.wrap".into()) }
            // `shl.masked` / `shr.masked`: the count masked to the width and the answer cut to it, as the
            // chip does (owner, 2026-10-07: "both", then "Whatever's accurate")
            Tok::Word(w) if (w == "shl" || w == "shr") && dot_word("masked") => { self.op_len.set(3); Some(format!("{w}.masked")) }
            Tok::Word(w) if (w == "and" || w == "or") && dot_word("bit") => { self.op_len.set(3); Some(format!("{w}.bit")) }
            // `and.then` / `or.else` — the right side only when the left has not decided (owner, 2026-09-25)
            Tok::Word(w) if w == "and" && dot_word("then") => { self.op_len.set(3); Some("and.then".into()) }
            Tok::Word(w) if w == "or" && dot_word("else") => { self.op_len.set(3); Some("or.else".into()) }
            // `div.floor` / `div.ceil` / `div.trunc` / `div.near`, and `mod.` the same: every division
            // says how it rounds (owner, 2026-10-02); a bare `div` or `mod` is refused in `climb`
            Tok::Word(w) if (w == "div" || w == "mod") && *self.peek_at(k + 1) == Tok::Dot => match self.peek_at(k + 2) {
                // `div.trunc.wrap`: wraps at the type's width instead of overflowing (owner, 2026-10-03)
                Tok::Word(b) if *self.peek_at(k + 3) == Tok::Dot && matches!(self.peek_at(k + 4), Tok::Word(x) if x == "wrap") => { self.op_len.set(5); Some(format!("{w}.{b}.wrap")) }
                // `div.ceil.ieee`: IEEE's own rounding, a zero from below kept as -0 (owner, 2026-10-07: "`a`")
                Tok::Word(b) if *self.peek_at(k + 3) == Tok::Dot && matches!(self.peek_at(k + 4), Tok::Word(x) if x == "ieee") => { self.op_len.set(5); Some(format!("{w}.{b}.ieee")) }
                Tok::Word(b) => { self.op_len.set(3); Some(format!("{w}.{b}")) }
                _ => Some(w.clone())
            },
            Tok::Word(w) if w == "x" || w == "xx" => { self.op_old.set(true); Some(w.clone()) }
            Tok::Word(w) if OP_WORDS.contains(&w.as_str()) => Some(w.clone()),
            Tok::Word(w) if w == "xor" || w == "shl" || w == "shr" => Some(w.clone()),
            // a Limb's word operator with its path, `vecmath::dot`, `vecmath::dot.fast` (owner, 2026-10-02/03)
            Tok::Word(p) if *self.peek_at(k + 1) == Tok::ColonColon && matches!(self.peek_at(k + 2), Tok::Word(n) if !KEYWORDS.contains(&n.as_str())) && !matches!(self.peek_at(k + 3), Tok::LBracket | Tok::ColonColon) => {
                let Tok::Word(n) = self.peek_at(k + 2).clone() else { unreachable!() };
                if *self.peek_at(k + 3) == Tok::Dot && matches!(self.peek_at(k + 4), Tok::Word(_)) && *self.peek_at(k + 5) != Tok::LBracket {
                    let Tok::Word(b) = self.peek_at(k + 4).clone() else { unreachable!() };
                    self.op_len.set(5); Some(format!("{p}::{n}.{b}"))
                } else { self.op_len.set(3); Some(format!("{p}::{n}")) }
            }
            // a narrowed custom word operator, `dot.fast` (owner, 2026-10-02/03)
            Tok::Word(w) if !KEYWORDS.contains(&w.as_str()) && w != "not" && *self.peek_at(k + 1) == Tok::Dot && matches!(self.peek_at(k + 2), Tok::Word(_)) && !matches!(self.peek_at(k + 3), Tok::LBracket | Tok::Dot) => {
                let Tok::Word(b) = self.peek_at(k + 2).clone() else { unreachable!() };
                self.op_len.set(3); Some(format!("{w}.{b}"))
            }
            // a custom word operator (decided 2026-09-20): a bare non-keyword word between operands
            Tok::Word(w) if !KEYWORDS.contains(&w.as_str()) && w != "not" && *self.peek_at(k + 1) != Tok::LBracket && *self.peek_at(k + 1) != Tok::ColonColon && *self.peek_at(k + 1) != Tok::Dot => Some(w.clone()),
            Tok::Eq => Some("=".into()),
            _ => None,
        }
    }

    /// The word a retired operator symbol became (owner, 2026-10-03).
    fn old_op_error(&self, op: &str) -> PResult<Expr> {
        let now = match op { "+" => "plus", "-" => "minus", "x" => "times", "xx" => "pow", "+~" => "plus.wrap", "-~" => "minus.wrap", "x~" => "times.wrap", _ => "pow.wrap" };
        let note = if op.starts_with("xx") { ", std's operator" } else { "" };
        self.err(format!("`{op}` is `{now}` now{note} (owner, 2026-10-03)"))
    }

    fn is_custom_op(&self, op: &str) -> bool { self.lvl(op).0 == 0 && op != "=" }

    // expr — precedence follows mathematics where mathematics has a settled
    // answer (decided 2026-09-14); where it does not, parentheses are required;
    // parentheses always override.
    //
    //   tightest   xx                     right-associative (a power tower)
    //              unary -                so -2 xx 2 is -(2 xx 2)
    //              x  div  mod            x chains; div and mod must be parenthesised
    //                                     when mixed with x, div, or mod (6 div 2 x 3 has
    //                                     no settled reading; mod has no notation)
    //              +  -                   left-associative
    //              ++                     text join, left-associative; below arithmetic,
    //                                     above comparisons (decided 2026-09-19)
    //              == !== < > </== >/==   never chained; never mixed with `not`
    //              not                    applies to one operand or a parenthesis
    //              and
    //   loosest    or                     ∧ before ∨, as in logic
    fn expr(&mut self) -> PResult<Expr> {
        let e = self.climb(0)?;
        self.validate(&e)?;
        Ok(e)
    }

    /// The language's own operator levels, in hundreds so a Limb's levels fit between them
    /// (owner, 2026-10-03); 0 = no level (brackets beside any other operator).
    fn base_level(op: &str) -> u16 {
        match op {
            "or" | "or.else" => 100,
            "and" | "and.then" => 200,
            "==" | "!==" | "<" | ">" | "</==" | ">/==" => 300,
            "++" => 400,
            "+" | "-" | "+~" | "-~" => 500,
            "x" | "x~" => 600,
            o if o.starts_with("div.") || o.starts_with("mod.") => 600,
            "neg" => 700,
            // std's `pow` binds above a negative, right pair first, as `xx` did (owner, 2026-10-03)
            "xx" | "xx~" | "pow" | "pow.wrap" | "pow.ieee" => 800,
            _ => 0,
        }
    }
    /// How a chain of the language's own operator reads: 0 left first, 1 right first, 2 refused.
    fn base_chain(op: &str) -> u8 {
        match op { "==" | "!==" | "<" | ">" | "</==" | ">/==" => 2, "xx" | "xx~" | "pow" | "pow.wrap" | "pow.ieee" => 1, _ => 0 }
    }
    fn lvl(&self, op: &str) -> (u16, u8) {
        // a path names whose operator it is; its level is the operator's own
        let op = op.rsplit("::").next().unwrap_or(op);
        let op = op.strip_suffix(".ieee").unwrap_or(op);
        self.user_levels.get(op).copied().unwrap_or((Parser::base_level(op), Parser::base_chain(op)))
    }

    /// Read only the file's `precedence` blocks (the first pass over a program's files, and the
    /// start of every parse).
    pub fn scan_levels(&mut self) -> PResult<()> {
        let starts: Vec<usize> = self.toks.iter().enumerate().filter(|(_, t)| matches!(&t.tok, Tok::Word(w) if w == "precedence")).map(|(i, _)| i).collect();
        for at in starts { self.pos = at; let levels = self.precedence_block()?; self.place_levels(levels)?; }
        self.pos = 0;
        Ok(())
    }

    /// `precedence.low_first { level.when_chained.left_first 1 { … } … }` (owner, 2026-10-03).
    /// Returns the levels loosest first: (number, chain, operator names, line).
    fn precedence_block(&mut self) -> PResult<Vec<(i128, u8, Vec<String>, usize)>> {
        self.expect_word("precedence")?;
        if *self.peek() != Tok::Dot { return self.err("say which way the levels count: `precedence.low_first { … }` (level 1 binds loosest, good practice) or `precedence.high_first { … }` (owner, 2026-10-03)"); }
        self.advance();
        let high_first = match self.advance().tok {
            Tok::Word(w) if w == "low_first" => false,
            Tok::Word(w) if w == "high_first" => true,
            other => return self.err(format!("`precedence.low_first` or `precedence.high_first`, found `precedence.{other}`")),
        };
        self.expect(Tok::LBrace, "`{` to open the levels")?;
        let mut levels = Vec::new();
        while *self.peek() != Tok::RBrace {
            let line = self.line();
            if !self.is_word("level") { return self.err(format!("a `precedence` block holds `level.when_chained.… N {{ … }}` lines, found {}", self.peek())); }
            self.advance();
            let chain_err = "say how a chain of this level reads: `level.when_chained.left_first`, `level.when_chained.right_first` or `level.when_chained.refused` (owner, 2026-10-03)";
            if *self.peek() != Tok::Dot { return self.err(chain_err); }
            self.advance();
            if !self.is_word("when_chained") { return self.err(chain_err); }
            self.advance();
            self.expect(Tok::Dot, chain_err)?;
            let chain = match self.advance().tok {
                Tok::Word(w) if w == "left_first" => 0, Tok::Word(w) if w == "right_first" => 1, Tok::Word(w) if w == "refused" => 2,
                _ => return self.err(chain_err),
            };
            let n = match self.advance().tok { Tok::Int(i) => i, other => return self.err(format!("a level's number, `level.when_chained.left_first 1 {{ … }}`, found {other}")) };
            self.expect(Tok::LBrace, "`{` to open the level's operators")?;
            let mut names = Vec::new();
            while *self.peek() != Tok::RBrace {
                let mut name = match self.advance().tok {
                    Tok::Word(w) => w, Tok::Op(o) => o,
                    other => return self.err(format!("a level lists operators, found {other}")),
                };
                if *self.peek() == Tok::Dot { if let Tok::Word(b) = self.peek_at(1).clone() { self.advance(); self.advance(); name = format!("{name}.{b}"); } }
                let name = match name.as_str() {
                    "plus" => "+".to_string(), "minus" => "-".into(), "times" => "x".into(),
                    "plus.wrap" => "+~".into(), "minus.wrap" => "-~".into(), "times.wrap" => "x~".into(),
                    "+" | "-" | "x" | "xx" | "+~" | "-~" | "x~" | "xx~" => { self.op_len.set(1); return self.old_op_error(&name).map(|_| Vec::new()); }
                    _ => name,
                };
                names.push(name);
            }
            self.advance();
            levels.push((n, chain, names, line));
        }
        self.advance();
        levels.sort_by_key(|l| if high_first { -l.0 } else { l.0 });
        Ok(levels)
    }

    /// Put a `precedence` block's levels on the language's scale: a level holding a language
    /// operator sits where it does; the others sit between by order (owner, 2026-10-03).
    fn place_levels(&mut self, levels: Vec<(i128, u8, Vec<String>, usize)>) -> PResult<()> {
        let bad = |line: usize, msg: String| Err(ParseError { line, col: 0, msg });
        let mut pinned: Vec<Option<u16>> = Vec::new();
        for (i, (n, chain, names, line)) in levels.iter().enumerate() {
            if levels[..i].iter().any(|l| l.0 == *n) { return bad(*line, format!("two levels numbered {n}")); }
            let mut pin: Option<u16> = None;
            for name in names {
                if BIT_OPS.contains(&name.as_str()) || name == "not.bit" || name == "not" {
                    return bad(*line, format!("`{name}` has no precedence in the language, and a Limb cannot give it one"));
                }
                let base = Parser::base_level(name);
                if base == 0 { continue; }
                if name != "neg" && Parser::base_chain(name) != *chain {
                    let says = ["left_first", "right_first", "refused"][Parser::base_chain(name) as usize];
                    let shown = match name.as_str() { "+" => "plus", "-" => "minus", "x" => "times", "+~" => "plus.wrap", "-~" => "minus.wrap", "x~" => "times.wrap", o => o };
                    return bad(*line, format!("`{shown}`'s chains read `{says}`: its level must say `level.when_chained.{says}`"));
                }
                if let Some(p) = pin { if p != base { return bad(*line, format!("`{name}` and another language operator in one level, but the language has them at different levels")); } }
                pin = Some(base);
            }
            pinned.push(pin);
        }
        let mut values = vec![0u16; levels.len()];
        for i in 0..levels.len() {
            if let Some(p) = pinned[i] { values[i] = p; continue; }
            let prev = (0..i).rev().find_map(|k| pinned[k].map(|v| (k, v)));
            let next = (i + 1..levels.len()).find_map(|k| pinned[k].map(|v| (k, v)));
            let (pk, pv) = prev.map(|(k, v)| (k as i64, v as i64)).unwrap_or((-1, 0));
            let mut v = match next {
                Some((nk, nv)) => pv + (nv as i64 - pv) * (i as i64 - pk) / (nk as i64 - pk),
                None => pv + 10 * (i as i64 - pk),
            };
            if (1..=8).any(|h| v == h * 100) { v += 1; }
            if let Some(prev) = prev { if v as u16 <= prev.1 { return bad(levels[i].3, "levels out of order with the language's".into()); } }
            values[i] = v as u16;
        }
        for (i, (_, chain, names, line)) in levels.iter().enumerate() {
            for name in names {
                if Parser::base_level(name) != 0 { continue; }
                // the same block read twice (the first pass and this file's own parse) agrees with itself;
                // two Limbs giving one operator different levels do not
                if let Some(old) = self.user_levels.get(name) { if *old != (values[i], *chain) { return bad(*line, format!("`{name}` is in two levels")); } }
                self.user_levels.insert(name.clone(), (values[i], *chain));
                let clash = SHARED_LEVELS.with(|l| { let mut l = l.borrow_mut(); match l.get(name) { Some(old) if *old != (values[i], *chain) => true, _ => { l.insert(name.clone(), (values[i], *chain)); false } } });
                if clash { return bad(*line, format!("`{name}` has a different precedence in another Limb")); }
            }
        }
        Ok(())
    }

    fn climb(&mut self, min: u16) -> PResult<Expr> {
        let line = self.line();
        let mut lhs = self.unary()?;
        loop {
            let Some(op) = self.binop() else { break };
            if op == "=" { if self.in_map_key { break; } return self.err("`=` inside a value: assignment is a statement; equality is `==`"); }
            if self.op_old.get() && !self.internal_ops { return self.old_op_error(&op); }
            if self.no_std && is_std_op(&op) { return self.std_missing(&format!("`{}`", op_shown(&op))); }
            if op == "div" || op == "mod" {
                return self.err(format!("say how `{op}` rounds: `{op}.floor`, `{op}.ceil`, `{op}.trunc` or `{op}.near` (owner, 2026-10-02)"));
            }
            if let Some(base) = op.strip_suffix(".wrap") {
                if op.starts_with("mod.") { return self.err(format!("`mod` never overflows, so there is nothing to wrap: write `{base}` (owner, 2026-10-03)")); }
                if op.starts_with("div.") && !crate::interp::is_div_mod(base) { return self.err(format!("no `{base}`: a division rounds `.floor`, `.ceil`, `.trunc` or `.near`")); }
            } else if let Some(base) = op.strip_suffix(".ieee") {
                // every operator that can make a -0 has one (owner, 2026-10-07: "yea", `mod` and `pow` too)
                if !matches!(base, "+" | "-" | "x" | "pow" | "div.floor" | "div.ceil" | "div.trunc" | "div.near" | "mod.floor" | "mod.ceil" | "mod.trunc") { return self.err(format!("`.ieee` goes on an operator that can make a -0, `plus`, `minus`, `times`, `pow`, a `div.X`, `mod.floor`, `mod.ceil` or `mod.trunc`; `{}` has no `.ieee` form", op_shown(base))); }
            } else if (op.starts_with("div.") || op.starts_with("mod.")) && !crate::interp::is_div_mod(&op) {
                return self.err(format!("no `{op}`: a division rounds `.floor`, `.ceil`, `.trunc` or `.near`"));
            }
            if self.is_custom_op(&op) {
                // A custom operator has no precedence (decided 2026-09-20): it may not
                // sit next to another operator without parentheses. Bitwise operators
                // follow the same rule (2026-09-21): mathematics has no reading for them.
                if min > 0 { break; }
                let what = if BIT_OPS.contains(&op.as_str()) { "a bitwise operator" } else { "a custom operator" };
                if matches!(lhs, Expr::Binary(..)) || matches!(&lhs, Expr::Unary(u, ..) if u == "-" || u == "neg.ieee") { return self.err(format!("`{op}` next to another operator: {what} has no precedence — parenthesise")); }
                for _ in 0..self.op_len.get() { self.advance(); }
                let rhs = self.unary()?;
                if let Some(next) = self.binop() { if next != "=" { return self.err(format!("`{next}` after `{op}`: {what} has no precedence — parenthesise")); } }
                lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs), line);
                continue;
            }
            let (lv, chain) = self.lvl(&op);
            if lv < min || lv == 0 { break; }
            // a level whose chains are refused (owner, 2026-10-03): the language's comparisons are
            // checked in `validate`; a Limb's here
            if chain == 2 && self.user_levels.contains_key(&op) {
                if let Expr::Binary(prev, ..) = &lhs { if self.lvl(prev).0 == lv { return self.err(format!("`{prev}` then `{op}`: this level does not chain (`level.when_chained.refused`) — parenthesise")); } }
            }
            for _ in 0..self.op_len.get() { self.advance(); }
            // right pair first where the level says so (std's `pow`), left pair first otherwise
            let rhs = if chain == 1 { self.climb(lv)? } else { self.climb(lv + 1)? };
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs), line);
        }
        Ok(lhs)
    }

    fn validate(&self, e: &Expr) -> PResult<()> {
        let bad = |line: usize, msg: String| Err(ParseError { line, col: 0, msg });
        match e {
            Expr::Binary(op, l, r, line) => {
                let is_cmp = |o: &str| matches!(o, "==" | "!==" | "<" | ">" | "</==" | ">/==");
                for child in [l.as_ref(), r.as_ref()] {
                    match child {
                        Expr::Binary(cop, ..) => {
                            if cop.starts_with("mod") || op.starts_with("mod") {
                                return bad(*line, format!("`{cop}` next to `{op}`: mathematics has no settled reading for mod beside another operator — parenthesise"));
                            }
                            let muly = |o: &str| o == "x" || o.starts_with("div.") || o.starts_with("mod.");
                            let both_mul = muly(op) && muly(cop);
                            if both_mul && !(op == "x" && cop == "x") {
                                return bad(*line, format!("`{cop}` next to `{op}`: mathematics has no settled reading here (6 div 2 x 3, or anything with mod) — parenthesise one side"));
                            }
                            if is_cmp(op) && is_cmp(cop) {
                                return bad(*line, format!("`{cop}` next to `{op}`: comparisons do not chain — write `[(a {cop} b) and (b {op} c)]`, or parenthesise"));
                            }
                        }
                        Expr::Unary(u, ..) if u == "not" && is_cmp(op) => {
                            return bad(*line, "`not` next to a comparison: no settled reading — write `[not (a == b)]` or `[(not a) == b]`".into());
                        }
                        _ => {}
                    }
                }
                self.validate(l)?;
                self.validate(r)
            }
            Expr::Unary(_, inner, _) => self.validate(inner),
            Expr::Paren(inner, _) => self.validate(inner),
            _ => Ok(()),
        }
    }

    // unary := ('not' | '-') unary | primary
    fn unary(&mut self) -> PResult<Expr> {
        let line = self.line();
        if self.no_std && self.is_word("not") {
            let bit = *self.peek_at(1) == Tok::Dot && matches!(self.peek_at(2), Tok::Word(b) if b == "bit");
            return self.std_missing(if bit { "`not.bit`" } else { "`!`" });
        }
        if self.no_std && *self.peek() == Tok::Op("-".into()) { return self.std_missing("`-`"); }
        // `neg.ieee 'x'`: IEEE's negation, 0 becoming -0 (owner, 2026-10-07: "yea"); it binds as `-` does
        if self.is_word("neg") && *self.peek_at(1) == Tok::Dot && matches!(self.peek_at(2), Tok::Word(b) if b == "ieee") {
            if self.no_std { return self.std_missing("`neg.ieee`"); }
            self.advance(); self.advance(); self.advance();
            return Ok(Expr::Unary("neg.ieee".into(), Box::new(self.climb(701)?), line));
        }
        if self.is_word("not") {
            self.advance();
            if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(b) if b == "bit") {
                self.advance(); self.advance();
                let inner = self.primary()?;
                return Ok(Expr::Unary("not.bit".into(), Box::new(inner), line));
            }
            return Ok(Expr::Unary("not".into(), Box::new(self.unary()?), line));
        }
        // unary minus sits below xx: -2 xx 2 is -(2 xx 2), as in mathematics
        if *self.peek() == Tok::Op("-".into()) { self.advance(); return Ok(Expr::Unary("-".into(), Box::new(self.climb(701)?), line)); }
        self.primary()
    }

    fn primary(&mut self) -> PResult<Expr> {
        let line = self.line();
        match self.peek().clone() {
            Tok::Int(i) => {
                self.advance();
                // `5 seconds` — a duration literal (design/concurrency.md): the unit sits on the number
                if let Tok::Word(w) = self.peek().clone() {
                    if let Some((_, per)) = DURATION_UNITS.iter().find(|(u, _)| *u == w) {
                        self.advance();
                        if i < 0 { return self.err("a duration is never negative"); }
                        return Ok(Expr::Duration(i * per, line));
                    }
                }
                // without ASP a literal is a bit pattern (owner, 2026-10-06); ASP makes it a value again
                if self.lacks.iter().any(|c| c == "ASP") { return Ok(Expr::RawInt(i, line)); }
                Ok(Expr::Int(i, line))
            }
            Tok::BigInt(s) => { self.advance(); Ok(Expr::BigInt(s, line)) }
            Tok::Bin(b, t) => { self.advance(); Ok(Expr::Bin(b, t, line)) }
            // `*0.1` — "I want exactly this" (owner, 2026-10-02): a decimal literal that is a deci
            Tok::Op(o) if o == "*" => {
                self.advance();
                match self.advance().tok {
                    Tok::Bin(_, t) => Ok(Expr::Exact(t, line)),
                    Tok::Int(i) => self.err(format!("`*` marks a decimal literal as exact; `{i}` is a whole number and exact already")),
                    other => self.err(format!("`*` before a literal means \"exactly this\": `*0.1`; found {other} (and `*` is not multiplication: that is `x`)")),
                }
            }
            Tok::Text(t, _) => { self.advance(); Ok(Expr::Text(t, line)) }
            Tok::Char(c) => { self.advance(); Ok(Expr::Char(c, line)) }
            Tok::Newline => { self.advance(); Ok(Expr::Newline(line)) }
            Tok::Var(v) => {
                self.advance();
                if *self.peek() == Tok::LBracket {
                    // 'f'[args] calls a function value, 'xs'[2] reads a list: which one is decided by
                    // what the variable holds (no `pos` since 2026-09-27, owner)
                    let says_pos = matches!(self.peek_at(1), Tok::Word(w) if w == "pos") && !matches!(self.peek_at(2), Tok::LBracket | Tok::Dot | Tok::RBracket | Tok::Comma);
                    if self.strict && says_pos {
                        let idx = self.index_bracket(&v)?;
                        return Ok(Expr::Index(v, Box::new(idx), line));
                    }
                    if says_pos { return self.err(format!("`pos` is gone (2026-09-27): a position is written bare, `'{v}'[2]` (`pos` belongs to `#strict` files)")); }
                    let args = self.bracket()?;
                    return Ok(Expr::CallValue(v, args, None, line));
                }
                if *self.peek() == Tok::Dot {
                    self.advance();
                    let field = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a field name after `'{v}'.`, found {other}")) };
                    if *self.peek() == Tok::LBracket {
                        let idx = self.index_bracket(&format!("{v}'.{field}"))?;
                        return Ok(Expr::FieldIndex(v, field, Box::new(idx), line));
                    }
                    return Ok(Expr::Field(v, field, line));
                }
                Ok(Expr::Var(v, line))
            }
            Tok::LParen => {
                self.advance();
                let e = self.expr()?;
                self.expect(Tok::RParen, "`)`")?;
                Ok(Expr::Paren(Box::new(e), line))
            }
            Tok::LBracket => Ok(Expr::Nested(self.bracket()?)),
            Tok::Ellipsis => {
                // `...'xs'` — unpack a list into a call's variadic part (decided 2026-09-21)
                self.advance();
                // `...copy 'xs'` / `...give 'xs'`: how owned elements are handed over (decided 2026-09-21)
                let verb = match self.peek() {
                    Tok::Word(w) if w == "copy" => { self.advance(); Some(Verb::Copy) }
                    Tok::Word(w) if w == "give" => { self.advance(); Some(Verb::Give) }
                    Tok::Word(w) if w == "lend" => return self.err("`...'xs'` with no verb already lends its elements to a `lent ...` parameter; `copy` or `give` are the verbs that go after `...`"),
                    _ => None,
                };
                match self.advance().tok {
                    Tok::Var(v) => Ok(Expr::Spread(v, verb, line)),
                    other => self.err(format!("`...` unpacks a list variable into a call: `...'xs'`, found {other}")),
                }
            }
            Tok::Op(o) if o == "&" => {
                // `&'s'` / `&mut 's'` — Rust's spelling, an alternative to `lend` / `lend.mut` (owner, 2026-09-21)
                self.advance();
                let verb = if self.is_word("mut") { self.advance(); Verb::LendMut } else { Verb::Lend };
                match self.advance().tok {
                    Tok::Var(v) => {
                        if *self.peek() == Tok::Dot {
                            self.advance();
                            let field = match self.advance().tok { Tok::Word(f) => f, other => return self.err(format!("expected a field name after `'{v}'.`, found {other}")) };
                            return Ok(Expr::OwnedField(verb, v, field, line));
                        }
                        if *self.peek() == Tok::LBracket { let idx = self.index_bracket(&v)?; return Ok(Expr::OwnedElem(verb, v, Box::new(idx), line)); }
                        Ok(Expr::Owned(verb, v, line))
                    }
                    other => self.err(format!("`&` lends a variable: `&'name'` or `&mut 'name'`, found {other}")),
                }
            }
            Tok::Word(w) => match w.as_str() {
                "give" | "copy" | "lend" => {
                    self.advance();
                    let verb = match w.as_str() {
                        "give" => Verb::Give,
                        "copy" => Verb::Copy,
                        _ => {
                            if *self.peek() == Tok::Dot {
                                self.advance();
                                match self.advance().tok { Tok::Word(m) if m == "mut" => Verb::LendMut, other => return self.err(format!("the only narrowing of `lend` is `.mut`, found {other}")) }
                            } else { Verb::Lend }
                        }
                    };
                    match self.advance().tok {
                        Tok::Var(v) => {
                            if *self.peek() == Tok::Dot {
                                self.advance();
                                let field = match self.advance().tok { Tok::Word(f) => f, other => return self.err(format!("expected a field name after `'{v}'.`, found {other}")) };
                                return Ok(Expr::OwnedField(verb, v, field, line));
                            }
                            if *self.peek() == Tok::LBracket {
                                let idx = self.index_bracket(&v)?;
                                return Ok(Expr::OwnedElem(verb, v, Box::new(idx), line));
                            }
                            Ok(Expr::Owned(verb, v, line))
                        }
                        other => self.err(format!("`{w}` takes a variable: `{w} 'name'`, found {other}")),
                    }
                }
                "lent" => self.err("`lent` describes a parameter; a caller writes `lend`"),
                "range" => self.err("`range[…]` is only decided as what a `for` runs over: `for temp 'i' in range[1, 10]`"),
                "check" | "nocheck" | "NO_FUCKING_CHECKS" => self.err(format!("`{w}` goes in front of a statement, not inside a value; if half a statement needs it, split the statement")),
                "list" | "array" => { let (h, o, a) = self.builtin()?; Ok(Expr::Builtin(h, o, a, line)) }
                "channel" if *self.peek_at(1) == Tok::LBracket => {
                    // channel[T] / channel[T][0, N] as a value: a new channel
                    self.advance(); self.advance();
                    let ty = self.ty()?;
                    self.expect(Tok::RBracket, "`]` after the channel's type")?;
                    let mut cap = None;
                    if *self.peek() == Tok::LBracket && matches!(self.peek_at(1), Tok::Int(_)) {
                        self.advance();
                        let lo = self.bound()?; self.expect(Tok::Comma, "`,` between the bounds")?; let hi = self.bound()?; self.expect(Tok::RBracket, "`]` after the bounds")?;
                        match (lo, hi) { (Bound::Int(0), Bound::Int(b)) if b >= 1 => cap = Some(b as usize), _ => return self.err("a channel's bound is `[0, N]` with N at least 1; no bound is a hand-off") }
                    }
                    Ok(Expr::ChannelNew(ty, cap, line))
                }
                "channel" => {
                    let (h, o, a) = self.builtin()?;
                    if o == "take" { return Ok(Expr::MapFail("take".into(), a, None, line)); }   // can fail `closed`
                    Ok(Expr::Builtin(h, o, a, line))
                }
                "simd" => { let (h, o, a) = self.builtin()?; Ok(Expr::Builtin(h, o, a, line)) }
                "map" if *self.peek_at(1) == Tok::LBracket => {
                    // map["ann" = 5551, "bob" = 5552]  (decided 2026-09-22): `key = value`, the struct's spelling
                    self.advance(); self.advance();
                    if *self.peek() == Tok::RBracket { return self.err("an empty map is `map.empty`, not `map[]`"); }
                    let mut pairs = Vec::new();
                    loop {
                        self.in_map_key = true;
                        let k = self.expr();
                        self.in_map_key = false;
                        let k = k?;
                        if *self.peek() != Tok::Eq { return self.err("a map literal pairs each key with a value: `map[\"ann\" = 5551, …]`"); }
                        self.advance();
                        let v = self.value()?;
                        pairs.push((k, Bracket { values: vec![v], line, one_list: false }));
                        if *self.peek() == Tok::Comma { self.advance(); if *self.peek() == Tok::RBracket { break; } continue; }
                        break;
                    }
                    self.expect(Tok::RBracket, "`]` to close the map")?;
                    Ok(Expr::MapLit(pairs, line))
                }
                "map" => {
                    let (h, o, a) = self.builtin()?;
                    if o == "get" || o == "remove" { return Ok(Expr::MapFail(o, a, None, line)); }   // can fail `missing`
                    Ok(Expr::Builtin(h, o, a, line))
                }
                // `ptr.cast[T]['p']` (owner, 2026-09-30: a): the type in its own brackets, then the pointer
                "ptr" if matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Word(c) if c == "cast") => {
                    self.advance(); self.advance(); self.advance();
                    self.expect(Tok::LBracket, "`[` and the type a pointer is cast to: `ptr.cast[Point]['p']`")?;
                    let ty = self.ty()?;
                    self.expect(Tok::RBracket, "`]` after the type: `ptr.cast[Point]['p']`")?;
                    if *self.peek() != Tok::LBracket { return self.err("`ptr.cast[T]` then the pointer in its own brackets: `ptr.cast[Point]['p']`"); }
                    let args = self.bracket()?;
                    if args.values.len() != 1 { return self.err(format!("`ptr.cast[T]` takes one pointer, given {}", args.values.len())); }
                    self.use_power("raw_memory.cast", Some("ptr.cast".into()), line);
                    Ok(Expr::PtrCast(ty, args, line))
                }
                "ptr" => { let (h, o, a) = self.ptr_op()?; Ok(Expr::Builtin(h, o, a, line)) }
                "str" if matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Word(o) if STD_STR_OPS.contains(&o.as_str())) => Ok(Expr::Call(self.call()?)),
                "str" if matches!(self.peek_at(1), Tok::Dot) && !matches!(self.peek_at(2), Tok::Word(u) if u == "utf8" || u == "utf16" || u == "utf32" || u == "ascii") => {
                    // str.bytes[…] / str.points[…] / str.graphemes[…] / str.each.<unit>[…]  (decided 2026-09-20)
                    self.advance(); self.advance();
                    let op = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected an operation after `str.`, found {other}")) };
                    let op = match op.as_str() {
                        "bytes" | "points" | "graphemes" => op,
                        "each" => {
                            self.expect(Tok::Dot, "`.byte`, `.point`, or `.grapheme` after `str.each`")?;
                            match self.advance().tok {
                                Tok::Word(u) if u == "byte" || u == "point" || u == "grapheme" => format!("each.{u}"),
                                other => return self.err(format!("`str.each` walks by a unit: `.byte`, `.point`, or `.grapheme`; found {other}")),
                            }
                        }
                        "length" | "len" | "count" => return self.err("a text has no single length: `str.bytes[…]`, `str.points[…]`, or `str.graphemes[…]` — say which (decided 2026-09-20)"),
                        "split" | "find" | "contains" | "trim" | "upper" | "lower" | "replace" | "starts_with" | "ends_with" | "at" | "slice" =>
                            return self.err(format!("`str.{op}` is std's: `std::str.{op}[…]` — the language owns only the units (bytes/points/graphemes) and `each`")),
                        _ => return self.err(format!("`str.{op}` does not exist; the language has str.bytes, str.points, str.graphemes, str.each.<unit>")),
                    };
                    if *self.peek() != Tok::LBracket { return self.err(format!("`str.{op}` is a call and needs `[…]`")); }
                    let args = self.bracket()?;
                    if args.values.len() != 1 { return self.err(format!("`str.{op}` takes one text: `str.{op}[lend 's']`")); }
                    Ok(Expr::Builtin("str".into(), op, args, line))
                }
                "char" if matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Word(u) if u == "code" || ((u == "point" || u == "byte" || u == "grapheme") && *self.peek_at(3) == Tok::Dot)) => {
                    // char.code[c] — the number under a char.point / char.byte;
                    // char.point.from_code[n] / char.byte.from_code[n] — back, failing not_a_char (decided 2026-09-21)
                    self.advance(); self.advance();
                    let w = match self.advance().tok { Tok::Word(w) => w, _ => unreachable!() };
                    if w == "code" {
                        if *self.peek() != Tok::LBracket { return self.err("`char.code` is a call and needs `[…]`: `char.code[`A`]`"); }
                        let args = self.bracket()?;
                        if args.values.len() != 1 { return self.err("`char.code` takes one character"); }
                        return Ok(Expr::Builtin("char".into(), "code".into(), args, line));
                    }
                    self.advance(); // the dot
                    match self.advance().tok { Tok::Word(f) if f == "from_code" => {}, other => return self.err(format!("`char.{w}.` is followed by `from_code[…]`, found {other}")) }
                    let unit = match w.as_str() { "point" => CharUnit::Point, "byte" => CharUnit::Byte, _ => return self.err("a grapheme has no single code; build it from text: `to.char.grapheme[\"…\"]`") };
                    if *self.peek() != Tok::LBracket { return self.err(format!("`char.{w}.from_code` is a call and needs `[…]`")); }
                    let args = self.bracket()?;
                    if args.values.len() != 1 { return self.err(format!("`char.{w}.from_code` takes one number")); }
                    Ok(Expr::CharFromCode(unit, args, None, line))
                }
                "size" | "align" => {
                    // size.of[T] / align.of[T]   (design/ffi.md)
                    self.advance();
                    self.expect(Tok::Dot, &format!("`.of` after `{w}` (`{w}.of[int32]`)"))?;
                    match self.advance().tok { Tok::Word(o) if o == "of" => {}, other => return self.err(format!("`{w}.of[…]` is the only form, found {other}")) }
                    self.expect(Tok::LBracket, "`[`")?;
                    let saved = self.at_boundary; self.at_boundary = true;
                    let ty = self.ty();
                    self.at_boundary = saved;
                    let ty = ty?;
                    self.expect(Tok::RBracket, "`]`")?;
                    Ok(Expr::SizeOf(ty, w == "size", line))
                }
                "to" => {
                    // to.<type>[value]   (decided 2026-09-19)
                    self.advance();
                    self.expect(Tok::Dot, "`.` after `to` (`to.int32[…]`, `to.str.utf8[…]`)")?;
                    // `to.array[give 'l']` / `to.list[give 'a']` (owner, 2026-09-23): the element type
                    // does not change, so it is not written
                    if let Tok::Word(k) = self.peek().clone() {
                        if (k == "list" || k == "array") && matches!(self.peek_at(1), Tok::LBracket) {
                            self.advance();
                            let b = self.bracket()?;
                            if b.values.len() != 1 { return self.err(format!("`to.{k}[…]` converts exactly one value")); }
                            if *self.peek() == Tok::LBracket { return self.err(format!("`to.{k}[…]` keeps the element type, so it is not written: `to.{k}[give 'x']`")); }
                            let blank = Box::new(Ty::Var(String::new()));
                            return Ok(Expr::Convert(if k == "array" { Ty::Array(blank) } else { Ty::List(blank) }, b, None, 0, line));
                        }
                    }
                    // `.useful` follows the verb (owner, 2026-09-30: b): `to.useful.str.utf8[…]`, `to.useful.deci64[…]`
                    let useful_first = if matches!(self.peek(), Tok::Word(w) if w == "useful") && matches!(self.peek_at(1), Tok::Dot) { self.advance(); self.advance(); true } else { false };
                    // `to.stored.str.utf8[…]`: the stored form, said outright as prints say it (owner, 2026-10-02)
                    let stored_first = !useful_first && matches!(self.peek(), Tok::Word(w) if w == "stored") && matches!(self.peek_at(1), Tok::Dot);
                    if stored_first { self.advance(); self.advance(); }
                    // `to.int8.bits[128]`: the raw form, the value's bits as the type's (owner, 2026-10-06:
                    // ASP keeps a way to write numbers like the hardware, never the default)
                    let bits = !useful_first && !stored_first && matches!(self.peek(), Tok::Word(_)) && *self.peek_at(1) == Tok::Dot && matches!(self.peek_at(2), Tok::Word(b) if b == "bits");
                    if bits { self.toks.drain(self.pos + 1..self.pos + 3); }
                    self.to_target = true;
                    let ty = self.ty();
                    self.to_target = false;
                    let ty = ty?;
                    if bits {
                        if !matches!(ty, Ty::Int(_) | Ty::Uint(_)) { return self.err(format!("`.bits` is for whole-number types (`to.int8.bits[…]`); what `to.{}.bits` means is not decided", ty_name(&ty))); }
                        if *self.peek() != Tok::LBracket { return self.err(format!("`to.{}.bits` converts a value: `to.{}.bits[…]`", ty_name(&ty), ty_name(&ty))); }
                        let b = self.bracket()?;
                        if b.values.len() != 1 { return self.err("`to.…[…]` converts exactly one value"); }
                        return Ok(Expr::Convert(ty, b, None, 3, line));
                    }
                    if matches!(ty, Ty::List(_) | Ty::Array(_)) { return self.err("`to.list[…]` / `to.array[…]` keep the element type, so it is not written: `to.array[give 'l']`"); }
                    // a named type is allowed here only if it is a `layout.C enum` (checked when it runs:
                    // the parser cannot yet tell a struct's name from an enum's)
                    if matches!(ty, Ty::Inst(..) | Ty::Var(_)) { return self.err(format!("no conversion to {}; conversions are between numbers, text, lists and arrays", ty_name(&ty))); }
                    // `to.deci64.useful[…]`: the shortest decimal that reads back as the same bin (owner, 2026-09-29)
                    // the type parser reads `deci64.useful` as one dotted name
                    let (ty, dotted_useful) = match &ty {
                        Ty::Struct(n) if n.ends_with(".useful") && matches!(&n[..n.len() - 7], "deci32" | "deci64" | "deci128") => (Ty::Deci(n[4..n.len() - 7].parse().unwrap()), true),
                        _ => (ty, false),
                    };
                    if dotted_useful || (*self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(w) if w == "useful")) {
                        return self.err(format!("`.useful` follows the verb: `to.useful.{}[…]` (owner, 2026-09-30)", ty_name(&ty)));
                    }
                    if useful_first && !matches!(ty, Ty::Deci(_) | Ty::Str(_) | Ty::TextRanged(..)) && !matches!(&ty, Ty::Ranged(t, ..) if matches!(**t, Ty::Deci(_))) { return self.err(format!("`to.useful` makes text or a deci: `to.useful.str.utf8[…]`, `to.useful.deci64[…]`; `to.{}` has no useful form", ty_name(&ty))); }
                    if stored_first && !matches!(ty, Ty::Str(_) | Ty::TextRanged(..) | Ty::Deci(_)) && !matches!(&ty, Ty::Ranged(t, ..) if matches!(**t, Ty::Deci(_))) { return self.err(format!("`to.stored` makes text or a deci: `to.stored.str.utf8[…]`, `to.stored.deci64[…]`; `to.{}` has one form", ty_name(&ty))); }
                    let useful: u8 = if useful_first { 2 } else if stored_first { 1 } else { 0 };
                    if matches!(ty, Ty::Ranged(..)) && *self.peek() != Tok::LBracket { return self.err(format!("`to.{}` converts a value: `to.{}[…]`", ty_name(&ty), ty_name(&ty))); }
                    if *self.peek() != Tok::LBracket { return self.err(format!("`to.{}` converts a value: `to.{}[…]`", ty_name(&ty), ty_name(&ty))); }
                    let b = self.bracket()?;
                    if b.values.len() != 1 { return self.err("`to.…[…]` converts exactly one value"); }
                    Ok(Expr::Convert(ty, b, None, useful, line))
                }
                // bare `true[…]` is the boolean followed by a bracket; a function named `true` is
                // legal and reached by path (keywords may name functions, 2026-09-19)
                "match" => {
                    // 'label' := match 's' { empty { "free" } … };   (owner, 2026-09-29)
                    self.advance();
                    let value = self.rhs()?;
                    if *self.peek() != Tok::LBrace { return self.err(format!("expected `{{` after the value to match, found {}", self.peek())); }
                    let arms = self.arms("match")?;
                    Ok(Expr::Match(value, arms, line))
                }
                "op" => {
                    // `op +` — an operator as a value (owner, 2026-09-29); negation is `op neg`,
                    // since `op -` is subtraction
                    self.advance();
                    let op = match self.peek().clone() {
                        Tok::Word(w) if w == "neg" => { self.advance(); w }
                        Tok::Word(w) if w == "not" => {
                            self.advance();
                            if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(b) if b == "bit") { self.advance(); self.advance(); "not.bit".to_string() } else { w }
                        }
                        _ => match self.binop() {
                            Some(o) if o == "=" => return self.err("`=` is not an operator; equality is `op ==`"),
                            Some(_) if self.op_old.get() && !self.internal_ops => { let o = self.binop().unwrap(); return self.old_op_error(&o); }
                            Some(o) => { for _ in 0..self.op_len.get() { self.advance(); } o }
                            None => return self.err(format!("`op` is followed by an operator: `op +`, `op mod.floor`, `op neg`; found {}", self.peek())),
                        },
                    };
                    if op == "div" || op == "mod" { return self.err(format!("say how `{op}` rounds: `op {op}.floor`, `op {op}.ceil`, `op {op}.trunc` or `op {op}.near` (owner, 2026-10-02)")); }
                    if op == "and.then" || op == "or.else" {
                        let plain = op.split('.').next().unwrap();
                        return self.err(format!("`op {op}` cannot skip anything: a function's arguments are worked out before it runs, so it would be `op {plain}`. Write `op {plain}` (owner, 2026-09-29)"));
                    }
                    let cmp = matches!(op.as_str(), "==" | "!==" | "<" | ">" | "</==" | ">/==");
                    let src = match op.as_str() {
                        "neg" => "fn ['a': <T>] -> <T> { return -'a'; }".to_string(),
                        "not" => "fn ['a': bool] -> bool { return not 'a'; }".to_string(),
                        "not.bit" => "fn ['a': <T>] -> <T> { return not.bit 'a'; }".to_string(),
                        "and" | "or" => format!("fn ['a': bool, 'b': bool] -> bool {{ return 'a' {op} 'b'; }}"),
                        "++" => "fn ['a': <A>, 'b': <B>] -> str.utf8 { return 'a' ++ 'b'; }".to_string(),
                        "xx" | "xx~" | "shl" | "shr" => format!("fn ['a': <T>, 'b': <U>] -> <T> {{ return 'a' {op} 'b'; }}"),
                        _ if cmp => format!("fn ['a': <T>, 'b': <T>] -> bool {{ return 'a' {op} 'b'; }}"),
                        _ if Parser::base_level(&op) != 0 || OP_WORDS.contains(&op.as_str()) || BIT_OPS.contains(&op.as_str()) || op == "-~" || op == "+~" => format!("fn ['a': <T>, 'b': <T>] -> <T> {{ return 'a' {op} 'b'; }}"),
                        _ => return Ok(Expr::OpValue(op, None, line)),
                    };
                    let toks = crate::lexer::lex(&src).map_err(|e| ParseError { line, col: 0, msg: e.msg })?;
                    let mut p = Parser::new_in(toks, self.module.clone());
                    p.internal_ops = true;
                    // the oracle's own stand-in functions: their blanks take whatever the operator takes
                    p.declared = Some(vec!["T".into(), "U".into(), "A".into(), "B".into()]);
                    let mut f = p.anon_func()?;
                    f.name = format!("op {op}");
                    f.bounds = p.bounds;
                    Ok(Expr::OpValue(op, Some(std::rc::Rc::new(f)), line))
                }
                "true" | "false" if matches!(self.peek_at(1), Tok::LBracket) => self.err(format!("bare `{w}` is the boolean, so `{w}[…]` is no call; a function named `{w}` is reached by path: `my.{w}[…]`")),
                "true" => { self.advance(); Ok(Expr::Bool(true, line)) }
                "false" => { self.advance(); Ok(Expr::Bool(false, line)) }
                "my" if !matches!(self.peek_at(1), Tok::Dot) => self.err("`my` is the root of this Limb and is followed by `.`: `my.helper[]`, `my.'pi'`"),
                "fn" => {
                    // fn [params] -> T with [lend 'v', give 'w'] { … } — an anonymous function (2026-09-21)
                    let saved_bounds = std::mem::take(&mut self.bounds);
                    // reuse func(): give it a name it can never collide with
                    let mut f = self.anon_func()?;
                    self.bounds = saved_bounds;
                    f.name = format!("<anonymous at line {line}>");
                    let caps = std::mem::take(&mut self.pending_captures);
                    Ok(Expr::Lambda(std::rc::Rc::new(f), caps, line))
                }
                _ if KEYWORDS.contains(&w.as_str()) && w != "my" => self.err(format!("`{w}` is a keyword and cannot appear inside a value")),
                _ if self.is_fn_ref() => {
                    // a bare function name with no brackets: the function as a value
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), _ => unreachable!() }
                        if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(_)) { self.advance(); continue; }
                        break;
                    }
                    Ok(Expr::FnRef(self.resolve(&segs), self.module.clone(), line))
                }
                _ if self.is_enum_bare() => {
                    // `Status.empty` / `shapes.Status.empty` — an enum case with no data;
                    // `Status.claimed["x"]` goes through call() and resolves at run time
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), _ => unreachable!() }
                        if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(_)) { self.advance(); continue; }
                        break;
                    }
                    let case = segs.pop().unwrap();
                    Ok(Expr::EnumValue(self.type_name(&segs)?, case, None, line))
                }
                _ if self.is_const_path() => {
                    // `shapes.'pi'`, `my.'pi'`
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), _ => unreachable!() }
                        self.expect(Tok::Dot, "`.`")?;
                        if let Tok::Var(_) = self.peek() { break; }
                    }
                    let name = match self.advance().tok { Tok::Var(v) => v, _ => unreachable!() };
                    Ok(Expr::ConstPath(self.resolve(&segs), name, self.module.clone(), line))
                }
                // another Limb's struct or const: `geometry::Circle[radius = 2.0]`, `geometry::'pi'`
                // (limbs.md: `::` crosses into another Limb)
                _ if self.is_limb_item() => {
                    let limb = match self.advance().tok { Tok::Word(w) => w, _ => unreachable!() };
                    self.advance(); // ::
                    let mut segs = vec![format!("::{limb}")];
                    loop {
                        match self.peek().clone() {
                            Tok::Var(v) => { self.advance(); return Ok(Expr::ConstPath(segs, v, self.module.clone(), line)); }
                            Tok::Word(w) => { self.advance(); segs.push(w); }
                            other => return self.err(format!("expected a name after `{limb}::`, found {other}")),
                        }
                        if *self.peek() == Tok::Dot { self.advance(); continue; }
                        break;
                    }
                    let w = segs.join(".");
                    self.path_forbidden(&w)?;
                    self.advance(); // [
                    self.construct_body(w, line)
                }
                _ if self.is_construct() => {
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), _ => unreachable!() }
                        if *self.peek() == Tok::Dot { self.advance(); continue; }
                        break;
                    }
                    let w = self.type_name(&segs)?;
                    self.advance(); // [
                    self.construct_body(w, line)
                }
                _ => Ok(Expr::Call(self.call()?)),
            },
            Tok::Eof => self.err("unexpected end of file inside a value"),
            other => self.err(format!("expected a value, found {other}")),
        }
    }
}

fn bad_ptr<T>(p: &Parser) -> PResult<T> {
    p.err("`ptr[T]` exists only at the boundary: in an `extern.C fn` signature or a `layout.C struct` field (design/ffi.md); a pointer is held with `:=`, never declared")
}

/// Module names (decided 2026-09-19, "whatever has most compat"): lowercase
/// ASCII letters, digits, `_`, starting with a letter; not a Windows device
/// name; not a keyword. The same rule for file names and `module` blocks.
pub fn module_name_ok(name: &str) -> Result<(), String> {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        Some(c) if c.is_ascii_uppercase() => return Err("lowercase only — macOS and Windows file systems do not tell `Shapes` from `shapes`".into()),
        _ => return Err("it must start with a lowercase ASCII letter".into()),
    }
    if let Some(c) = name.chars().find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')) {
        return Err(format!("`{c}` is not allowed — lowercase ASCII letters, digits, and `_` only"));
    }
    const DEVICES: &[&str] = &["con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9"];
    if DEVICES.contains(&name) { return Err("it is a reserved device name on Windows".into()); }
    if KEYWORDS.contains(&name) { return Err("it is a keyword".into()); }
    Ok(())
}

/// Other names for CyborgPL's types (owner, 2026-10-02): Rust's number names, and the brain
/// float's `bfloat16` / `bf16` — the only aliases a type has.
pub fn rust_alias(name: &str) -> Option<&'static str> {
    Some(match name {
        "bfloat16" | "bf16" => "brain16",
        "i8" => "int8", "i16" => "int16", "i32" => "int32", "i64" => "int64", "i128" => "int128",
        "u8" => "uint8", "u16" => "uint16", "u32" => "uint32", "u64" => "uint64", "u128" => "uint128",
        "f16" => "bin16", "f32" => "bin32", "f64" => "bin64",
        _ => return None,
    })
}
