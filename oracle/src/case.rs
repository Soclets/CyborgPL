//! `case.insensitive` / `case.sensitive` (owner, 2026-10-06): a Limb's writer may make its names
//! case-insensitive, in its own code and as reached from other Limbs; the default is `case.sensitive`;
//! marks that disagree are an error; insensitive means Unicode's full case folding. The oracle reads the
//! program once, works out which names are insensitive, and then rewrites every spelling of such a name
//! to the one it was declared with before the program is read again, so the rest of the oracle stays
//! exact. Keywords, std's words and the built-in types stay exact.

use crate::ast::{self, BareTarget, Program};
use crate::lexer::{Tok, Token};
use std::collections::HashMap;

/// Unicode's full case folding, as the oracle approximates it: upper, then lower (`ß` → `SS` → `ss`,
/// `É` → `é`, final `ς` → `Σ` → `σ`).
pub fn fold(s: &str) -> String { s.to_uppercase().to_lowercase() }

#[derive(Default, Debug)]
pub struct LimbCase {
    /// covered bare names in this Limb: fn, struct, enum, interface, case, field (fold → as declared)
    words: HashMap<String, String>,
    /// those another Limb may reach with the path, and the fields and cases of its pub types
    pub_words: HashMap<String, String>,
    /// those another Limb may write without the path (`bare.optional` / `bare.only`)
    bare_words: HashMap<String, String>,
    /// the fields of its pub structs and the cases of its pub enums, met without a path
    members: HashMap<String, String>,
    /// covered consts (fold → as declared), and the pub ones
    vars: HashMap<String, String>,
    pub_vars: HashMap<String, String>,
    /// local variables: insensitive across the Limb, or in these modules (and their blocks)
    locals_all: bool,
    local_modules: Vec<Vec<String>>,
}

thread_local! {
    pub static PLAN: std::cell::RefCell<HashMap<String, LimbCase>> = std::cell::RefCell::new(HashMap::new());
}

fn root_key(module: &[String]) -> String { module.first().filter(|r| r.starts_with("::")).cloned().unwrap_or_default() }

fn word_mark(i: bool) -> &'static str { if i { "case.insensitive" } else { "case.sensitive" } }

/// Is a thing of `kind`, `last` in `module`, insensitive? None marks: sensitive. Marks that disagree: an error.
fn covered(prog: &Program, kind: &str, module: &[String], last: Option<&str>, line: usize, shown: &str) -> Result<bool, String> {
    let block = |m: &[String]| prog.modules.iter().any(|d| d.path == m && d.line > 0);
    let root: Vec<String> = match module.first() { Some(r) if r.starts_with("::") => vec![r.clone()], _ => Vec::new() };
    let marks: Vec<&ast::CaseMark> = prog.case_marks.iter().filter(|m| match &m.target {
        BareTarget::Item(k, n) => last.is_some() && (k == kind || (k == "op" && kind == "fn")) && *n == module.iter().map(String::as_str).chain(last).collect::<Vec<_>>().join("."),
        BareTarget::Block(p) => module.starts_with(p),
        BareTarget::File(p) => module.starts_with(p) && (p.len()..module.len()).all(|i| block(&module[..=i])),
        BareTarget::Kind(k) => k == kind && m.limb_root == root,
    }).collect();
    let Some(first) = marks.first() else { return Ok(false) };
    if let Some(other) = marks.iter().find(|m| m.insensitive != first.insensitive) {
        return Err(format!("line {line}: {shown} is marked both `{}` ({}) and `{}` ({}): marks that disagree are an error (owner, 2026-10-06)", word_mark(first.insensitive), crate::interp::line_of(first.line), word_mark(other.insensitive), crate::interp::line_of(other.line)));
    }
    Ok(first.insensitive)
}

/// Work out the plan from a first reading of the whole program. `bare` holds the `bare` marks
/// (`check_bare`'s keys, `kind:path`).
pub fn plan(prog: &Program, bare: &HashMap<String, ast::Bare>) -> Result<HashMap<String, LimbCase>, String> {
    let mut plan: HashMap<String, LimbCase> = HashMap::new();
    if prog.case_marks.is_empty() { return Ok(plan); }
    // every bare name each Limb declares, to refuse two that differ only in case
    let mut declared: HashMap<String, Vec<(String, usize)>> = HashMap::new();
    // (root, word?, fold, as declared, line, pub, bare-reachable, a member)
    let mut found: Vec<(String, bool, String, String, usize, bool, bool, bool)> = Vec::new();
    let last_of = |n: &str| n.rsplit('.').next().unwrap_or(n).to_string();
    let is_bare = |kind: &str, module: &[String], last: &str| matches!(bare.get(&crate::interp::bare_key(kind, module, last)), Some(ast::Bare::Optional | ast::Bare::Only));
    let add = |root: String, decl: &mut HashMap<String, Vec<(String, usize)>>, name: &str, line: usize| decl.entry(root).or_default().push((name.to_string(), line));
    for f in &prog.funcs {
        let last = last_of(&f.name);
        let root = root_key(&f.module);
        add(root.clone(), &mut declared, &last, f.line);
        for (t, _) in &f.bounds { add(root.clone(), &mut declared, t, f.line); }
        let kind = if f.is_op { "op" } else { "fn" };
        if covered(prog, "fn", &f.module, Some(&last), f.line, &format!("`{kind} {last}`"))? {
            found.push((root, true, fold(&last), last.clone(), f.line, f.vis == ast::Vis::Pub, is_bare(kind, &f.module, &last), false));
        }
    }
    for d in &prog.structs {
        let last = last_of(&d.name);
        let root = root_key(&d.module);
        add(root.clone(), &mut declared, &last, d.line);
        for p in &d.params { add(root.clone(), &mut declared, p, d.line); }
        let public = d.vis == ast::Vis::Pub;
        if covered(prog, "struct", &d.module, Some(&last), d.line, &format!("`struct {last}`"))? {
            found.push((root.clone(), true, fold(&last), last.clone(), d.line, public, is_bare("struct", &d.module, &last), false));
        }
        for fld in &d.fields {
            add(root.clone(), &mut declared, &fld.name, d.line);
            if covered(prog, "field", &d.module, None, d.line, &format!("the field `{}` of `{last}`", fld.name))? {
                found.push((root.clone(), true, fold(&fld.name), fld.name.clone(), d.line, public, false, true));
            }
        }
    }
    for d in &prog.enums {
        let last = last_of(&d.name);
        let root = root_key(&d.module);
        add(root.clone(), &mut declared, &last, d.line);
        for p in &d.params { add(root.clone(), &mut declared, p, d.line); }
        let public = d.vis == ast::Vis::Pub;
        if covered(prog, "enum", &d.module, Some(&last), d.line, &format!("`enum {last}`"))? {
            found.push((root.clone(), true, fold(&last), last.clone(), d.line, public, is_bare("enum", &d.module, &last), false));
        }
        for c in &d.cases {
            add(root.clone(), &mut declared, &c.name, d.line);
            if covered(prog, "case", &d.module, None, d.line, &format!("the case `{}` of `{last}`", c.name))? {
                found.push((root.clone(), true, fold(&c.name), c.name.clone(), d.line, public, false, true));
            }
        }
    }
    for d in &prog.interfaces {
        let last = last_of(&d.name);
        let root = root_key(&d.module);
        add(root.clone(), &mut declared, &last, d.line);
        if covered(prog, "interface", &d.module, Some(&last), d.line, &format!("`interface {last}`"))? {
            found.push((root, true, fold(&last), last.clone(), d.line, d.vis == ast::Vis::Pub, is_bare("interface", &d.module, &last), false));
        }
    }
    let mut consts: HashMap<String, Vec<(String, usize)>> = HashMap::new();
    for c in &prog.consts {
        let root = root_key(&c.module);
        consts.entry(root.clone()).or_default().push((c.name.clone(), c.line));
        if covered(prog, "const", &c.module, Some(&c.name), c.line, &format!("`const '{}'`", c.name))? {
            found.push((root, false, fold(&c.name), c.name.clone(), c.line, c.vis == ast::Vis::Pub, false, false));
        }
    }
    for (root, word, f, name, line, public, bare_ok, member) in found {
        // two names that differ only in case would be one name (design claude's reading)
        let pool = if word { declared.get(&root) } else { consts.get(&root) };
        if let Some((other, oline)) = pool.and_then(|p| p.iter().find(|(o, _)| *o != name && fold(o) == f)) {
            let (a, b) = if word { (format!("`{other}`"), format!("`{name}`")) } else { (format!("`'{other}'`"), format!("`'{name}'`")) };
            return Err(format!("line {oline}: {a} and {b} ({}) differ only in case, and {b} is `case.insensitive`: they would be one name", crate::interp::line_of(line)));
        }
        let lc = plan.entry(root).or_default();
        let (all, public_map) = if word { (&mut lc.words, &mut lc.pub_words) } else { (&mut lc.vars, &mut lc.pub_vars) };
        all.insert(f.clone(), name.clone());
        if public { public_map.insert(f.clone(), name.clone()); }
        if public && member { lc.members.insert(f.clone(), name.clone()); }
        if bare_ok { lc.bare_words.insert(f, name); }
    }
    // local variables: a `var` kind mark, or a file or block module marked insensitive
    for m in &prog.case_marks {
        let root = m.limb_root.first().cloned().unwrap_or_default();
        match &m.target {
            BareTarget::Kind(k) if k == "var" && m.insensitive => plan.entry(root).or_default().locals_all = true,
            BareTarget::File(p) | BareTarget::Block(p) if m.insensitive => plan.entry(root).or_default().local_modules.push(p.clone()),
            _ => {}
        }
    }
    // a `var` mark and a module mark that disagree, over the variables in that module
    for m in &prog.case_marks {
        if let (BareTarget::File(p) | BareTarget::Block(p), false) = (&m.target, m.insensitive) {
            if let Some(k) = prog.case_marks.iter().find(|k| matches!(&k.target, BareTarget::Kind(x) if x == "var") && k.insensitive && k.limb_root == m.limb_root) {
                return Err(format!("line {}: the variables of `{}` are marked both `case.insensitive` ({}) and `case.sensitive` ({}): marks that disagree are an error (owner, 2026-10-06)", m.line, p.iter().map(|s| s.trim_start_matches("::")).collect::<Vec<_>>().join("."), crate::interp::line_of(k.line), crate::interp::line_of(m.line)));
            }
        }
    }
    Ok(plan)
}

/// A word the language, std or the built-in types own: never rewritten, as keywords stay exact.
fn builtin_word(w: &str) -> bool {
    let num = |p: &str| w.strip_prefix(p).map_or(false, |d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()));
    crate::parser::KEYWORDS.contains(&w) || crate::parser::STD_BARE_HEADS.contains(&w) || w == "MAIN"
        || ["bool", "str", "char", "brain16", "std", "cca", "case", "insensitive", "sensitive", "bare", "never", "optional", "only", "not", "neg", "plus", "minus", "times", "div", "mod", "pow", "and", "or"].contains(&w)
        || num("int") || num("uint") || num("bin") || num("deci")
}

/// Rewrite one file's tokens: `limb_root` is its Limb, `module` its module.
pub fn rewrite(toks: &mut [Token], limb_root: &[String], module: &[String]) {
    PLAN.with(|plan| {
        let plan = plan.borrow();
        if plan.is_empty() { return; }
        let own_key = limb_root.first().cloned().unwrap_or_default();
        let own = plan.get(&own_key);
        // which Limb a path's segment belongs to: Some(key) after `prefix::`, "" for built-in chains
        let mut owner: Vec<Option<String>> = vec![None; toks.len()];
        // the module each token sits in, for block modules
        let mut mods: Vec<Vec<String>> = vec![module.to_vec(); toks.len()];
        let mut stack: Vec<(Vec<String>, usize)> = Vec::new();
        let mut depth = 0usize;
        let mut cur = module.to_vec();
        let mut local_names: HashMap<String, String> = HashMap::new();
        for i in 0..toks.len() {
            match &toks[i].tok {
                Tok::LBrace => {
                    depth += 1;
                    if i >= 2 && matches!(&toks[i - 2].tok, Tok::Word(w) if w == "module") { if let Tok::Word(n) = &toks[i - 1].tok { stack.push((cur.clone(), depth)); cur.push(n.clone()); } }
                }
                Tok::RBrace => { if stack.last().map_or(false, |(_, d)| *d == depth) { cur = stack.pop().unwrap().0; } depth = depth.saturating_sub(1); }
                _ => {}
            }
            mods[i] = cur.clone();
            let segment = matches!(toks[i].tok, Tok::Word(_) | Tok::Var(_));
            if !segment { continue; }
            let in_chain = i >= 2 && matches!(toks[i - 1].tok, Tok::Dot | Tok::ColonColon) && matches!(toks[i - 2].tok, Tok::Word(_) | Tok::Var(_));
            if in_chain { if let Some(o) = owner[i - 2].clone() { owner[i] = Some(o); } }
            if let Tok::Word(w) = &toks[i].tok {
                if owner[i].is_none() && toks.get(i + 1).map_or(false, |t| t.tok == Tok::ColonColon) && w != "my" { owner[i] = Some(format!("::{w}")); continue; }
                // `my.` is this Limb's root, so what follows it is this Limb's
                if owner[i].is_none() && builtin_word(w) && w != "my" { owner[i] = Some(String::new()); continue; }
                if w == "my" { continue; }
            }
            let after_dot = i >= 1 && toks[i - 1].tok == Tok::Dot;
            let new = match (&toks[i].tok, &owner[i]) {
                (Tok::Word(w), _) if builtin_word(w) => None,
                (Tok::Word(w), Some(o)) => plan.get(o).and_then(|lc| lc.pub_words.get(&fold(w))).cloned(),
                (Tok::Var(v), Some(o)) => plan.get(o).and_then(|lc| lc.pub_vars.get(&fold(v))).cloned(),
                (Tok::Word(w), None) => {
                    let f = fold(w);
                    own.and_then(|lc| lc.words.get(&f)).cloned()
                        // a field of another Limb's type, after `.`; a bare name another Limb lets be written bare
                        .or_else(|| plan.iter().filter(|(k, _)| **k != own_key).find_map(|(_, lc)| if after_dot { lc.members.get(&f) } else { lc.bare_words.get(&f).or_else(|| lc.members.get(&f)) }).cloned())
                }
                (Tok::Var(v), None) => {
                    let f = fold(v);
                    own.and_then(|lc| lc.vars.get(&f)).cloned().or_else(|| {
                        let lc = own?;
                        let here = &mods[i];
                        if lc.locals_all || lc.local_modules.iter().any(|p| here.starts_with(p)) {
                            Some(local_names.entry(f).or_insert_with(|| v.clone()).clone())
                        } else { None }
                    })
                }
                _ => None,
            };
            if let Some(n) = new {
                toks[i].tok = match &toks[i].tok { Tok::Var(_) => Tok::Var(n), _ => Tok::Word(n) };
            }
        }
    });
}
