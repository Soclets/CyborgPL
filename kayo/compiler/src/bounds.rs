//! `coll.bound` (owner, 2026-10-25; `design/collections.md`): named bounded
//! types. `coll.bound quantity { quantity = i32[min: 1, max: 99];
//! quantity.small = i32[min: 1, max: 9]; }` names two types, and a use of a
//! name *is* that type, not a new one, so values move between them freely.
//! Read, as a `use` is, as a rewrite of the file's tokens before parsing: each
//! block is taken out and each name written as its type wherever it starts a
//! path, so a type anywhere may name one. Every line writes its full name;
//! only a bounded type, one with a limit after it, is named.
//!
//! Every file of a kit is read for its bounds first (owner, 2026-10-26),
//! each under its module's path: bare in its own module, the root's as
//! `my.quantity` from anywhere in the kit, a module's by its path,
//! `shop.order.discount`, from outside it only when `pub` or `pub.kit`, as
//! the module's other items. Another kit's use (`kit::q`) is not built, as in
//! the oracle.

use crate::diag::{err, Diagnostic, Result};
use crate::lexer::{Tok, Token};
use crate::source::FileId;

/// One named bound of the kit: its module's path (the root's is empty), its
/// name's tokens and its type's, and whether it is `pub`.
#[derive(Clone, Debug)]
pub struct Bound {
    pub module: Vec<String>,
    pub name: Vec<Token>,
    pub ty: Vec<Token>,
    pub public: bool,
}

/// The kit being read: every file's bounds, and each file's module path.
#[derive(Clone, Debug, Default)]
pub struct Kit {
    pub bounds: Vec<Bound>,
    pub files: Vec<(FileId, Vec<String>)>,
}

thread_local! {
    static KIT: std::cell::RefCell<Kit> = std::cell::RefCell::new(Kit::default());
}

/// The kit's bounds for its files to be read with, and the ones before.
pub fn set_kit(kit: Kit) -> Kit {
    KIT.with(|k| std::mem::replace(&mut *k.borrow_mut(), kit))
}

/// A path's end at `i`: words joined by `.`.
fn path_end(toks: &[Token], mut i: usize) -> usize {
    if !matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Word(_))) {
        return i;
    }
    i += 1;
    while matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Dot)) && matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Word(_))) {
        i += 2;
    }
    i
}

fn spelled(toks: &[Token]) -> String {
    toks.iter().map(|t| match &t.tok {
        Tok::Word(w) => w.clone(),
        Tok::Dot => ".".into(),
        _ => String::new(),
    }).collect()
}

fn words(toks: &[Token]) -> Vec<String> {
    toks.iter().filter_map(|t| match &t.tok {
        Tok::Word(w) => Some(w.clone()),
        _ => None,
    }).collect()
}

/// Whether a type's tokens put a limit on it: `min:`, `max:`, `max_places:`
/// or `min_places:` inside a bracket.
fn bounded(ty: &[Token]) -> bool {
    let mut depth = 0i32;
    for (k, t) in ty.iter().enumerate() {
        match &t.tok {
            Tok::LBracket => depth += 1,
            Tok::RBracket => depth -= 1,
            Tok::Word(w) if depth > 0 && matches!(w.as_str(), "min" | "max" | "max_places" | "min_places") && matches!(ty.get(k + 1).map(|t| &t.tok), Some(Tok::Colon)) => return true,
            _ => {}
        }
    }
    false
}

/// The module blocks open at each token, as a path under `base`: each
/// `module till {` pushes `till` until its `}`.
fn scopes(toks: &[Token], base: &[String]) -> Vec<Vec<String>> {
    let mut out = Vec::with_capacity(toks.len());
    let mut stack: Vec<(String, i32)> = Vec::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < toks.len() {
        let here: Vec<String> = base.iter().cloned().chain(stack.iter().map(|(n, _)| n.clone())).collect();
        match &toks[i].tok {
            Tok::Word(w) if w == "module" => {
                if let (Some(Tok::Word(n)), Some(Tok::LBrace)) = (toks.get(i + 1).map(|t| &t.tok), toks.get(i + 2).map(|t| &t.tok)) {
                    out.push(here.clone());
                    out.push(here.clone());
                    out.push(here);
                    depth += 1;
                    stack.push((n.clone(), depth));
                    i += 3;
                    continue;
                }
            }
            Tok::LBrace => depth += 1,
            Tok::RBrace => {
                if stack.last().is_some_and(|(_, d)| *d == depth) {
                    stack.pop();
                }
                depth -= 1;
            }
            _ => {}
        }
        out.push(here);
        i += 1;
    }
    out
}

/// A file's tokens with its `coll.bound` blocks taken out, and the bounds,
/// each under its module's path (`base`, and the module blocks it is in).
pub fn blocks(toks: &[Token], base: &[String]) -> Result<(Vec<Token>, Vec<Bound>)> {
    let is_word = |i: usize, w: &str| matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Word(x)) if x == w);
    let scope = scopes(toks, base);
    let mut bounds: Vec<Bound> = Vec::new();
    let mut kept: Vec<Token> = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
        let starts = is_word(i, "coll") && matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Dot)) && is_word(i + 2, "bound");
        if !starts {
            kept.push(toks[i].clone());
            i += 1;
            continue;
        }
        let at = toks[i].span;
        let module = scope.get(i).cloned().unwrap_or_default();
        // `pub` (or `pub.kit`) in front: the rest of the kit may name it.
        let mut public = false;
        if kept.last().is_some_and(|t| matches!(&t.tok, Tok::Word(w) if w == "pub")) {
            kept.pop();
            public = true;
        } else if kept.len() >= 3 && matches!(&kept[kept.len() - 3].tok, Tok::Word(w) if w == "pub") && kept[kept.len() - 2].tok == Tok::Dot && matches!(&kept[kept.len() - 1].tok, Tok::Word(w) if w == "kit") {
            kept.truncate(kept.len() - 3);
            public = true;
        }
        let Some(Tok::Word(name)) = toks.get(i + 3).map(|t| t.tok.clone()) else {
            return err(Diagnostic::error("a `coll.bound` says its name, then its members in `{ }`: `coll.bound quantity { quantity = i32[min: 1, max: 99]; }`").named("collection_form").at(at));
        };
        if !matches!(toks.get(i + 4).map(|t| &t.tok), Some(Tok::LBrace)) {
            return err(Diagnostic::error(format!("`coll.bound {}` holds its members in `{{ }}`", name)).named("collection_form").at(toks[i + 3].span));
        }
        let mut k = i + 5;
        loop {
            match toks.get(k).map(|t| &t.tok) {
                Some(Tok::RBrace) => {
                    k += 1;
                    break;
                }
                None | Some(Tok::Eof) => return err(Diagnostic::error(format!("`coll.bound {}`'s `{{` is never closed", name)).named("unclosed_block").at(at)),
                _ => {}
            }
            let n1 = path_end(toks, k);
            let member = toks[k..n1].to_vec();
            if member.is_empty() || !matches!(toks.get(n1).map(|t| &t.tok), Some(Tok::Op(o)) if o == "=") {
                return err(Diagnostic::error(format!("a member of `coll.bound {}` is its full name, `=`, and a bounded type, then `;`: `{}.small = i32[min: 1, max: 9];`", name, name)).named("collection_form").at(toks.get(k).map_or(at, |t| t.span)));
            }
            let full = spelled(&member);
            if !(full == name || full.starts_with(&format!("{}.", name))) {
                return err(Diagnostic::error(format!("a member writes the collection's name in front: `{}.{} = …;`", name, full)).named("collection_member_unprefixed").at(member[0].span).fix_edit(member[0].span, crate::diag::Edit::Insert, format!("{}.", name)));
            }
            if bounds.iter().any(|b| b.module == module && spelled(&b.name) == full) {
                return err(Diagnostic::error(format!("`{}` is named twice: a `coll.bound` names each type once", full)).named("defined_twice").at(member[0].span));
            }
            // The type, to its `;` outside every bracket.
            let mut t1 = n1 + 1;
            let mut depth = 0i32;
            while let Some(t) = toks.get(t1) {
                match &t.tok {
                    Tok::LBracket | Tok::LParen | Tok::LBrace => depth += 1,
                    Tok::RBracket | Tok::RParen | Tok::RBrace if depth > 0 => depth -= 1,
                    Tok::Semi if depth == 0 => break,
                    Tok::RBrace | Tok::Eof => return err(Diagnostic::error(format!("`{} = …` ends with `;`", full)).named("semicolon_missing").at(t.span)),
                    _ => {}
                }
                t1 += 1;
            }
            let ty = toks[n1 + 1..t1].to_vec();
            if !bounded(&ty) {
                return err(Diagnostic::error(format!("`{}` limits nothing: a `coll.bound` names a bounded type, a type with a limit after it (`i32[min: 1, max: 99]`, `str.utf8[max: 40 %graphemes]`, `std::list[i32][max: 3]`)", full)).named("bound_type_unbounded").at(member[0].span));
            }
            bounds.push(Bound { module: module.clone(), name: member, ty, public });
            k = t1 + 1;
        }
        i = k;
    }
    Ok((kept, bounds))
}

pub fn rewrite(toks: &[Token]) -> Result<Vec<Token>> {
    let kit = KIT.with(|k| k.borrow().clone());
    let file = toks.first().map(|t| t.span.file);
    let base: Vec<String> = file.and_then(|f| kit.files.iter().find(|(id, _)| *id == f).map(|(_, m)| m.clone())).unwrap_or_default();
    let has_block = toks.windows(3).any(|w| matches!(&w[0].tok, Tok::Word(x) if x == "coll") && w[1].tok == Tok::Dot && matches!(&w[2].tok, Tok::Word(x) if x == "bound"));
    if !has_block && kit.bounds.is_empty() {
        return Ok(toks.to_vec());
    }
    let (kept, own) = blocks(toks, &base)?;
    // This file's own bounds count even when its kit was not read first (a
    // lone file); when it was, the kit's table has them already.
    let mut all: Vec<Bound> = kit.bounds.clone();
    for b in own {
        if !all.iter().any(|x| x.module == b.module && spelled(&x.name) == spelled(&b.name)) {
            all.push(b);
        }
    }
    // Longest names first: `quantity.small` before `quantity`.
    all.sort_by_key(|b| std::cmp::Reverse(b.module.len() + b.name.len()));
    substitute(&kept, &all, &base)
}

/// Each bound's name written as its type where it starts a path: bare in its
/// own module, `my.` and the name for the root's, the module's path and the
/// name for any other's.
fn substitute(kept: &[Token], bounds: &[Bound], base: &[String]) -> Result<Vec<Token>> {
    let scope = scopes(kept, base);
    let mut out: Vec<Token> = Vec::with_capacity(kept.len());
    // The words `want`, joined by `.`, at `at`, and no longer path.
    let matches_at = |at: usize, want: &[String]| -> bool {
        if want.is_empty() {
            return false;
        }
        let end = at + want.len() * 2 - 1;
        kept.get(at..end).is_some_and(|w| w.iter().enumerate().all(|(n, t)| if n % 2 == 0 { matches!(&t.tok, Tok::Word(x) if *x == want[n / 2]) } else { t.tok == Tok::Dot }))
            && !matches!(kept.get(end).map(|t| &t.tok), Some(Tok::Dot | Tok::ColonColon))
    };
    let mut k = 0;
    while k < kept.len() {
        let t = &kept[k];
        let head = k == 0 || !matches!(kept[k - 1].tok, Tok::Dot | Tok::ColonColon);
        if !head || !matches!(t.tok, Tok::Word(_)) {
            out.push(t.clone());
            k += 1;
            continue;
        }
        let here = scope.get(k).cloned().unwrap_or_default();
        // After `:` or `->` a name is a type, whatever follows it.
        let typed = k > 0 && (matches!(kept[k - 1].tok, Tok::Colon) || matches!(&kept[k - 1].tok, Tok::Op(o) if o == "->"));
        let mut found: Option<(&Bound, usize)> = None;
        for b in bounds {
            let name = words(&b.name);
            // The root's, `my.` and its name, from anywhere in the kit.
            let mine: Vec<String> = std::iter::once("my".to_string()).chain(name.iter().cloned()).collect();
            if b.module.is_empty() && matches_at(k, &mine) {
                found = Some((b, mine.len() * 2 - 1));
                break;
            }
            // A module's, by its path, from anywhere in the kit.
            if !b.module.is_empty() {
                let full: Vec<String> = b.module.iter().cloned().chain(name.iter().cloned()).collect();
                if matches_at(k, &full) {
                    if !b.public && !here.starts_with(&b.module) {
                        return err(Diagnostic::error(format!("`{}` is private to `{}`; mark its `coll.bound` `pub` or `pub.kit`", full.join("."), b.module.join("."))).named("not_public").at(t.span));
                    }
                    found = Some((b, full.len() * 2 - 1));
                    break;
                }
            }
            // Bare in its own module; not a field of that name (`quantity =
            // 4`, a label) unless it follows `:`.
            if b.module == here && matches_at(k, &name) {
                let end = k + name.len() * 2 - 1;
                let field = matches!(kept.get(end).map(|t| &t.tok), Some(Tok::Colon)) || matches!(kept.get(end).map(|t| &t.tok), Some(Tok::Op(o)) if o == "=");
                if typed || !field {
                    found = Some((b, name.len() * 2 - 1));
                    break;
                }
            }
        }
        if let Some((b, len)) = found {
            for p in &b.ty {
                let mut p = p.clone();
                p.span = t.span;
                p.line_first = false;
                out.push(p);
            }
            if let Some(first) = out.len().checked_sub(b.ty.len()) {
                out[first].line_first = t.line_first;
            }
            k += len;
            continue;
        }
        out.push(t.clone());
        k += 1;
    }
    Ok(out)
}
