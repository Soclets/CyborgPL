//! The Precog oracle's claim check (design/precog.md, attacks (b) and (c)):
//! read every site CCA calls proven (`cca precog --json`), find it in the
//! generator's own tree by its span, and ask a fresh Z3 — through check
//! claude's own translation (check_gen::solve), over every input Precog
//! considers — whether it can fail, and whether its value can fall outside
//! the set or interval Precog claimed for it. A "yes" is a refuted claim,
//! with the input that refutes it.

use crate::engines::{self, Tools};
use crate::json::{self, J};
use check_gen::ast::{Program, Spans};
use check_gen::solve::{encode_with, parse_answers, Answer, Encoded, Test};
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct Claim {
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub kind: String,
    pub construct: String,
    pub verdict: String,
    pub layer: String,
    pub function: String,
    /// The value Precog claims: a set, an interval, or nothing to compare.
    pub value: Value,
    /// For a landing: the type it must fit — base name and range.
    pub fits: Option<(String, i128, i128)>,
}

#[derive(Clone, Debug)]
pub enum Value {
    Set(Vec<i128>),
    Interval(i128, i128),
    Other,
}

pub fn read_claims(text: &str) -> Vec<Claim> {
    let mut out = vec![];
    for l in text.lines() {
        let Some(j) = json::parse(l.trim()) else { continue };
        let pos = |k: &str| -> Option<(u32, u32)> {
            let a = j.get(k)?.arr()?;
            Some((a.first()?.int()? as u32, a.get(1)?.int()? as u32))
        };
        let (Some((line, col)), Some((end_line, end_col))) = (pos("start"), pos("end")) else { continue };
        let s = |k: &str| j.get(k).and_then(J::str).unwrap_or("").to_string();
        let value = match j.get("value") {
            Some(v) if v.get("values").is_some() => Value::Set(v.get("values").and_then(J::arr).unwrap_or(&[]).iter().filter_map(J::int).collect()),
            Some(v) if v.get("lo").is_some() => match (v.get("lo").and_then(J::int), v.get("hi").and_then(J::int)) {
                (Some(lo), Some(hi)) => Value::Interval(lo, hi),
                _ => Value::Other,
            },
            _ => Value::Other,
        };
        let fits = match (j.get("fits").and_then(J::str), j.get("range")) {
            (Some(f), Some(r)) => match (r.get("lo").and_then(J::int), r.get("hi").and_then(J::int)) {
                (Some(lo), Some(hi)) => Some((f.split('[').next().unwrap_or(f).to_string(), lo, hi)),
                _ => None,
            },
            _ => None,
        };
        out.push(Claim {
            fits,
            line,
            col,
            end_line,
            end_col,
            kind: s("kind"),
            construct: s("construct"),
            verdict: s("verdict"),
            layer: s("layer"),
            function: s("function"),
            value,
        });
    }
    out
}

/// My sites a claim can be about: same kind (allowing the few names that
/// differ), and the same span as the site's expression or statement.
pub fn matches(c: &Claim, e: &Encoded, spans: &Spans) -> Vec<usize> {
    if c.line != c.end_line {
        return vec![];
    }
    // CCA's `overflow` on an operator is my check on that operator (mine
    // asks the weaker "past 64 bits", so it can never refute falsely).
    let kind_ok = |k: &str| k == c.kind || (c.kind == "overflow" && matches!(k, "shift_width" | "past_64_bits"));
    e.sites
        .iter()
        .enumerate()
        .filter(|(_, s)| kind_ok(s.kind))
        // a landing must be into the same type (CCA gives every argument of a
        // call the call's span, so the type is what tells them apart)
        .filter(|(_, s)| match (&c.fits, s.fits) {
            (Some((cb, clo, chi)), Some((sb, slo, shi))) if c.kind == "narrowing" => cb == sb && *clo == slo && *chi == shi,
            _ => true,
        })
        .filter(|(_, s)| {
            let hit = |sp: Option<&(u32, u32, u32)>| sp.map_or(false, |&(l, a, b)| l == c.line && a == c.col && b == c.end_col);
            if s.kind != "narrowing" {
                // an operator: its own expression's span
                return hit(spans.expr.get(&s.node));
            }
            if let Some(&(l, a0, a1, b)) = spans.stmt.get(&s.stmt) {
                // a landing in a statement (let, =, return, print): CCA gives
                // the statement's span, with or without its tier word or `;`,
                // or, for a print's argument, the argument's own (034493a)
                return (l == c.line && (a0 == c.col || a1 == c.col) && (b == c.end_col || b - 1 == c.end_col)) || hit(spans.expr.get(&s.node));
            }
            if s.stmt != 0 {
                // an argument: its own span (CCA 034493a); before that, the
                // call's, which two arguments of one type shared
                return hit(spans.expr.get(&s.node)) || hit(spans.expr.get(&s.stmt));
            }
            // a conversion: its own span
            hit(spans.expr.get(&s.node))
        })
        .map(|(k, _)| k)
        .collect()
}

pub struct Refuted {
    pub claim: Claim,
    pub test: &'static str,
    pub inputs: Vec<u64>,
    pub site: String,
}

pub struct Outcome {
    pub proven: usize,
    pub matched: usize,
    /// Claims attack (b)'s simple intervals already justify: not asked.
    pub justified: usize,
    /// Whether attack (b) followed the whole program (else it justifies nothing).
    pub intervals_complete: bool,
    pub asked: usize,
    /// Z3 gave up on the question within its budget.
    pub unknown: usize,
    /// Z3 was stopped at the time limit before reaching the question.
    pub unasked: usize,
    pub refuted: Vec<Refuted>,
    /// Refutations the exact evaluator did not confirm on the solver's
    /// input: artefacts of the translation's approximations, not findings.
    pub unconfirmed: usize,
    pub unmatched: Vec<Claim>,
    /// Claims whose span and type fit more than one of my sites (two
    /// arguments of one call with the same type): not asked, since the
    /// claim is about only one of them (seed 8049, 2026-09-29).
    pub ambiguous: usize,
    /// Claims with nothing to ask: a negative number's minus, a `for`
    /// counter over literal bounds, the program's own printer.
    pub literal: usize,
}

/// Ask CCA for its claims on `p`, and Z3 whether each proven one holds.
pub fn check_claims(t: &Tools, p: &Program, dir: &Path) -> Result<Outcome, String> {
    if p.unevaluable {
        return Err("holds values the solver does not model (deci, bin, text)".into());
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let (text, spans) = p.render_spans();
    let file = dir.join("prog.cpls");
    std::fs::write(&file, &text).map_err(|e| e.to_string())?;
    let r = engines::run(Command::new(&t.cca).arg("precog").arg("--json").arg(&file), dir, &[], false, engines::BUILD_LIMIT);
    if r.timed_out {
        return Err("cca precog ran past its limit".into());
    }
    let claims = read_claims(&String::from_utf8_lossy(&r.stdout));
    // Every input Precog considers: any length a uint64 holds.
    let enc = encode_with(p, u64::MAX as u128).ok_or("too big to translate")?;
    let mut questions = vec![];
    let mut about = vec![];
    let ranges = check_gen::intervals::analyse(p, u64::MAX as u128);
    let mut out = Outcome { proven: 0, matched: 0, justified: 0, intervals_complete: ranges.is_some(), asked: 0, unknown: 0, unasked: 0, refuted: vec![], unconfirmed: 0, unmatched: vec![], literal: 0, ambiguous: 0 };
    let lines: Vec<&str> = text.lines().collect();
    // The text a claim's span covers.
    let span_text = |c: &Claim| -> String {
        lines.get(c.line as usize - 1).and_then(|l| l.get(c.col as usize - 1..(c.end_col as usize - 1).min(l.len()))).unwrap_or("").to_string()
    };
    for c in claims.iter().filter(|c| c.verdict == "proven") {
        out.proven += 1;
        let ks = matches(c, &enc, &spans);
        if ks.is_empty() {
            // A negative literal (CCA sees a minus on a number; my tree holds
            // the number) or a `for` counter (literal bounds): nothing to ask.
            let t = span_text(c);
            let t = t.trim_matches(|ch| ch == '(' || ch == ')');
            let neg_lit = t.starts_with('-') && t[1..].chars().all(|ch| ch.is_ascii_digit()) && t.len() > 1;
            // bare numbers alone: a constant, folded exactly (operators.md)
            let bare_group = !span_text(c).contains('\'') && !span_text(c).contains('[') && !span_text(c).is_empty();
            let for_counter = lines.get(c.line as usize - 1).map_or(false, |l| l.trim_start().starts_with("for ")) && c.end_col == c.col + 1;
            // The program's own printer comes before the generated code.
            let in_printer = lines.iter().position(|l| l.starts_with("fn f") || l.starts_with("MAIN")).map_or(false, |first| (c.line as usize) <= first);
            if neg_lit || bare_group || for_counter || in_printer {
                out.literal += 1;
            } else {
                out.unmatched.push(c.clone());
            }
            continue;
        }
        out.matched += 1;
        // Several instances of one site (a function inlined at each call)
        // share its node and statement; different sites do not.
        let mut groups: Vec<(usize, usize)> = ks.iter().map(|k| (enc.sites[*k].node, enc.sites[*k].stmt)).collect();
        groups.sort();
        groups.dedup();
        if groups.len() > 1 {
            out.ambiguous += 1;
            continue;
        }
        // Attack (b): does the simple analysis keep every instance's value
        // inside what Precog claims? Then the claim is justified.
        let inside = |k: usize| -> bool {
            let Some(rs) = &ranges else { return false };
            let s = &enc.sites[k];
            if !matches!(s.kind, "narrowing" | "past_64_bits" | "division_by_zero" | "modulo_by_zero" | "shift_count" | "negative_exponent") {
                return false;
            }
            let Some(&(lo, hi)) = rs.get(&(s.node, s.stmt, s.kind)) else { return false };
            match &c.value {
                Value::Interval(a, b) => *a <= lo && hi <= *b,
                Value::Set(vs) => hi - lo < 4096 && (lo..=hi).all(|v| vs.binary_search(&v).is_ok() || vs.contains(&v)),
                Value::Other => false,
            }
        };
        if ks.iter().all(|k| inside(*k)) {
            out.justified += 1;
            continue;
        }
        // One question per claim, about every instance at once. A claimed
        // set or interval lies inside what the site must fit, so "can the
        // value fall outside the claim" covers "can it fail"; only a claim
        // without one (layer 3, "solver") is asked whether it can fail.
        // `shift_width`'s value means something else in CCA than in my
        // translation (the shifted result), so it is only asked "can it fail".
        let first = ks[0];
        let t = match &c.value {
            _ if c.kind == "shift_width" || c.kind == "overflow" && c.construct == "shl" => None,
            Value::Set(v) if !v.is_empty() => Some(Test::OutsideSet(v.clone())),
            Value::Interval(lo, hi) => Some(Test::OutsideInterval(*lo, *hi)),
            _ => None,
        };
        match t {
            Some(t) => {
                questions.push((ks, t));
                about.push((c.clone(), "value outside the claim", first));
            }
            None => {
                questions.push((ks, Test::Fails));
                about.push((c.clone(), "can fail", first));
            }
        }
    }
    out.asked = questions.len();
    if questions.is_empty() {
        return Ok(out);
    }
    let f = dir.join("claims.smt2");
    std::fs::write(&f, enc.claim_script(&questions)).map_err(|e| e.to_string())?;
    let z = engines::run(Command::new("z3").arg("-smt2").arg(&f), dir, &[("PATH".into(), "/opt/homebrew/bin:/usr/bin:/bin".into())], false, engines::CLAIM_LIMIT);
    let answers = parse_answers(&String::from_utf8_lossy(&z.stdout), questions.len(), enc.inputs.len());
    for (k, a) in answers.iter().enumerate() {
        match a {
            Answer::Fails(v) => {
                let (claim, test, site) = &about[k];
                // Confirm on the actual program: run it exactly on the
                // solver's input and look at what the claimed site saw.
                let run = check_gen::eval::run(p, v);
                let keys: Vec<(usize, usize, &'static str)> = questions[k].0.iter().map(|i| {
                    let s = &enc.sites[*i];
                    (s.node, s.stmt, s.kind)
                }).collect();
                let confirmed = !run.exhausted
                    && match &questions[k].1 {
                        Test::Fails => run.stopped.map_or(false, |st| keys.contains(&st)),
                        t => keys.iter().any(|key| {
                            run.seen.get(key).map_or(false, |vals| {
                                vals.iter().any(|x| match t {
                                    Test::OutsideSet(set) => !set.contains(x),
                                    Test::OutsideInterval(lo, hi) => x < lo || x > hi,
                                    Test::Fails => false,
                                })
                            })
                        }),
                    };
                if confirmed {
                    out.refuted.push(Refuted { claim: claim.clone(), test, inputs: v.clone(), site: enc.sites[*site].what.clone() });
                } else {
                    out.unconfirmed += 1;
                }
            }
            Answer::Unknown => out.unknown += 1,
            Answer::Never => {}
        }
    }
    out.unasked += questions.len().saturating_sub(answers.len());
    Ok(out)
}
