//! Random valid programs. Valid by construction: every name is declared
//! before use, every type matches, every loop ends, and every operation
//! stays inside its type unless the generator deliberately leaves it
//! unguarded (`Config::risk`), so that both engines are also compared on
//! checks that fire.
//!
//! The generator keeps its own interval per value — a deliberately simple
//! bookkeeping that is never shown to either engine. Where an operation
//! could leave its type, an operand is folded into range with
//! `lo + (e mod w)`, which is a plain CyborgPL guardrail.

use crate::ast::*;
use crate::printer::{printer, strict_printer};
use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct Config {
    /// 1..=6: how many functions, statements, and how deep.
    pub size: u32,
    /// Read outside values (environment-variable lengths) at the start of MAIN.
    pub inputs: u32,
    /// Operations left unguarded, per thousand.
    pub risk: u64,
    /// Write `xx` (CCA does not generate it yet).
    pub pow: bool,
    /// Keep `for` loops whose last step would pass the counter type's edge
    /// (CCA refuses them; finding sent 2026-09-23).
    pub range_edge: bool,
    /// `shl`, `not.bit` and the wrapping operators on any typed operand, not
    /// only a name (owner, 2026-09-23, 88da73f; CCA refuses it as of 61a8554).
    pub width_from_context: bool,
    /// Give some names, parameters and returns a range type.
    pub ranges: bool,
    /// Aim at the 64-bit edges: mostly int64/uint64, edge literals, call
    /// results known only by their type, comparisons across types
    /// (layer 3's wrong-proof family, compiler claude 2026-09-23).
    pub edge: bool,
    /// Give the operands of shifts, bitwise and wrapping operators a size
    /// with `to.T[…]` unless they are names or literals: CCA says "not yet"
    /// to them on an exact answer (2026-09-23).
    pub sized_bits: bool,
    /// Write recursive functions: a range-typed counter parameter that
    /// counts down to 0 (or up to its top) on every call.
    pub recursion: bool,
    /// Attack (a) of the Precog oracle: narrow every input at the door of
    /// MAIN into a small typed value, so that the whole input domain (at most
    /// this many combinations) can be run. 0 = off.
    pub exhaust: u64,
    /// Lists (design/lists.md): literals, positions (check sites), `change
    /// 'xs'[p]`, `list.append`, `list.length`, `for 'i', 'x' in lend 'xs'`.
    pub lists: bool,
    /// Use a `for … in lend` item only through `to.T['x']`: the oracle gives
    /// the item no width for `+~` or `not.bit` (sent 2026-09-29).
    pub item_through_to: bool,
    /// Use a `match` arm's data name only through `to.T['d']`: the oracle
    /// gives it no width for `not.bit` or the wrapping operators (sent
    /// 2026-09-29).
    pub data_through_to: bool,
    /// Functions that hand back a borrow (`-> lent@x list[T]`, 46faa74), and
    /// chains of them in MAIN followed by a `give`, an append or a change of
    /// one of the lists.
    pub borrows: bool,
    /// Per mille of programs with borrows that get one borrow mistake on
    /// purpose (a borrow of a local, the wrong group, an unsaid borrow, or a
    /// list changed while a borrow of it is still used); both engines must
    /// refuse those (`Program::expect_refused`).
    pub borrow_wrong: u64,
    /// Enums with integer data, `==` on them, and `match` as a statement
    /// and as a value (design/enums.md; 23ab20b).
    pub enums: bool,
    /// Per mille of programs written as `#strict` files.
    pub strict: u64,
    /// simd vectors (design/simd.md): `simd.fill`, lane-wise operators kept
    /// inside the lane type, comparisons folded with `all` / `any`, and
    /// `max` / `min` / `sum`.
    pub simd: bool,
    /// Read single lanes, `'v'[k]` (CCA read wrong values, sent 2026-09-29).
    pub simd_lanes: bool,
    /// The wrapping operators on vectors (the oracle refuses them, sent
    /// 2026-09-29); without them a lane that could leave its type is made
    /// by a bitwise operator instead.
    pub simd_wrap: bool,
    /// deci values, their arithmetic and conversions, and text into
    /// numbers (design/types.md, design/conversion.md; owner "yea",
    /// 2026-09-30), watched through printed integers.
    pub deci: bool,
    /// The `to.useful.…` forms (CCA does not take the 2026-09-30 spelling yet).
    pub deci_useful: bool,
    /// `for !cheat` / `for !cheat seq` on some range loops (owner, 2026-10-01;
    /// spelled `!c` until 2b7f336).
    pub no_cheat: bool,
    /// `brain16` (owner, 2026-10-02): its own family, each step rounded,
    /// seen through conversions; never inside a `check`.
    pub brain16: bool,
    /// `mod` on brain16 (its sign for bins is open with the owner, 2026-10-02).
    pub brain_mod: bool,
    /// The roundings past `.floor`: `div.trunc`, `div.ceil`, `div.near` and
    /// their `mod`s on integers; `.floor`, `.ceil`, `.trunc` and their
    /// `mod`s on decis and brain16 (owner, 2026-10-02, 692a444, 5a12c07).
    pub roundings: bool,
    /// The `m1` call lending one list to change and another to read
    /// (38c91e7); the evaluator does not follow it.
    pub mutator: bool,
    /// The `.ieee` operators and `negate.ieee` in brain16 blocks (owner, 2026-10-07).
    pub ieee: bool,
    /// `list.remove` (owner, 2026-10-08).
    pub remove: bool,
    /// `shadow` declarations (owner, 2026-10-08), and how often in 1000 a
    /// program gets a plain `let` over a visible name instead, a mistake.
    pub shadow: bool,
    pub shadow_wrong: u64,
    /// stdkit's `str.at` / `str.slice` (CCA has no stdkit yet).
    pub std_lib: bool,
    /// A `Counter` iterator and `optional` (owner, 2026-10-09).
    pub iterators: bool,
    /// `std::print` of decis and brain16, their own text (Synapse 252fa188).
    pub print: bool,
    /// `to.decN.written[…]` from text (owner, 2026-10-09).
    pub written: bool,
    /// `const` declarations in bodies (owner, 2026-10-10).
    pub consts: bool,
    /// `!mut`, `!check` and `! 'x'` (owner, 9a9e573) on most programs.
    pub new_words: bool,
    /// Exact bare constants at 2^63, 2^64 - 1 and -2^63 landing early in a
    /// function, before its other sites (Synapse's layer 3 read 2^63 at 64
    /// bits and proved every later site vacuously, f4d99bd1; b63fed46).
    pub edge_landings: bool,
    /// `change 'i' = 'i' OP e;`, guarded to keep the name's range. Its
    /// compound spellings (`+=`, `'i'++;`) are refused since 2026-10-16
    /// (owner), so it is written in full.
    pub compound: bool,
    /// Quoted names with a symbol in them, `'v12?'`, `'d3%'`, `'v4.x'`,
    /// `'t5 a'`, `'k6\''`: any printable ASCII, single spaces between words,
    /// `\\` and `\'` escaped (owner, 2026-10-16).
    pub symbol_names: bool,
    /// Now and then a shift of a bare number in a branch never taken, in
    /// one of many places: it must be refused before the program runs
    /// wherever nothing gives it a width (4866c834; the oracle checked only
    /// where run, dfce8e06, then two shapes before running, 59c45ac6).
    pub width_probes: bool,
    /// Percent of functions that recurse, and their deepest counter
    /// (30 and 10 before 2026-10-17); `mutual` adds a pair of functions
    /// calling each other, now and then failing at a depth. For kayo's
    /// recursion inlining (9fa412d2), which must leave output unchanged.
    pub rec_rate: u64,
    pub rec_depth: i128,
    pub mutual: bool,
    /// A chain of functions each passing its `lent.mut` list on with
    /// `lend.mut` and changing it again after the call (oracle c322239b).
    pub reborrows: bool,
    /// Typed literals, `5: uint8`, for `to.uint8[5]` (owner, 2026-10-17).
    pub typed_literals: bool,
    pub std_all: bool,
    /// A `list[bool]` made by list.fill and used only by position, which kayo
    /// holds as bits (abd0ef6d); now and then handed to a call, which keeps
    /// it a byte per item. Compared with KAYO_BITS=0 by `check recref`.
    pub bool_lists: bool,
    /// Pure functions of a small whole-number parameter, which Precog may
    /// turn into a table of answers (3f065caa), sometimes with a
    /// `table_bytes` limit, called over many arguments and under `no_cheat`.
    /// Compared with KAYO_TABLES=0 by `check recref`.
    pub tables: bool,
    /// A `coll.fn` (own function, presets, a member with its body, a member
    /// pointing at a function written elsewhere) and a `coll.handler` used
    /// through `otherwise` (owner, 2026-10-19; oracle 8ae7abd3, kayo 3776553e).
    pub collections: bool,
    /// Versions of the collection's own function and of a member, in
    /// `versions` blocks (owner, 2026-10-20/21; kayo 21cd0b37).
    pub versions: bool,
    /// `ptr.null` in C calls, lists of C pointers and `layout.C` fields
    /// (owner, 2026-10-21; oracle 2c8cfb33, kayo 320605eb).
    pub ptr_null: bool,
    /// std's names with their path, `std::list[i64]`, and no `bare.` marks
    /// (owner, 2026-10-22; oracle 82d88048, kayo a42b37c4).
    pub std_paths: bool,
    /// std's glyph with its path, `'a' std::xx 2` (design claude, 2026-10-22;
    /// oracle edcbf2cb, kayo 0aced748).
    pub std_glyph: bool,
    /// Calls in `( )` and `$name` (owner, 2026-10-22; oracle c16dc2d7, kayo 33f7b3dd).
    pub paren_calls: bool,
    /// MAIN ending in `std::exit(code: N)` (owner, 2026-10-23; oracle 69c6aedf).
    pub exits: bool,
    /// A generic's bare literal filled by i32, d64 and f64 (owner, 2026-10-22;
    /// oracle 340f5b98, kayo a319a218).
    pub generics: bool,
    /// Channels from `channel.empty`, sent from `concur` tasks and bounded
    /// (owner; oracle 73a48973, kayo a319a218).
    pub channels: bool,
    /// A handler member taking an argument, `otherwise h.or_default(default: K)`
    /// (owner, 2026-10-23; oracle 07353d0f, kayo b02769fe).
    pub handler_args: bool,
    /// A print writer filled, a plain print between, flushed; `std::print.flush()`;
    /// a print with `otherwise { write_failed { … } }` (owner, 2026-10-23; kayo 6c647471).
    pub writers: bool,
    /// Short lambdas, `fn ['s'] = 's' at_least 60`, with and without `with`, and
    /// one in a typed `fn[i32] -> bool` (owner, 2026-10-24; kayo 22eb479f).
    pub short_lambdas: bool,
    /// Writers written `.whole` or `.split` (owner, 2026-10-24; oracle deda6720, kayo 64cdc5de).
    pub writer_kinds: bool,
    /// `std::print.format` and `std::format` with holes `{}` or numbered, and
    /// shapes fitting each value's kind (2026-09-22, 10-01; kayo e0437f9e).
    pub formats: bool,
    /// C interop (owner, 2026-10-19; oracle 927b1905, kayo 6ae65a5f): a
    /// `layout.C union` written through one field and read through another,
    /// a `layout.C struct` of bit-fields written at and past their limits,
    /// and C's printf between our own prints.
    pub c_interop: bool,
    /// Whole literals written in binary, hex and octal (owner, 2026-10-19).
    pub based_literals: bool,
    /// A pub enum whose values may drop its name where the type is known,
    /// `bare.enum_prefix.optional` (owner, 2026-10-20; kayo 91845e68).
    pub bare_enums: bool,
    /// Percents under `:=` and `percent[d32]`, and d64s limited by
    /// `max_places` / `min_places` with the round family (owner,
    /// 2026-10-17/20; kayo f143b8b2, f91773d1). No `%` beside an f64 yet.
    pub percents: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config { size: 3, inputs: 0, risk: 5, pow: false, range_edge: false, width_from_context: false, ranges: true, edge: false, sized_bits: true, recursion: true, exhaust: 0, lists: true, item_through_to: false, data_through_to: false, borrows: true, borrow_wrong: 250, enums: true, strict: 200, simd: true, simd_lanes: true, simd_wrap: true, deci: true, deci_useful: true, no_cheat: true, brain16: true, brain_mod: true, roundings: true, mutator: true, ieee: true, remove: true, shadow: true, shadow_wrong: 50, std_lib: true, iterators: true, print: true, written: true, consts: true, new_words: true, edge_landings: true, compound: true, symbol_names: true, width_probes: true, rec_rate: 30, rec_depth: 10, mutual: true, reborrows: true, typed_literals: true, std_all: true, bool_lists: true, tables: true, collections: true, versions: true, ptr_null: true, std_paths: true, std_glyph: true, paren_calls: true, exits: true, generics: true, channels: true, handler_args: true, writers: true, short_lambdas: true, writer_kinds: true, formats: true, c_interop: true, based_literals: true, bare_enums: true, percents: true }
    }
}

#[derive(Clone, Debug)]
struct VarInfo {
    name: String,
    ty: Ty,
    iv: Iv,
    assignable: bool,
    /// Set for a list: `ty` and `iv` are then its items'.
    list: Option<ListInfo>,
    /// The item of a `for … in lend` (see `Config::item_through_to`).
    item: bool,
    /// A `match` arm's data (see `Config::data_through_to`).
    data: bool,
    /// Its range is known to the generator only: it comes from a self-call
    /// (whose result only an induction over the calls could bound), or from
    /// a value that used such a name, with no range type to say it. Never
    /// used in a `check` statement or block (seed 137, 2026-09-29).
    unseen: bool,
    /// Set for an enum value: the index of its enum in `Gen::enums`.
    en: Option<usize>,
    /// Set for a simd vector: its lanes (`ty` and `iv` are then a lane's).
    simd: Option<usize>,
}

#[derive(Clone, Debug)]
struct EnumGen {
    def: EnumDef,
    /// What each case's data can be (the generator's own promise).
    ivs: Vec<Option<Iv>>,
}

#[derive(Clone, Debug)]
struct ListInfo {
    /// Its length is never below this (nothing is ever removed) …
    lo: u64,
    /// … and never above this: appends stop there.
    cap: u64,
    /// The most items the appends generated so far can have made.
    len_now: u64,
    mutable: bool,
    /// `mult` where it was declared: an append runs mult / decl_mult times
    /// per life of the list.
    decl_mult: u64,
    /// Being walked by a `for … in lend`: nothing may change it.
    frozen: u32,
    /// The walk's position name, while inside one that has it.
    walk_pos: Option<String>,
    /// Walked with `lend.mut`: while the walk lasts nothing else may use it,
    /// not even to read (Rust-style borrowing; seed 9512, 2026-09-30).
    lent_mut: bool,
}

#[derive(Clone, Debug)]
struct Sig {
    name: String,
    params: Vec<(String, Ty, Iv)>,
    ret: Ty,
    ret_iv: Iv,
    /// The return has a range type, so a caller sees what it can be.
    ret_ranged: bool,
    cost: u64,
}

/// A borrow function as its callers see it.
#[derive(Clone, Debug)]
struct BSig {
    name: String,
    arity: usize,
    /// The parameters its result may borrow (by the signature's group, which
    /// the generator makes equal to what the body can return).
    deps: Vec<usize>,
    wrong: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum LoopKind {
    None,
    Loop,
}

struct Gen {
    rng: Rng,
    cfg: Config,
    scopes: Vec<Vec<VarInfo>>,
    sigs: Vec<Sig>,
    callable: usize,
    fresh: u32,
    in_loop: LoopKind,
    in_fn: Option<(Ty, Iv)>,
    /// Product of the trip counts of the loops around the current point.
    mult: u64,
    /// Work units left for the current function or MAIN.
    budget: u64,
    spent: u64,
    risky: bool,
    /// Inside a `check` block: no operation is left unguarded here, since
    /// CCA must (rightly) refuse a `check` it cannot prove.
    in_check: u32,
    /// The statement being built reads or writes a list by position: not a
    /// `check` statement, since a guarded position is a fact about the
    /// list's length that Precog need not see.
    positioned: bool,
    bsigs: Vec<BSig>,
    belem: Ty,
    enums: Vec<EnumGen>,
    /// The program holds something my exact evaluator does not model.
    unevaluable: bool,
    /// Functions written as source.
    extra_fns: Vec<String>,
    /// `m1 [lent.mut 'a', lent 'b']` is defined: one call may not lend one
    /// list both ways (owner: a, 38c91e7).
    mutator: bool,
    /// A plain `let` over a visible name was written on purpose.
    shadow_mistake: bool,
    /// `Counter`, `Counter.next` and `half` are in the program.
    iter_ready: bool,
    /// How many call arguments the expression being built is inside.
    call_nest: u32,
    /// A `#strict` file: the whole program under the generator's `check`
    /// rules (no risk, no positions, nothing unseen), since every site must
    /// be proven in its own function, and promised ranges are whole types.
    strict: bool,
}

const MAIN_BUDGET: u64 = 20_000;
const FN_BUDGET: u64 = 400;

pub fn generate(seed: u64, cfg: &Config) -> Program {
    let mut root = Rng::new(seed);
    let mut prng = root.split();
    // `#strict` is gone; what it wrote out is every file's (owner, 2026-10-07)
    let _ = cfg.strict;
    let strict = false;
    let printer = if strict { strict_printer(&mut prng) } else { printer(&mut prng) };
    // the main spellings of 9a9e573 on most programs, MTM's (which a lone file
    // also takes) on the rest; no draw at all when they are off
    // MTM is gone: one spelling each (owner, 2026-10-07)
    let new_words = false;
    let loose = root.chance(50);
    let mut g = Gen {
        rng: root.split(),
        cfg: cfg.clone(),
        scopes: vec![],
        sigs: vec![],
        callable: 0,
        fresh: 0,
        in_loop: LoopKind::None,
        in_fn: None,
        mult: 1,
        budget: 0,
        spent: 0,
        risky: false,
        in_check: 0,
        positioned: false,
        bsigs: vec![],
        belem: Ty::I32,
        enums: vec![],
        strict,
        unevaluable: false,
        extra_fns: vec![],
        mutator: false,
        shadow_mistake: false,
        iter_ready: false,
        call_nest: 0,
    };
    if strict {
        g.in_check = 1;
    }
    if cfg.enums && g.rng.chance(45) {
        for k in 0..1 + g.rng.below(2) as usize {
            g.enum_def(k);
        }
    }
    let nfns = g.rng.below(cfg.size as u64 + 1) as usize;
    let mut funcs = vec![];
    for i in 0..nfns {
        funcs.push(g.function(i));
    }
    // Borrow functions, and whether this program gets a mistake on purpose.
    let mut bfuncs = vec![];
    let mut wrong = None;
    if cfg.borrows && g.rng.chance(35) {
        g.belem = g.int_ty();
        let n = 1 + g.rng.below(4) as usize;
        let mistake = g.rng.below(1000) < cfg.borrow_wrong;
        let bad = if mistake && g.rng.chance(50) { Some(g.rng.below(n as u64) as usize) } else { None };
        for i in 0..n {
            bfuncs.push(g.borrow_fn(i, bad == Some(i)));
        }
        if g.rng.chance(50) && g.cfg.mutator {
            let t = g.belem.name();
            g.extra_fns.push(format!("fn m1 [lent.mut 'a': list[{t}], lent 'b': list[{t}]] {{\n    if list.length[lend 'b'] > 0 {{\n        list.append[lend.mut 'a', value: 1];\n    }}\n}}\n"));
            g.mutator = true;
        }
        if mistake {
            wrong = Some(bad);
        }
    }
    // An iterator and an `optional` (owner, 2026-10-09, 5c52ba8a): a type
    // with its own `next`, walked by `for`, and a function that may give none.
    if cfg.iterators && g.rng.chance(30) {
        g.extra_fns.push("struct Counter {\n    now: i32,\n    stop: i32\n}\n\nfn Counter.next [lent.mut 'c': Counter] -> optional[i32] {\n    if 'c'.now at_least 'c'.stop { return optional.none; }\n    change 'c'.now = 'c'.now plus 1;\n    return optional.value['c'.now];\n}\n\nfn half ['n': i32] -> optional[i32] {\n    if ('n' mod.floor 2) equal_to 0 { return optional.value['n' div.floor 2]; }\n    return optional.none;\n}\n".to_string());
        g.iter_ready = true;
    }
    // MAIN
    g.callable = g.sigs.len();
    g.in_fn = None;
    g.budget = MAIN_BUDGET;
    g.spent = 0;
    g.mult = 1;
    g.scopes = vec![vec![]];
    let mut inputs = vec![];
    let mut main = vec![];
    let mut room = cfg.exhaust;
    for k in 0..cfg.inputs {
        let name = format!("in{}", k + 1);
        if cfg.exhaust > 0 {
            // The raw input is never visible: only its narrowed door value.
            let (door, domain) = g.door(&name, room);
            room /= domain;
            inputs.push(Input { name: name.clone(), var: format!("CHK_IN{}", k + 1), domain: Some(domain) });
            main.push(door);
        } else {
            inputs.push(Input { name: name.clone(), var: format!("CHK_IN{}", k + 1), domain: None });
            g.declare(name, Ty::U64, Ty::U64.range(), false);
        }
    }
    // The mistake goes first in MAIN, where it always runs (the oracle
    // checks borrows only on the paths it runs, sent 2026-09-29).
    if let Some(bad) = wrong {
        main.push(g.episode(Some(bad)));
    }
    // A few typed names first, so comparisons always have something typed.
    for _ in 0..1 + g.rng.below(3) {
        main.push(g.let_stmt());
    }
    if g.cfg.recursion && g.cfg.mutual && g.rng.chance(15) {
        let call = g.mutual_pair();
        main.push(Stmt::Src(call));
    }
    if g.cfg.c_interop && g.rng.chance(10) {
        let lines = g.c_episode();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.ptr_null && g.rng.chance(10) {
        let lines = g.ptr_null();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.generics && g.rng.chance(8) {
        let lines = g.generic_total();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.writers && g.cfg.print && g.rng.chance(8) {
        let lines = g.writer_episode();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.formats && g.cfg.print && g.rng.chance(10) {
        let lines = g.format_episode();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.short_lambdas && g.rng.chance(8) {
        let lines = g.short_lambda();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.channels && g.rng.chance(8) {
        let lines = g.channel_episode();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.percents && g.rng.chance(10) {
        let lines = g.percent_places();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.bare_enums && !g.cfg.std_paths && g.rng.chance(10) {
        let lines = g.bare_enum();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.collections && g.rng.chance(12) {
        let lines = g.collection();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.tables && g.rng.chance(15) {
        let lines = g.table_fn();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.bool_lists && g.cfg.lists && g.rng.chance(12) {
        let call = g.bool_list();
        main.push(Stmt::Src(call));
    }
    if g.cfg.reborrows && g.cfg.borrows && g.cfg.print && g.rng.chance(12) {
        let lines = g.reborrow_chain();
        main.extend(lines.into_iter().map(Stmt::Src));
    }
    if g.cfg.width_probes && g.rng.chance(5) {
        let probe = g.width_probe();
        main.extend(probe);
    }
    let n = g.stmt_count();
    main.extend(g.stmts(n, g.depth()));
    // `std::exit(code: N)` last (owner, 2026-10-23; oracle 69c6aedf): the
    // program's exit code, compared between the engines
    if g.cfg.exits && g.rng.chance(8) {
        g.unevaluable = true;
        main.push(Stmt::Src(format!("std::exit[code: {}];", g.rng.range(0, 99))));
    }
    Program { seed, printer, inputs, funcs, bfuncs, extra_fns: g.extra_fns.clone(), enums: g.enums.iter().map(|e| e.def.clone()).collect(), main, loose, expect_refused: wrong.is_some() || g.shadow_mistake, strict, new_words, typed_lits: cfg.typed_literals, based_lits: cfg.based_literals, std_paths: cfg.std_paths, std_glyph: cfg.std_glyph, paren_calls: cfg.paren_calls, unevaluable: g.unevaluable }
}

impl Gen {
    /// A union of same-size fields (the oracle reads one field's bytes as
    /// another only between numbers of one size), a bit-field struct set at
    /// its edges and sometimes pushed past one in a loop, and C's printf.
    /// Compared between the engines only.
    fn c_episode(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let wide = self.rng.chance(40);
        let (i, u, f) = if wide { ("i64", "u64", "f64") } else { ("i32", "u32", "f32") };
        self.extra_fns.push(format!("layout.C union Un{id} {{ i: {i}, u: {u}, f: {f} }}"));
        let (na, nl, nw) = (self.rng.range(1, 7), self.rng.range(2, 7), self.rng.range(1, 7));
        self.extra_fns.push(format!("layout.C struct Bf{id} {{ a: u8[bits: {na}], r: bool[bits: 1], l: i8[bits: {nl}], w: u8[bits: {nw}] }}"));
        if !self.extra_fns.iter().any(|x| x.contains("fn c_printf")) {
            self.extra_fns.push("extern.C fn c_printf = C.printf ['format': ptr[u8], ...'args'] -> i32;".to_string());
        }
        let lit = |v: i128| if v < 0 { format!("({v})") } else { v.to_string() };
        let (amax, lmin, lmax, wmax) = ((1i128 << na) - 1, -(1i128 << (nl - 1)), (1i128 << (nl - 1)) - 1, (1i128 << nw) - 1);
        let first = self.rng.range(-2_000_000_000, 2_000_000_000);
        let (u_, s_) = (format!("'un{id}'"), format!("'bf{id}'"));
        let mut lines = vec![
            format!("let mut {u_} := Un{id}[i = {}];", lit(first)),
            format!("no_check let 'ux{id}' := {u_}.u;"),
            format!("change {u_}.f = {};", *self.rng.pick(&["1.5", "-2.25", "0.1", "1000000.0", "-0.0"])),
            format!("no_check let 'ui{id}' := {u_}.i;"),
            format!("no_check let 'uu{id}' := {u_}.u;"),
            format!("std::print.useful['ux{id}', \" \", 'ui{id}', \" \", 'uu{id}', newln];"),
            format!("let mut {s_} := Bf{id}[a = {amax}, r = true, l = {}, w = 0];", lit(lmin)),
            format!("change {s_}.l = {lmax};"),
            format!("change {s_}.w = {wmax};"),
            format!("change {s_}.a = {};", self.rng.range(0, amax)),
            format!("std::print.useful[{s_}.a, \" \", {s_}.r, \" \", {s_}.l, \" \", {s_}.w, newln];"),
            format!("let 'fm{id}' := C.str.make[\"c: %d %d; \"];"),
            format!("no_check c_printf[lend 'fm{id}', to.i32[{s_}.w], to.i32[{s_}.l]];"),
            "std::print.useful[\"ours\", newln];".to_string(),
        ];
        if self.rng.chance(30) {
            // past the field's top on the last trip: a check where it lands
            lines.push(format!("for 'bk{id}': u8 [from: {}, to: {}, by: 1] {{ change {s_}.a = 'bk{id}'; }}", (amax - 1).max(0), amax + 1));
            lines.push(format!("std::print.useful[{s_}.a, newln];"));
        }
        lines
    }

    /// `ptr.null`, C's NULL (owner, 2026-10-21; oracle 2c8cfb33), only where
    /// its pointer type is known: a C call's argument, a list beside pointers
    /// from C, a `layout.C` field. Each asked with `ptr.is_null`; the C memory freed.
    fn ptr_null(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        for d in ["extern.C fn pn_free = C.free ['p': ptr.mut[u8]];", "extern.C fn pn_malloc = C.malloc ['n': u64] -> ptr.mut[u8];"] {
            if !self.extra_fns.iter().any(|x| x == d) {
                self.extra_fns.push(d.to_string());
            }
        }
        self.extra_fns.push(format!("layout.C struct Pn{id} {{ p: ptr.mut[u8], v: i32 }}"));
        let v = |n: &str| format!("'{n}{id}'");
        let mut lines = vec![
            "no_check pn_free[ptr.null];".to_string(),
            format!("no_check let {} := pn_malloc[{}];", v("pa"), self.rng.range(1, 64)),
            format!("no_check let {} := pn_malloc[{}];", v("pb"), self.rng.range(1, 64)),
        ];
        // at least one real pointer, so the list's kind is known
        let n = self.rng.range(2, 6) as usize;
        let real = self.rng.below(n as u64) as usize;
        let items: Vec<String> = (0..n).map(|k| if k == real || self.rng.chance(40) { v(if self.rng.chance(50) { "pa" } else { "pb" }) } else { "ptr.null".to_string() }).collect();
        lines.push(format!("let {} := [{}];", v("ps"), items.join(", ")));
        let asks: Vec<String> = (1..=n).map(|k| format!("ptr.is_null[{}[pos: {k}]]", v("ps"))).collect();
        lines.push(format!("std::print.useful[{}, \" \", list.length[lend {}], newln];", asks.join(", \" \", "), v("ps")));
        let (va, vb) = (self.rng.range(-99, 99), self.rng.range(-99, 99));
        let (fa, fb) = if self.rng.chance(50) { ("ptr.null".to_string(), v("pa")) } else { (v("pb"), "ptr.null".to_string()) };
        lines.push(format!("let {} := Pn{id}[p = {fa}, v = {}];", v("sa"), if va < 0 { format!("({va})") } else { va.to_string() }));
        lines.push(format!("let {} := Pn{id}[p = {fb}, v = {}];", v("sb"), if vb < 0 { format!("({vb})") } else { vb.to_string() }));
        lines.push(format!("std::print.useful[ptr.is_null[{a}.p], \" \", ptr.is_null[{b}.p], \" \", {a}.v + {b}.v, newln];", a = v("sa"), b = v("sb")));
        lines.push(format!("no_check pn_free[{}];", v("pa")));
        lines.push(format!("no_check pn_free[{}];", v("pb")));
        lines
    }

    /// A print writer (owner, 2026-10-23): filled in a loop and by a function
    /// lent it, a plain print between, then flushed, so the order of the bytes
    /// says when the writer's text went out; `std::print.flush()`; and a print
    /// handling `write_failed`.
    fn writer_episode(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let v = |n: &str| format!("'{n}{id}'");
        self.extra_fns.push(format!("fn wrep{id} [lent.mut 'out': std::print.writer, 'name': str.utf8, 'n': i64] {{\n    std::print.useful[lend.mut 'out', 'name', \": \", 'n', newln];\n}}"));
        let (k, m, t) = (self.rng.range(1, 5), self.rng.range(-99, 99), self.rng.range(0, 9999));
        let size = *self.rng.pick(&[16u32, 32, 256, 1024, 4096]);
        // a writer `.whole` written out, or `.split` filling to exactly its size
        // (owner, 2026-10-24; oracle deda6720)
        let kind = if self.cfg.writer_kinds { *self.rng.pick(&["", ".whole", ".split"]) } else { "" };
        let lit = |v: i128| if v < 0 { format!("({v})") } else { v.to_string() };
        vec![
            format!("std::print.useful[\"before {id}\", newln];"),
            "std::print.flush[];".to_string(),
            format!("let mut {} := std::print.writer{kind}[size: {size} bytes];", v("pw")),
            format!("for {i}: i64 [from: 1, to: {k}, by: 1] {{ std::print.useful[lend.mut {w}, \"w \", {i}, \" \", {i} x {}, newln]; }}", lit(m), i = v("wi"), w = v("pw")),
            format!("wrep{id}[lend.mut {}, \"total\", {t}];", v("pw")),
            format!("std::print.useful[\"between {id}\", newln];"),
            format!("std::print.flush[lend.mut {}];", v("pw")),
            format!("std::print.useful[\"after {id}\", newln] otherwise {{ write_failed {{ std::exit[code: 3]; }} }};"),
        ]
    }

    /// The format language (design/strings.md; examples/print_format.cy):
    /// values of each kind, each hole given a shape its kind takes (bases on
    /// whole numbers only, `.places` and `e` on bins and decis only), plain
    /// text with `%`, `\t` and escaped braces between; `{}` in order or
    /// numbered holes using every value; printed by `print.format`, made by
    /// `std::format`, and printed into a writer.
    fn format_episode(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let v = |n: &str| format!("'{n}{id}'");
        let lit = |v: i128| if v < 0 { format!("({v})") } else { v.to_string() };
        #[derive(Clone, Copy, PartialEq)]
        enum K { Int, Small, Bin, Deci, Text, Bool }
        let words = ["rope", "tea", "a b", "x", "z9", "ok"];
        let mut lets = vec![];
        let mut vals: Vec<(String, K)> = vec![];
        let n = self.rng.range(1, 5) as usize;
        for j in 0..n {
            let k = *self.rng.pick(&[K::Int, K::Small, K::Bin, K::Deci, K::Text, K::Bool]);
            let name = v(&format!("fv{j}_"));
            let decl = match k {
                K::Int => format!("let {name}: i64 = {};", lit(self.rng.range(-99999, 99999))),
                K::Small => format!("let {name}: u8 = {};", self.rng.range(0, 255)),
                K::Bin => format!("let {name}: f64 = {}{}.{};", if self.rng.chance(30) { "-" } else { "" }, self.rng.range(0, 999), self.rng.range(0, 999)),
                K::Deci => format!("let {name}: d64 = {}.{:02};", self.rng.range(0, 9999), self.rng.range(0, 99)),
                K::Text => format!("let {name}: str.utf8 = \"{}\";", self.rng.pick(&words)),
                K::Bool => format!("let {name}: bool = {};", if self.rng.chance(50) { "true" } else { "false" }),
            };
            lets.push(decl);
            vals.push((name, k));
        }
        let fill = |r: &mut crate::rng::Rng| *r.pick(&["", "*", "-", "."]);
        let shape = |k: K, r: &mut crate::rng::Rng| -> String {
            let w = r.range(1, 9);
            let align = *r.pick(&["<", "^", ">"]);
            match (k, r.below(6)) {
                (_, 0) => String::new(),
                (_, 1) => format!(":{}{align}{w}", fill(r)),
                (K::Int | K::Small, 2) => format!(":{}", r.pick(&["x", "X", "o", "b"])),
                (K::Int | K::Small, 3) => format!(":0{w}{}", r.pick(&["", "x", "X", "b"])),
                (K::Int | K::Small, 4) => ":+".to_string(),
                (K::Bin | K::Deci, 2) => format!(":.{}", r.range(0, 4)),
                (K::Bin | K::Deci, 3) => format!(":.{}e", r.range(0, 3)),
                (K::Bin | K::Deci, 4) => ":?".to_string(),
                (K::Bin | K::Deci, 5) => format!(":{}{align}{}.{}", fill(r), r.range(4, 12), r.range(0, 3)),
                _ => String::new(),
            }
        };
        let plain = ["", " ", " at ", ", ", " 50% off ", "\\t", " \\{x\\} ", " = "];
        let numbered = self.rng.chance(35);
        let mut text = String::new();
        let order: Vec<usize> = if numbered {
            let mut o: Vec<usize> = (0..n).collect();
            for k in (1..o.len()).rev() {
                let j = self.rng.below(k as u64 + 1) as usize;
                o.swap(k, j);
            }
            if self.rng.chance(40) {
                o.push(self.rng.below(n as u64) as usize);
            }
            o
        } else {
            (0..n).collect()
        };
        for (pos, &j) in order.iter().enumerate() {
            if pos > 0 || self.rng.chance(50) {
                text.push_str(self.rng.pick(&plain));
            }
            let sh = shape(vals[j].1, &mut self.rng);
            if numbered {
                text.push_str(&format!("{{{}{sh}}}", j + 1));
            } else {
                text.push_str(&format!("{{{sh}}}"));
            }
        }
        let args = vals.iter().map(|(nm, _)| nm.clone()).collect::<Vec<_>>().join(", ");
        let mut lines = lets;
        lines.push(format!("std::print.format[\"{text}\\n\", {args}];"));
        lines.push(format!("let {} := std::format[\"[{text}]\", {args}];", v("ft")));
        lines.push(format!("std::print.useful[{}, newln];", v("ft")));
        lines.push(format!("let mut {} := std::print.writer[size: 64 bytes];", v("fw")));
        lines.push(format!("std::print.format[lend.mut {}, \"w {text}\\n\", {args}];", v("fw")));
        lines.push(format!("std::print.flush[lend.mut {}];", v("fw")));
        lines
    }

    /// Short lambdas (owner, 2026-10-24): one value after `=`, the types from
    /// where it goes; with a `with` capture, without, and in a typed `fn[i32] -> bool`.
    fn short_lambda(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let v = |n: &str| format!("'{n}{id}'");
        self.extra_fns.push(format!("fn keep{id} [lent 'xs': list[i32], 'test': fn[i32] -> bool] -> list[i32] {{\n    let mut 'out': list[i32] = list.empty;\n    for 'x' in lend 'xs' {{\n        if 'test'['x'] {{ list.append[lend.mut 'out', value: 'x']; }}\n    }}\n    return give 'out';\n}}"));
        let lit = |v: i128| if v < 0 { format!("({v})") } else { v.to_string() };
        let xs: Vec<String> = (0..self.rng.range(1, 7)).map(|_| lit(self.rng.range(-99, 99))).collect();
        let (lim, md, big, x) = (self.rng.range(-50, 50), self.rng.range(2, 5), self.rng.range(-50, 50), self.rng.range(-99, 99));
        vec![
            format!("let {}: list[i32] = [{},];", v("sc"), xs.join(", ")),
            format!("let {}: i32 = {};", v("lim"), lit(lim)),
            format!("let {} := keep{id}[lend {sc}, fn ['s'] with [lend {lm}] = 's' at_least {lm}];", v("kp"), sc = v("sc"), lm = v("lim")),
            format!("let {} := keep{id}[lend {sc}, fn ['s'] = ('s' mod.floor {md}) equal_to 0];", v("ke"), sc = v("sc")),
            format!("std::print.useful[{}, \" \", {}, newln];", v("kp"), v("ke")),
            format!("let {}: fn[i32] -> bool = fn ['s'] = 's' at_least {};", v("bg"), lit(big)),
            format!("std::print.useful[{}[{}], newln];", v("bg"), lit(x)),
        ]
    }

    /// A generic whose `let mut 'sum': <T> = K;` takes `<T>`'s type (owner,
    /// 2026-10-22), filled by an i32, a d64 and an f64 list.
    fn generic_total(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let k = self.rng.range(0, 9);
        self.extra_fns.push(format!("fn gtot{id} <T: any> [lent 'xs': list[<T>]] -> <T> {{\n    let mut 'sum': <T> = {k};\n    for 'x' in lend 'xs' {{ change 'sum' = 'sum' + 'x'; }}\n    return 'sum';\n}}"));
        let lit = |v: i128| if v < 0 { format!("({v})") } else { v.to_string() };
        let ints: Vec<String> = (0..self.rng.range(1, 5)).map(|_| lit(self.rng.range(-999, 999))).collect();
        let decs: Vec<String> = (0..self.rng.range(1, 5)).map(|_| format!("{}.{:02}", self.rng.range(0, 999), self.rng.range(0, 99))).collect();
        let bins: Vec<String> = (0..self.rng.range(1, 5)).map(|_| format!("{}.{}", self.rng.range(0, 999), self.rng.range(0, 99))).collect();
        let v = |n: &str| format!("'{n}{id}'");
        vec![
            format!("let {}: list[i32] = [{},];", v("ga"), ints.join(", ")),
            format!("let {}: list[d64] = [{},];", v("gb"), decs.join(", ")),
            format!("let {}: list[f64] = [{},];", v("gc"), bins.join(", ")),
            format!("std::print.useful[gtot{id}[lend {}], \" \", gtot{id}[lend {}], \" \", gtot{id}[lend {}], newln];", v("ga"), v("gb"), v("gc")),
        ]
    }

    /// Channels from `channel.empty` (owner): two `concur` tasks each sending on
    /// its own channel, taken in order, and a bounded channel filled and
    /// emptied in order, one take past its end.
    fn channel_episode(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let v = |n: &str| format!("'{n}{id}'");
        let mut r = |lo: i128, hi: i128| self.rng.range(lo, hi);
        let (a, b, c, d) = (r(1, 999), r(1, 99), r(0, 9999), r(1, 999));
        let mut lines = vec![
            format!("let {}: channel[i64] = channel.empty;", v("ca")),
            format!("let {}: channel[i64] = channel.empty;", v("cb")),
            format!("concur with [lend {ca}] {{ channel.send[lend {ca}, value: {a} x {b} + {c}]; }}", ca = v("ca")),
            format!("concur with [lend {cb}] {{ channel.send[lend {cb}, value: {d} - {c}]; }}", cb = v("cb")),
            format!("let {} := channel.take[lend {}] otherwise 0;", v("ra"), v("ca")),
            format!("let {} := channel.take[lend {}] otherwise 0;", v("rb"), v("cb")),
            format!("std::print.useful[{}, \" \", {}, newln];", v("ra"), v("rb")),
        ];
        let max = r(1, 6);
        let n = r(1, max);
        lines.push(format!("let {}: channel[i32][max: {max}] = channel.empty;", v("cq")));
        for _ in 0..n {
            let x = r(-500, 500);
            lines.push(format!("channel.send[lend {}, value: {}];", v("cq"), if x < 0 { format!("({x})") } else { x.to_string() }));
        }
        let mut takes = vec![];
        for j in 0..=n {
            lines.push(format!("let {} := channel.take[lend {}] otherwise (-1);", v(&format!("tq{j}_")), v("cq")));
            takes.push(v(&format!("tq{j}_")));
        }
        lines.push(format!("std::print.useful[{}, newln];", takes.join(", \" \", ")));
        lines
    }

    /// A price, two exact percents (`%` and `percent`), their sum and
    /// comparison, a bounded `percent[d32]`, and d64s with `max_places: 2`
    /// holding a rounded product and a sum, and one with `min_places`.
    fn percent_places(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let price = |r: &mut crate::rng::Rng| format!("{}.{:02}", r.range(0, 99999), r.range(0, 99));
        let pct = |r: &mut crate::rng::Rng| if r.chance(50) { format!("{}", r.range(0, 100)) } else { format!("{}.{}", r.range(0, 99), r.range(0, 9)) };
        let (p1, p2) = (price(&mut self.rng), price(&mut self.rng));
        let (t, o, r) = (pct(&mut self.rng), pct(&mut self.rng), pct(&mut self.rng));
        let round = *self.rng.pick(&["near", "half_up", "floor", "ceil", "trunc"]);
        let (mp, n) = (self.rng.range(1, 4), self.rng.range(0, 99999));
        let word = if self.rng.chance(50) { "%" } else { " percent" };
        let v = |s: &str| format!("'{s}{id}'");
        vec![
            format!("let {} := exactly {p1};", v("pr")),
            format!("let {} := exactly {t}%;", v("tx")),
            format!("let {} := exactly {o}{word};", v("of")),
            format!("std::print.useful[{} x to.d64[{}], \" \", {} x to.d64[{}], newln];", v("pr"), v("tx"), v("pr"), v("of")),
            format!("let {} := {} + {};", v("bo"), v("tx"), v("of")),
            format!("let {}: percent[d32][min: 0%, max: 100%] = exactly {r}%;", v("rt")),
            format!("std::print.useful[{}, \" \", {}, \" \", {} > {}, newln];", v("bo"), v("rt"), v("tx"), v("of")),
            format!("std::print.stored[{}, \" \", {}, newln];", v("tx"), v("rt")),
            format!("let {}: d64[min: 0.0, max: 1000000.0, max_places: 2] = exactly {p2};", v("pp")),
            format!("let {}: d64[max_places: 2] = round.{round}[{} x to.d64[{}], places: 2];", v("tt"), v("pp"), v("tx")),
            format!("let {}: d64[max_places: 2] = {} + {};", v("sm"), v("pp"), v("tt")),
            format!("let {}: d64[min_places: {mp}] = exactly {n};", v("sh")),
            format!("std::print.useful[{}, \" \", {}, \" \", {}, newln];", v("tt"), v("sm"), v("sh")),
            // a `%` beside a plain number is its fraction; a percent negated;
            // `%[d32]` for the type (owner, 2026-10-21; kayo 1f60f1cd)
            format!("let {}: f64 = {}.{};", v("gf"), self.rng.range(0, 9999), self.rng.range(0, 99)),
            format!("std::print.useful[{} + {}%, \" \", -{}, \" \", -{}%, newln];", v("gf"), self.rng.range(0, 200), v("tx"), self.rng.range(0, 99)),
            format!("let {}: %[d32] = exactly {}%;", v("sy"), pct(&mut self.rng)),
            format!("std::print.useful[{}, newln];", v("sy")),
            // print.written: a typed literal as typed, else what it says to do
            format!("let {}: f16 = 0.{};", v("wx"), self.rng.range(1, 999999999)),
            format!("std::print.written[{}, \" \", {}, \" \", -2.50, newln];", v("wx"), v("tx")),
            format!("let {} := {} + 0.5: f16;", v("wy"), v("wx")),
            format!("std::print.written[{}, newln] otherwise std::print.useful;", v("wy")),
            format!("let mut {}: f64 = 1.25;", v("wz")),
            format!("change {} = {} x 2.0;", v("wz"), v("wz")),
            format!("std::print.written[{}, newln] otherwise std::print.stored;", v("wz")),
        ]
    }

    /// A pub enum with `bare.enum_prefix.optional` (owner, 2026-10-20), a
    /// function matching on it, and calls writing its values bare, with the
    /// enum's name, and through a typed name. Compared between the engines.
    fn bare_enum(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        const WORDS: &[&str] = &["calm", "loud", "soft", "warm", "cool", "dark", "bright", "slow"];
        let n = self.rng.range(2, 5) as usize;
        let vals: Vec<&str> = WORDS[..n].to_vec();
        let (e, f) = (format!("Md{id}"), format!("say{id}"));
        self.extra_fns.push(format!("pub bare.kit_prefix.only bare.enum_prefix.optional enum {e} {{\n    {}\n}}", vals.join(",\n    ")));
        let arms: Vec<String> = vals.iter().enumerate().map(|(i, v)| format!("{v} {{ return ('k' x {}) + {i}; }}", i + 1)).collect();
        self.extra_fns.push(format!("fn {f} ['m': {e}, 'k': i32] -> i32 {{\n    match 'm' {{ {} }}\n}}", arms.join(" ")));
        let pick = |r: &mut crate::rng::Rng| vals[r.below(n as u64) as usize];
        let (a, b, c, d) = (pick(&mut self.rng), pick(&mut self.rng), pick(&mut self.rng), pick(&mut self.rng));
        let k = |r: &mut crate::rng::Rng| { let v = r.range(-500, 500); if v < 0 { format!("({v})") } else { v.to_string() } };
        let (k1, k2, k3) = (k(&mut self.rng), k(&mut self.rng), k(&mut self.rng));
        let v = format!("'md{id}'");
        vec![
            format!("let {v}: {e} = {a};"),
            format!("std::print.useful[{f}[{b}, {k1}], \" \", {f}[{e}.{c}, {k2}], \" \", {f}[copy {v}, {k3}], \" \", {f}[{d}, 0], newln];"),
        ]
    }

    /// A `coll.fn` with its own function, two presets, a member with its own
    /// body and one pointing at a function written elsewhere, and a fallible
    /// function with a `coll.handler` of two members, used through
    /// `otherwise` in functions whose `return` the arms take. The MAIN lines
    /// calling every member are returned; compared between the engines only.
    fn collection(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let (c, h, hf, help) = (format!("cf{id}"), format!("hh{id}"), format!("hf{id}"), format!("help{id}"));
        let (a, m1, d) = (self.rng.range(2, 13), self.rng.range(2, 97), self.rng.range(2, 9));
        let (k1, mm1, mm2) = (self.rng.range(-9, 9), self.rng.range(1, 97), self.rng.range(1, 97));
        self.extra_fns.push(format!("fn {help} ['x': i32] -> i32 {{\n    return ('x' mod.floor {m1}) + {a};\n}}"));
        self.extra_fns.push(format!(
            "coll.fn {c} {{\n    {c} ['x': i32, 'k': i32, 'm': i32[min: 1, max: 97]] -> i32 {{ return ('x' x 'k') mod.floor 'm'; }}\n    {c}.p1 ['k' = {}, 'm' = {mm1}];\n    {c}.p2 ['m' = {mm2}];\n    {c}.flip ['x': i32] -> i32 {{ return 0 - ('x' mod.floor {m1}); }}\n    {c}.other = {help};\n}}",
            if k1 < 0 { format!("({k1})") } else { k1.to_string() }
        ));
        self.extra_fns.push(format!(
            "fn {hf} ['n': i32] -> i32 or fail[odd, negative: i32] {{\n    if 'n' < 0 {{ fail negative['n']; }}\n    if ('n' mod.floor {d}) not_equal 0 {{ fail odd; }}\n    return 'n' div.floor {d};\n}}"
        ));
        let (z1, z2) = (self.rng.range(-5, 5), self.rng.range(-5, 5));
        let lit = |v: i128| if v < 0 { format!("({v})") } else { v.to_string() };
        self.extra_fns.push(format!(
            "coll.handler {h} {{\n    {h}.as_a {{\n        odd {{ return {}; }}\n        negative 'v' {{ return 'v'; }}\n    }}\n    {h}.as_b {{\n        odd {{ return {}; }}\n        negative 'v' {{ return 0 - 'v'; }}\n    }}{}\n}}",
            lit(z1), lit(z2),
            // a member taking an argument (owner, 2026-10-23)
            if self.cfg.handler_args { format!("\n    {h}.or_default ['default': i32] {{\n        odd {{ return 'default'; }}\n        negative 'v' {{ return 'v' + 'default'; }}\n    }}") } else { String::new() }
        ));
        self.extra_fns.push(format!("fn use_a{id} ['n': i32] -> i32 {{\n    let 'h' := {hf}['n'] otherwise {h}.as_a;\n    return 'h' + 100;\n}}"));
        // versions (owner, 2026-10-20): one name picked by its arguments' count
        // and exact types; in a coll.fn of their own, since a preset beside
        // versions is not decided; every call typed, so none is ambiguous
        let ov = format!("ov{id}");
        let (q1, q2) = (self.rng.range(2, 999), self.rng.range(2, 9));
        // written in `versions` blocks (owner, 2026-10-21; oracle 78a25324)
        if self.cfg.versions {
            self.extra_fns.push(format!(
                "coll.fn {ov} {{\n    versions {ov} {{\n        ['x': i32, 'k': i32] -> i32 {{ return ('x' x 'k') mod.floor {q1}; }}\n        ['x': i64] -> i32 {{ return to.i32['x' mod.floor {q1}]; }}\n        ['x': i16] -> i32 {{ return to.i32['x'] x {q2}; }}\n        ['x': bool] -> i32 {{ if 'x' {{ return {q2}; }} return 0 - {q2}; }}\n    }}\n    versions {ov}.flip {{\n        ['x': i32] -> i32 {{ return 0 - ('x' mod.floor {q1}); }}\n        ['x': i64] -> i32 {{ return to.i32['x' mod.floor {q1}] + 1; }}\n    }}\n}}"
            ));
        }
        self.extra_fns.push(format!("fn use_b{id} ['n': i32] -> i32 {{\n    let 'h' := {hf}['n'] otherwise {h}.as_b;\n    return 'h' + 200;\n}}"));
        let kd = self.rng.range(-99, 99);
        if self.cfg.handler_args {
            self.extra_fns.push(format!("fn use_c{id} ['n': i32] -> i32 {{\n    let 'h' := {hf}['n'] otherwise {h}.or_default[default: {}];\n    return 'h' + 300;\n}}", lit(kd)));
        }
        let mut r = |lo: i128, hi: i128| lit(self.rng.range(lo, hi));
        let (x1, k, mm, x2, x3, x4, x5, x6) = (r(-500, 500), r(-20, 20), r(1, 97), r(-500, 500), r(-500, 500), r(-500, 500), r(-500, 500), r(-50, 50));
        let (n1, n2, n3) = (r(-30, 30), r(-30, 30), r(-30, 30));
        let (o1, o2, o3, o4, o5) = (self.rng.range(0, 99999), self.rng.range(0, 30000), self.rng.range(-500, 500), self.rng.range(0, 99999), self.rng.range(0, 99999));
        let mut lines = if !self.cfg.versions { vec![] } else { vec![
            format!("std::print.useful[{ov}[{o1}: i64], \" \", {ov}[{o2}: i16], \" \", {ov}[{}, {}], \" \", {ov}.flip[{o4}: i32], \" \", {ov}.flip[{o5}: i64], \" \", {ov}[to.i16[{}]], newln];", lit(o3), self.rng.range(1, 9), self.rng.range(-300, 300)),
            // a bool, and arithmetic typed by its operand (kayo c8a27e08)
            format!("std::print.useful[{ov}[{}], \" \", {ov}[to.i64[{}] x {}], \" \", {ov}.flip[(to.i32[{}] + {}) mod.floor 97], newln];", if o1 % 2 == 0 { "true" } else { "false" }, self.rng.range(0, 9999), self.rng.range(1, 9), self.rng.range(0, 999), self.rng.range(0, 99)),
        ] };
        lines.extend(vec![
            format!("std::print.useful[{c}[{x1}, {k}, {mm}], \" \", {c}.p1[{x2}], \" \", {c}.p2[{x3}, {k}], \" \", {c}.flip[{x4}], \" \", {c}.other[{x5}], \" \", {c}.other[{x6}], newln];"),
            format!("std::print.useful[use_a{id}[{n1}], \" \", use_a{id}[{n2}], \" \", use_b{id}[{n3}], \" \", use_b{id}[{n1}], newln];"),
        ]);
        if self.cfg.handler_args {
            lines.push(format!("std::print.useful[use_c{id}[{n1}], \" \", use_c{id}[{n2}], \" \", use_c{id}[{n3}], newln];"));
        }
        lines
    }

    /// A pure function of one small whole-number parameter, sometimes
    /// recursive down a range-typed counter and sometimes marked with a
    /// `table_bytes` limit (owner, 2026-10-19), and the MAIN lines that call
    /// it over many arguments, once directly and sometimes under `no_cheat`.
    /// Compared between the engines only.
    fn table_fn(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let f = format!("tab{}", self.fresh);
        let limit = match self.rng.below(5) { 0 => "table_bytes 0 ".to_string(), 1 => format!("table_bytes {} ", self.rng.range(1, 600)), 2 => "table_bytes 1000000 ".to_string(), _ => String::new() };
        let (a, b, m, c) = (self.rng.range(2, 97), self.rng.range(0, 999), self.rng.range(2, 9999), self.rng.range(0, 500));
        let (param, top) = match self.rng.below(3) {
            0 => ("u8".to_string(), 255),
            1 => { let t = self.rng.range(3, 40); (format!("u8[min: 0, max: {t}]"), t) }
            _ => ("i16[min: 0, max: 600]".to_string(), 600),
        };
        let body = if self.rng.chance(35) && top <= 40 {
            // recursive down the counter
            format!("    if 'x' equal_to 0 {{ return {b}; }}\n    return ({f}['x' - 1] x {a} + to.i64['x']) mod.floor {m};")
        } else if self.rng.chance(50) {
            format!("    if ('x' mod.floor 3) equal_to 0 {{ return (to.i64['x'] x {a}) mod.floor {m}; }}\n    return ((to.i64['x'] x to.i64['x'] + {b}) mod.floor {m}) - {c};")
        } else {
            format!("    return ((to.i64['x'] x {a} + {b}) mod.floor {m}) - {c};")
        };
        self.extra_fns.push(format!("{limit}fn {f} ['x': {param}] -> i64 {{\n{body}\n}}"));
        let pt = if param.starts_with("i16") { "i16" } else { "u8" };
        let step = self.rng.range(1, 7);
        let acc = format!("'tacc{}'", self.fresh);
        let mut lines = vec![
            format!("let mut {acc}: i64 = 0;"),
            format!("for 'ti{}': u64 [from: 0, to: {top}, by: {step}] {{ change {acc} = ({acc} + {f}[to.{pt}['ti{}']]) mod.floor 1000000007; }}", self.fresh, self.fresh),
            format!("put_i[{acc}];"),
            format!("put_i[{f}[{}]];", self.rng.range(0, top)),
        ];
        if self.rng.chance(40) {
            lines.push(format!("no_cheat let 'tq{}' := {f}[{}];", self.fresh, self.rng.range(0, top)));
            lines.push(format!("put_i['tq{}'];", self.fresh));
        }
        lines
    }

    /// A function filling a `list[bool]` (true, false or a value worked out
    /// while running), writing it by position with values known only then
    /// (step 1 to 5, sometimes `for para`, long enough to run in slices),
    /// and counting its trues inline or through a call. The call from MAIN
    /// is returned; compared between the engines only.
    fn bool_list(&mut self) -> String {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let f = format!("bits{id}");
        let fill = match self.rng.below(3) { 0 => "true".to_string(), 1 => "false".to_string(), _ => format!("('seed' mod.floor {}) equal_to 0", self.rng.range(2, 5)) };
        let step = if self.rng.chance(50) { 1 } else { self.rng.range(2, 5) };
        let para = if self.rng.chance(30) { "para " } else { "" };
        let (m, r) = (self.rng.range(2, 9), self.rng.range(2, 5));
        let write = match self.rng.below(3) {
            0 => format!("((to.i64['i'] x {m} + 'seed') mod.floor {r}) equal_to 0"),
            1 => format!("('seed' + to.i64['i']) < {}", self.rng.range(0, 9000)),
            _ => "not 'b'[pos: 'i']".to_string(),
        };
        // `not 'b'[pos 'i']` reads the place it writes: never under `para`
        let para = if write.starts_with("not") { "" } else { para };
        let by_call = self.rng.chance(35);
        let count = if by_call {
            if !self.extra_fns.iter().any(|x| x.starts_with("fn count_true ")) {
                self.extra_fns.push("fn count_true [lent 'b': list[bool]] -> u64 {\n    let mut 'c': u64 = 0;\n    for 'i': u64 [from: 1, to: list.length[lend 'b'], by: 1] {\n        if 'b'[pos: 'i'] { change 'c' = 'c' + 1; }\n    }\n    return 'c';\n}".to_string());
            }
            "    return count_true[lend 'b'];".to_string()
        } else {
            "    let mut 'c': u64 = 0;\n    for 'i': u64 [from: 1, to: list.length[lend 'b'], by: 1] {\n        if 'b'[pos: 'i'] { change 'c' = 'c' + 1; }\n    }\n    return 'c';".to_string()
        };
        self.extra_fns.push(format!(
            "fn {f} ['n': u64[min: 1, max: 30000], 'seed': i64] -> u64 {{\n    let mut 'b': list[bool] = list.fill[value: {fill}, count: 'n'];\n    for {para}'i': u64 [from: 1, to: 'n', by: {step}] {{\n        change 'b'[pos: 'i'] = {write};\n    }}\n{count}\n}}"
        ));
        // lengths of every size, most not a multiple of 8, some long enough for slices
        let n = if self.rng.chance(40) { self.rng.range(3999 * step, 3999 * step + 2000) } else { self.rng.range(1, 70) };
        format!("put_u[{f}[{n}, {}]];", self.rng.range(-50, 50))
    }

    /// `a` changes its `lent.mut` list, passes it on to `b` with `lend.mut`,
    /// and changes it again once `b` returns; `b` the same to `c`; the last
    /// appends the length. The whole list is printed, so the order of every
    /// change shows. Compared between the engines only.
    fn reborrow_chain(&mut self) -> Vec<String> {
        self.unevaluable = true;
        self.fresh += 1;
        let id = self.fresh;
        let n = self.rng.range(2, 4) as usize;
        let names: Vec<String> = (1..=n).map(|i| format!("rb{id}_{i}")).collect();
        for i in 0..n {
            let v = self.rng.range(-99, 99);
            let v = if v < 0 { format!("({v})") } else { v.to_string() };
            let mut body = vec![format!("    list.append[lend.mut 'a', value: {v}];")];
            if i + 1 < n {
                body.push(format!("    {}[lend.mut 'a'];", names[i + 1]));
                body.push(match self.rng.below(3) {
                    0 => format!("    list.append[lend.mut 'a', value: {}];", self.rng.range(100, 199)),
                    1 => "    change 'a'[pos: 1] = to.i32[list.length[lend 'a']];".to_string(),
                    _ => format!("    list.remove[lend.mut 'a', pos: list.length[lend 'a']];\n    list.append[lend.mut 'a', value: {}];", self.rng.range(200, 299)),
                });
            } else {
                body.push("    list.append[lend.mut 'a', value: to.i32[list.length[lend 'a']]];".to_string());
            }
            self.extra_fns.push(format!("fn {} [lent.mut 'a': list[i32]] {{\n{}\n}}", names[i], body.join("\n")));
        }
        let xs = format!("'xs{id}'");
        vec![
            // `[7,]` is a one-item list, `[7]` the number seven (lists.md)
            format!("let mut {xs}: list[i32] = [{},];", self.rng.range(0, 9)),
            format!("{}[lend.mut {xs}];", names[0]),
            format!("std::print.useful[{xs}, newln];"),
        ]
    }

    /// Two functions calling each other down a range-typed counter, each
    /// printing on the way, and sometimes a `mod` by zero at one depth; the
    /// call from MAIN is returned. My evaluator does not run them, so the
    /// program is compared between the engines only.
    fn mutual_pair(&mut self) -> String {
        self.unevaluable = true;
        self.fresh += 1;
        let (a, b) = (format!("ev{}", self.fresh), format!("od{}", self.fresh));
        let top = self.rng.range(2, (self.cfg.rec_depth * 4).max(3));
        let ty = format!("u32[min: 0, max: {top}]");
        let (k1, k2, m1, m2) = (self.rng.range(1, 50), self.rng.range(2, 9), self.rng.range(100, 9999), self.rng.range(100, 9999));
        // a failure at one depth: `mod.floor ('n' - f)` is a mod by zero there
        let div = if self.rng.chance(30) { format!("('n' - {})", self.rng.range(1, top)) } else { m2.to_string() };
        let base = if self.rng.chance(50) { "'acc'".to_string() } else { "'acc' x 2".to_string() };
        self.extra_fns.push(format!(
            "fn {a} ['n': {ty}, 'acc': i64] -> i64 {{\n    if 'n' equal_to 0 {{ return 'acc'; }}\n    put_i['acc'];\n    return {b}['n' - 1, ('acc' + {k1}) mod.floor {m1}];\n}}\n\nfn {b} ['n': {ty}, 'acc': i64] -> i64 {{\n    if 'n' equal_to 0 {{ return {base}; }}\n    return {a}['n' - 1, ('acc' x {k2}) mod.floor {div}];\n}}"
        ));
        let n = self.rng.range(0, top);
        format!("put_i[{a}[{n}, {}]];", self.rng.range(-50, 50))
    }

    /// `let 'p': bool = to.int64[1] equal_to to.int64[2]; if 'p' { … }`
    /// with a bare number shifted by a typed count in one of these places:
    /// whether each gives the shift a width is the engines' to agree on.
    fn width_probe(&mut self) -> Vec<Stmt> {
        // the engines may refuse it before running, which my evaluator, not
        // running source text, cannot see: compared between the engines only
        self.unevaluable = true;
        let p = self.name("p");
        let q = |n: &str| format!("'{n}'");
        let n = self.rng.range(1, 99);
        let k = self.rng.range(0, 5);
        let op = if self.rng.chance(50) { "shr" } else { "shl" };
        // a typed literal gives the shift a width wherever it sits (owner, 2026-10-17)
        let sh = if self.cfg.typed_literals && self.rng.chance(30) { format!("{n}: u8 {op} to.u8[{k}]") } else { format!("{n} {op} to.u8[{k}]") };
        let r = q(&self.name("r"));
        let line = match self.rng.below(13) {
            0 => format!("put_i[{sh}];"),
            1 => format!("let {r}: i32 = {sh};"),
            2 => format!("let {r} := to.i64[{sh}];"),
            3 => format!("let {r}: bool = ({sh}) equal_to to.i32[5];"),
            4 => format!("let {r}: list[i16] = [{sh}, 1];"),
            5 => format!("let {r}: i64 = to.i64[1] shl ({sh});"),
            6 => format!("let {r}: i32 = ({sh}) plus 1;"),
            7 => format!("let {r}: simd[i32, count: 4] = simd.fill[value: {sh}];"),
            8 => format!("let {r}: bool = ({sh}) < 9;"),
            9 => format!("let {r} := ({sh}) mod.floor 7;"),
            10 => format!("let {r}: i32 = 0 minus ({sh});"),
            11 => format!("std::print.useful[{sh}, newln];"),
            _ => format!("let {r}: bool = to.i16[3] at_most ({sh});"),
        };
        let guard = Expr::Cmp(CmpOp::Eq, Box::new(Expr::To(Ty::I64, Box::new(Expr::Lit(1)))), Box::new(Expr::To(Ty::I64, Box::new(Expr::Lit(2)))));
        vec![
            Stmt::Let { tier: Tier::Default, name: p.clone(), mutability: Mutability::Plain, ty: Ty::Bool.into(), value: guard },
            Stmt::If { arms: vec![(Expr::Var(p), vec![Stmt::Src(line)])], otherwise: None },
        ]
    }
}

fn isqrt(n: i128) -> i128 {
    let mut x = (n as f64).sqrt() as i128;
    while x * x > n {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= n {
        x += 1;
    }
    x
}

fn corners(ia: Iv, ib: Iv, f: impl Fn(i128, i128) -> Option<i128>) -> Option<Iv> {
    let vs = [f(ia.lo, ib.lo), f(ia.lo, ib.hi), f(ia.hi, ib.lo), f(ia.hi, ib.hi)];
    let mut lo = i128::MAX;
    let mut hi = i128::MIN;
    for v in vs {
        let v = v?;
        lo = lo.min(v);
        hi = hi.max(v);
    }
    Some(Iv::new(lo, hi))
}

fn floor_div(a: i128, b: i128) -> i128 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

/// A decimal written with digits after the point, one unit in its last
/// place up (`by` 1) or down (-1); never crosses zero here.
fn dec_step(t: &str, by: i32) -> String {
    let mut d: Vec<u8> = t.bytes().collect();
    let mut i = d.len();
    loop {
        i -= 1;
        if d[i] == b'.' {
            continue;
        }
        if by > 0 {
            if d[i] == b'9' { d[i] = b'0'; } else { d[i] += 1; break; }
        } else if d[i] == b'0' {
            d[i] = b'9';
        } else {
            d[i] -= 1;
            break;
        }
        if i == 0 {
            d.insert(0, b'1');
            break;
        }
    }
    String::from_utf8(d).unwrap()
}

fn is_literal_only(e: &Expr) -> bool {
    match e {
        Expr::Lit(_) | Expr::Bool(_) => true,
        Expr::Var(_) | Expr::Call(..) | Expr::To(..) | Expr::At(..) | Expr::Len(_) | Expr::Lend(_) | Expr::Case(..) | Expr::Match(..) | Expr::SimdFill(_) | Expr::SimdFold(..) | Expr::ToEnum(..) | Expr::Num(_) | Expr::Text(_) | Expr::Conv(..) | Expr::StrBytes(_) | Expr::Src(_) | Expr::Remove(..) => false,
        Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) | Expr::AndThen(a, b) | Expr::OrElse(a, b) => is_literal_only(a) && is_literal_only(b),
        Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) => is_literal_only(a),
    }
}

impl Gen {
    fn name(&mut self, prefix: &str) -> String {
        self.fresh += 1;
        if self.cfg.symbol_names && self.rng.chance(12) {
            // written as in the source: `\\` is a `\`, `\'` a `'`, a single
            // space between words (owner, 2026-10-16: "Allow 1, 2, and 4")
            const SYMS: &[&str] = &["!", "\"", "#", "$", "%", "&", "(", ")", "*", "+", ",", "-", "/", ":", ";", "<", "=", ">", "?", "@", "[", "\\\\", "]", "^", "_", "`", "{", "|", "}", "~", ".x", " a", "\\'"];
            let c = SYMS[self.rng.below(SYMS.len() as u64) as usize];
            return format!("{prefix}{}{c}", self.fresh);
        }
        format!("{prefix}{}", self.fresh)
    }

    fn stmt_count(&mut self) -> usize {
        2 + self.rng.below(self.cfg.size as u64 + 2) as usize
    }

    fn depth(&self) -> u32 {
        1 + self.cfg.size / 2
    }

    fn expr_depth(&mut self) -> u32 {
        self.rng.below(2 + self.cfg.size as u64 / 2) as u32 + 1
    }

    fn declare(&mut self, name: String, ty: Ty, iv: Iv, assignable: bool) {
        self.scopes.last_mut().unwrap().push(VarInfo { name, ty, iv, assignable, list: None, item: false, data: false, unseen: false, en: None, simd: None });
    }

    /// Plain values in scope (never a list; inside a `check` block, never
    /// an unseen one).
    fn vars(&self) -> impl Iterator<Item = &VarInfo> {
        let in_check = self.in_check > 0;
        // a shadowed declaration is out of sight: only the innermost of a name
        let all: Vec<&VarInfo> = self.scopes.iter().flatten().collect();
        let seen: Vec<&VarInfo> = all.iter().enumerate().filter(|(i, v)| !all[i + 1..].iter().any(|w| w.name == v.name)).map(|(_, v)| *v).collect();
        seen.into_iter().filter(move |v| v.list.is_none() && v.en.is_none() && v.simd.is_none() && !(in_check && v.unseen))
    }

    /// Whether `v` is read only through `to.T[…]` (an engine gives it no width).
    fn through_to(&self, v: &VarInfo) -> bool {
        (self.cfg.item_through_to && v.item) || (self.cfg.data_through_to && v.data)
    }

    fn simd_vars(&self, t: Option<Ty>) -> Vec<VarInfo> {
        self.scopes.iter().flatten().filter(|v| v.simd.is_some() && t.map_or(true, |t| v.ty == t)).cloned().collect()
    }

    fn enum_vars(&self) -> Vec<VarInfo> {
        self.scopes.iter().flatten().filter(|v| v.en.is_some()).cloned().collect()
    }

    /// Whether `e` reads an unseen name.
    fn uses_unseen(&self, e: &Expr) -> bool {
        // A call's result is unseen unless its return has a range type: the
        // generator's promise for it is its own (seed 3118, 2026-09-29).
        if self.calls_unseen(e) {
            return true;
        }
        let unseen = |n: &str| self.scopes.iter().flatten().rev().find(|v| v.name == n).map_or(false, |v| v.unseen);
        fn walk(e: &Expr, f: &dyn Fn(&str) -> bool) -> bool {
            match e {
                Expr::Var(n) => f(n),
                // A length's bounds and an item's range are the generator's.
                // A lane's range is the generator's too.
                Expr::Len(_) | Expr::At(..) | Expr::Match(..) | Expr::SimdFold(..) | Expr::Remove(..) => true,
                Expr::SimdFill(a) => walk(a, f),
                Expr::ToEnum(_, a, b) => walk(a, f) || walk(b, f),
                // a conversion's result is the generator's guess at most
                Expr::Conv(..) | Expr::StrBytes(_) | Expr::Src(_) => true,
                Expr::Num(_) | Expr::Text(_) => false,
                Expr::Case(_, _, d) => d.as_ref().map_or(false, |d| walk(d, f)),
                Expr::Lit(_) | Expr::Bool(_) | Expr::Lend(_) => false,
                Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) | Expr::AndThen(a, b) | Expr::OrElse(a, b) => walk(a, f) || walk(b, f),
                Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) => walk(a, f),
                Expr::Call(_, args) => args.iter().any(|a| walk(a, f)),
            }
        }
        walk(e, &unseen)
    }

    fn calls_unseen(&self, e: &Expr) -> bool {
        match e {
            // In a strict file every promise is the whole type: nothing unseen.
            Expr::Call(f, args) => (!self.strict && !self.sigs.iter().any(|s| &s.name == f && s.ret_ranged)) || args.iter().any(|a| self.calls_unseen(a)),
            Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) | Expr::AndThen(a, b) | Expr::OrElse(a, b) => self.calls_unseen(a) || self.calls_unseen(b),
            Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) | Expr::At(_, a) => self.calls_unseen(a),
            _ => false,
        }
    }

    fn mark_unseen(&mut self, name: &str) {
        if let Some(v) = self.scopes.iter_mut().flatten().rev().find(|v| v.name == name) {
            v.unseen = true;
        }
    }

    fn lists(&self) -> Vec<VarInfo> {
        self.scopes.iter().flatten().filter(|v| v.list.as_ref().map_or(false, |l| !l.lent_mut)).cloned().collect()
    }

    fn list_mut(&mut self, name: &str) -> &mut ListInfo {
        self.scopes.iter_mut().flatten().rev().find(|v| v.name == name).and_then(|v| v.list.as_mut()).unwrap()
    }

    fn vars_of(&self, t: Ty) -> Vec<VarInfo> {
        self.vars().filter(|v| v.ty == t && !self.through_to(v)).cloned().collect()
    }

    fn int_ty(&mut self) -> Ty {
        if self.cfg.edge && self.rng.chance(70) {
            return if self.rng.chance(50) { Ty::I64 } else { Ty::U64 };
        }
        *self.rng.pick(&INTS)
    }

    /// A sub-interval of `t` to promise for a name, or the whole type.
    fn ghost(&mut self, t: Ty) -> Iv {
        let r = t.range();
        if self.strict {
            // Facts from callers and other bodies are not used in a strict file.
            return r;
        }
        if self.cfg.edge && self.rng.chance(60) {
            return r;
        }
        match self.rng.below(4) {
            0 => r,
            1 => {
                // near an edge
                let w = self.rng.range(1, 64.min(r.width()));
                if self.rng.chance(50) {
                    Iv::new(r.lo, r.lo + w - 1)
                } else {
                    Iv::new(r.hi - w + 1, r.hi)
                }
            }
            _ => {
                let a = self.rng.range(r.lo.max(-1000), r.hi.min(1000));
                let b = self.rng.range(r.lo.max(-1000), r.hi.min(1000));
                Iv::new(a.min(b), a.max(b))
            }
        }
    }

    /// Maybe a range type over `t`, and the interval it allows.
    fn maybe_range(&mut self, t: Ty) -> (TyR, Option<Iv>) {
        if t == Ty::Bool || !self.cfg.ranges || !self.rng.chance(20) {
            return (t.into(), None);
        }
        let r = t.range();
        let near = |g: &mut Gen, edge: i128| -> i128 {
            let v = if g.rng.chance(50) { edge } else { g.rng.range(r.lo.max(-200), r.hi.min(200)) };
            v.clamp(r.lo, r.hi)
        };
        // The whole type written long is allowed (owner, 2026-09-23): rarely.
        if self.rng.chance(5) {
            let lo = if self.rng.chance(50) { None } else { Some(r.lo) };
            let hi = if self.rng.chance(50) { None } else { Some(r.hi) };
            return (TyR { base: t, range: Some((lo, hi)) }, Some(r));
        }
        let (lo, hi) = match self.rng.below(3) {
            0 => {
                let e1 = if self.rng.chance(50) { 0 } else { 1 };
                let a = near(self, e1);
                let e2 = if self.rng.chance(50) { 100 } else { r.hi };
                let b = near(self, e2);
                (Some(a.min(b)), Some(a.max(b)))
            }
            1 => (Some(near(self, 1).max(r.lo + 1)), None),
            _ => (None, Some(near(self, 100).min(r.hi - 1))),
        };
        if lo.is_none() && hi.is_none() {
            return (t.into(), None);
        }
        let iv = Iv::new(lo.unwrap_or(r.lo), hi.unwrap_or(r.hi));
        (TyR { base: t, range: Some((lo, hi)) }, Some(iv))
    }

    /// A door: `in` narrowed into a small typed value whose every value is
    /// reached by some length 0..domain-1, with domain <= room. Returns the
    /// `let` and the domain.
    fn door(&mut self, input: &str, room: u64) -> (Stmt, u64) {
        let name = self.name("d");
        let inp = || Box::new(Expr::Var(input.to_string()));
        let modulo = |m: i128| Expr::Bin(BinOp::Mod, inp(), Box::new(Expr::Lit(m)));
        let kind = if room >= 256 { self.rng.below(4) } else { 2 + self.rng.below(2) };
        let (ty, value, iv, m): (TyR, Expr, Iv, u64) = match kind {
            0 => (
                Ty::I8.into(),
                Expr::To(Ty::I8, Box::new(Expr::Bin(BinOp::Sub, Box::new(Expr::To(Ty::I16, Box::new(modulo(256)))), Box::new(Expr::Lit(128))))),
                Ty::I8.range(),
                256,
            ),
            1 => (Ty::U8.into(), Expr::To(Ty::U8, Box::new(modulo(256))), Ty::U8.range(), 256),
            2 if room >= 2 => {
                // a small range type, anywhere in its base type
                let t = self.int_ty();
                let m = self.rng.range(2, (room.min(40)) as i128);
                let r = t.range();
                let lo = if self.rng.chance(40) { self.rng.range(r.lo, r.hi - (m - 1)) } else { self.rng.range(r.lo.max(-60), (r.hi - (m - 1)).min(60)) };
                let base = Expr::To(t, Box::new(modulo(m)));
                let value = if lo == 0 {
                    base
                } else if lo > 0 {
                    Expr::Bin(BinOp::Add, Box::new(base), Box::new(Expr::Lit(lo)))
                } else {
                    Expr::Bin(BinOp::Sub, Box::new(base), Box::new(Expr::Lit(-lo)))
                };
                (TyR { base: t, range: Some((Some(lo), Some(lo + m - 1))) }, value, Iv::new(lo, lo + m - 1), m as u64)
            }
            _ => (Ty::Bool.into(), Expr::Cmp(CmpOp::Eq, Box::new(modulo(2)), Box::new(Expr::Lit(1))), Iv::new(0, 1), 2),
        };
        self.declare(name.clone(), ty.base, iv, false);
        (Stmt::Let { tier: Tier::Default, name, mutability: Mutability::Plain, ty, value }, m)
    }

    // ------------------------------------------------------------ functions

    fn function(&mut self, index: usize) -> Func {
        let name = format!("f{}", index + 1);
        let nparams = self.rng.below(4) as usize;
        let mut params = vec![];
        let mut param_tys = vec![];
        for k in 0..nparams {
            let t = if self.rng.chance(10) { Ty::Bool } else { self.int_ty() };
            let (tr, rng_iv) = self.maybe_range(t);
            let iv = if t == Ty::Bool { Iv::new(0, 1) } else { rng_iv.unwrap_or_else(|| self.ghost(t)) };
            params.push((format!("p{}", k + 1), t, iv));
            param_tys.push(tr);
        }
        // A recursive function: its first parameter is a range-typed counter
        // that every self-call moves one step toward the base case, so each
        // call ends. Downward to 0, or upward to the top — the upward shape is
        // one CCA's induction must not prove by mistake (compiler claude).
        let rec = if self.cfg.recursion && self.rng.chance(self.cfg.rec_rate) {
            let top = self.rng.range(1, self.cfg.rec_depth.max(1));
            let up = self.rng.chance(30);
            let ct = *self.rng.pick(&[Ty::I32, Ty::I64, Ty::U8, Ty::U32]);
            params.insert(0, ("pr".to_string(), ct, Iv::new(0, top)));
            param_tys.insert(0, TyR { base: ct, range: Some((Some(0), Some(top))) });
            Some((ct, top, up))
        } else {
            None
        };
        let ret = if self.rng.chance(10) { Ty::Bool } else { self.int_ty() };
        let (ret_tr, ret_range) = self.maybe_range(ret);
        let ret_iv = if ret == Ty::Bool { Iv::new(0, 1) } else { ret_range.unwrap_or_else(|| self.ghost(ret)) };
        self.callable = index;
        self.in_fn = Some((ret, ret_iv));
        self.budget = FN_BUDGET;
        self.spent = 0;
        self.mult = 1;
        self.scopes = vec![vec![]];
        for (n, t, iv) in &params {
            self.declare(n.clone(), *t, *iv, false);
            // In a recursive function a parameter's range (other than the
            // counter's range type) holds only by induction over the calls
            // (seed 1520, 2026-09-29).
            if rec.is_some() && n != "pr" {
                self.mark_unseen(n);
            }
        }
        let mut body = vec![];
        if let Some((_, top, up)) = rec {
            let pr = || Box::new(Expr::Var("pr".into()));
            let (base_cond, step) = if up {
                (Expr::Cmp(CmpOp::Ge, pr(), Box::new(Expr::Lit(top))), BinOp::Add)
            } else {
                (Expr::Cmp(CmpOp::Eq, pr(), Box::new(Expr::Lit(0))), BinOp::Sub)
            };
            let (base, _) = self.value_in(ret, ret_iv);
            body.push(Stmt::If { arms: vec![(base_cond, vec![Stmt::Return(base)])], otherwise: None });
            // The self-call, its value kept in a name the rest may use.
            let mut args = vec![Expr::Bin(step, pr(), Box::new(Expr::Lit(1)))];
            for (_, pt, piv) in params.iter().skip(1) {
                let (a, _) = if *pt == Ty::Bool { (self.bool_expr(1), Iv::new(0, 1)) } else { self.value_in(*pt, *piv) };
                args.push(a);
            }
            let rname = self.name("r");
            body.push(Stmt::Let { tier: Tier::Default, name: rname.clone(), mutability: Mutability::Plain, ty: ret.into(), value: Expr::Call(name.clone(), args) });
            self.declare(rname.clone(), ret, ret_iv, false);
            self.mark_unseen(&rname);
        }
        if self.cfg.edge_landings && self.rng.chance(30) {
            let st = self.edge_landing();
            body.push(st);
        }
        let n = self.stmt_count() / 2 + 1;
        body.extend(self.stmts(n, self.depth().saturating_sub(1).max(1)));
        let (e, _) = self.value_in(ret, ret_iv);
        body.push(Stmt::Return(e));
        // A recursive function runs its body once per step of its counter.
        let calls = rec.map_or(1, |(_, top, _)| top as u64 + 1);
        self.sigs.push(Sig { name: name.clone(), params: params.clone(), ret, ret_iv, ret_ranged: ret_tr.range.is_some(), cost: self.spent.max(1).saturating_mul(calls) });
        Func { name, params: params.into_iter().zip(param_tys).map(|((name, _, _), ty)| Param { name, ty }).collect(), ret: ret_tr, body }
    }

    // ------------------------------------------------------------ statements

    fn stmts(&mut self, n: usize, depth: u32) -> Vec<Stmt> {
        let mut out = vec![];
        for _ in 0..n {
            if self.spent >= self.budget {
                break;
            }
            self.spent += self.mult;
            out.push(self.stmt(depth));
        }
        out
    }

    fn block(&mut self, n: usize, depth: u32) -> Vec<Stmt> {
        self.scopes.push(vec![]);
        let b = self.stmts(n, depth);
        self.scopes.pop();
        b
    }

    fn stmt(&mut self, depth: u32) -> Stmt {
        let assignable = self.vars().any(|v| v.assignable);
        let nested = depth > 0;
        let w = [
            6,                                      // let
            if assignable { 6 } else { 0 },         // assign
            if nested { 3 } else { 0 },             // if
            if nested { 3 } else { 0 },             // for
            if nested { 1 } else { 0 },             // while
            if nested { 1 } else { 0 },             // loop
            4,                                      // print
            if nested { 1 } else { 0 },             // block
            if self.cfg.lists { 2 } else { 0 },     // list let
            if self.settable().is_empty() { 0 } else { 2 },   // change 'xs'[p]
            if self.appendable().is_empty() { 0 } else { 2 }, // list.append
            if nested && !self.walkable().is_empty() { 4 } else { 0 }, // for … in lend
            if self.in_fn.is_none() && self.bsigs.iter().any(|b| !b.wrong) { 2 } else { 0 }, // borrows
            if self.enums.is_empty() { 0 } else { 2 },                               // enum let
            if self.enum_vars().iter().any(|v| v.assignable) { 1 } else { 0 },       // enum change
            if nested && !self.enums.is_empty() { 2 } else { 0 },                    // match statement
            if self.cfg.simd { 2 } else { 0 },                                       // simd let
            if self.simd_vars(None).iter().any(|v| v.assignable) { 1 } else { 0 },   // simd change
            if self.cfg.simd_lanes && self.in_check == 0 && self.simd_vars(None).iter().any(|v| v.assignable) { 1 } else { 0 }, // lane change
            if self.cfg.deci { 2 } else { 0 },                                       // deci and conversions
            if self.in_check == 0 && !self.lists().is_empty() { 6 } else { 0 },    // loop over a list's positions
            // no Precog domain yet: every site is a run-time check, which a `check` must not hold
            if self.cfg.brain16 && self.in_check == 0 { 2 } else { 0 },             // brain16
            if self.removable().is_empty() { 0 } else { 2 },                        // list.remove
            if self.shadowable().is_empty() { 0 } else { 1 },                       // shadow
            if self.iter_ready && self.in_check == 0 { 2 } else { 0 },             // iterator, optional
            if self.cfg.print && (!self.lists().is_empty() || !self.simd_vars(None).is_empty() || !self.enum_vars().is_empty()) { 1 } else { 0 }, // print a whole value
            if self.cfg.consts { 2 } else { 0 },                                     // a const in a body
            if self.cfg.edge_landings { 1 } else { 0 },                              // 2^63 and its neighbours, exactly
        ];
        match self.rng.weighted(&w) {
            0 => self.let_stmt(),
            1 => self.assign_stmt(),
            2 => self.if_stmt(depth),
            3 => self.for_stmt(depth),
            4 => self.counted_loop(depth, false),
            5 => self.counted_loop(depth, true),
            8 => self.list_let(),
            9 => self.set_at(),
            10 => self.append(),
            11 => self.for_each(depth),
            12 => self.episode(None),
            13 => self.enum_let(),
            14 => self.enum_assign(),
            15 => self.match_stmt(depth),
            16 => self.simd_let(),
            17 => self.simd_assign(),
            18 => self.lane_write(),
            19 => self.deci_episode(),
            20 => self.list_loop(),
            21 => self.brain_episode(),
            22 => self.remove(),
            23 => self.shadow_stmt(),
            24 => self.iter_episode(),
            25 => self.print_whole(),
            26 => self.const_stmt(),
            27 => self.edge_landing(),
            6 if self.rng.chance(8) => self.min_by_minus_one(),
            6 => {
                let t = self.int_ty();
                let (e, _) = self.int_expr_typed(t);
                Stmt::Print { signed: t.signed(), value: e }
            }
            _ => {
                let tier = match self.rng.below(10) {
                    0..=3 => Tier::Check,
                    4 => Tier::Nocheck,
                    5 => Tier::NoChecks,
                    _ => Tier::Default,
                };
                let n = self.stmt_count() / 2 + 1;
                // No risk inside a vouch either: what it vouches for must hold.
                let guarded = tier != Tier::Default;
                if guarded {
                    self.in_check += 1;
                }
                let body = self.block(n, depth - 1);
                if guarded {
                    self.in_check -= 1;
                }
                Stmt::Block { tier, body }
            }
        }
    }

    // ------------------------------------------------------------ lists

    /// Lists whose items may be read by position without a guard failing:
    /// never empty, or walked right now with a position name.
    fn readable(&self, t: Option<Ty>) -> Vec<VarInfo> {
        if self.in_check > 0 {
            return vec![];
        }
        self.lists().into_iter().filter(|v| t.map_or(true, |t| v.ty == t)).filter(|v| {
            let l = v.list.as_ref().unwrap();
            l.lo >= 1 || l.walk_pos.is_some()
        }).collect()
    }

    fn settable(&self) -> Vec<VarInfo> {
        if self.in_check > 0 {
            return vec![];
        }
        self.lists().into_iter().filter(|v| {
            let l = v.list.as_ref().unwrap();
            l.mutable && l.frozen == 0 && l.lo >= 1
        }).collect()
    }

    fn appendable(&self) -> Vec<VarInfo> {
        self.lists().into_iter().filter(|v| {
            let l = v.list.as_ref().unwrap();
            l.mutable && l.frozen == 0 && l.len_now + (self.mult / l.decl_mult.max(1)).max(1) <= l.cap
        }).collect()
    }

    /// Lists an item may be removed from: changeable, not being walked, and
    /// here once per life of the list, so its least length stays known.
    fn removable(&self) -> Vec<VarInfo> {
        if self.in_check > 0 || !self.cfg.remove {
            return vec![];
        }
        self.lists().into_iter().filter(|v| {
            let l = v.list.as_ref().unwrap();
            l.mutable && l.frozen == 0 && self.mult == l.decl_mult && l.lo >= 1
        }).collect()
    }

    /// `list.remove[lend.mut 'xs', pos p]` (lists.md; `pos`, owner 2026-10-08),
    /// its item dropped or printed; past the end, as a risk taken, a check site.
    fn remove(&mut self) -> Stmt {
        let v = self.rng.pick(&self.removable()).clone();
        let lo = v.list.as_ref().unwrap().lo as i128;
        let k = if self.take_risk() { lo + self.rng.range(1, 2) } else { self.rng.range(1, lo) };
        let l = self.list_mut(&v.name);
        l.lo -= 1;
        l.len_now = l.len_now.saturating_sub(1).max(l.lo);
        let pos = Expr::Lit(k);
        if self.rng.chance(50) {
            Stmt::Remove { name: v.name, pos }
        } else {
            let (t, signed) = if v.ty.signed() { (Ty::I64, true) } else { (Ty::U64, false) };
            Stmt::Print { signed, value: Expr::To(t, Box::new(Expr::Remove(v.name, Box::new(pos)))) }
        }
    }

    fn walkable(&self) -> Vec<VarInfo> {
        let left = self.trips_allowed();
        self.lists().into_iter().filter(|v| v.list.as_ref().unwrap().cap <= left.max(1)).collect()
    }

    fn list_let(&mut self) -> Stmt {
        if self.rng.chance(30) {
            return self.list_fill();
        }
        let elem = self.int_ty();
        let n = self.rng.below(5);
        let mutable = n == 0 || self.rng.chance(60);
        let iv = self.ghost(elem);
        let mut items = vec![];
        self.risky = false;
        self.positioned = false;
        // The items take the list's item type as their context, as a
        // declaration's value does (owner, 62186f1).
        for _ in 0..n {
            items.push(self.value_in(elem, iv).0);
        }
        let room = if mutable { self.rng.below(9) } else { 0 };
        let name = self.name("xs");
        let list = ListInfo { lo: n, cap: n + room, len_now: n, mutable, decl_mult: self.mult, frozen: 0, walk_pos: None, lent_mut: false };
        self.scopes.last_mut().unwrap().push(VarInfo { name: name.clone(), ty: elem, iv, assignable: false, list: Some(list), item: false, data: false, unseen: false, en: None, simd: None });
        Stmt::ListLet { name, mutable, elem, items }
    }

    /// A loop over every position of a list whose trips are independent,
    /// which CCA may run in slices on several threads (34d9c18): each item
    /// rewritten from itself, or all of them added into one sum printed
    /// after. Either way the answer is the in-order one.
    fn list_loop(&mut self) -> Stmt {
        let ls = self.lists();
        let v = self.rng.pick(&ls).clone();
        let l = v.list.clone().unwrap();
        let i = self.name("i");
        let at = || Box::new(Expr::At(v.name.clone(), Box::new(Expr::Var(i.clone()))));
        let bounds = |body: Vec<Stmt>| Stmt::For { perm: false, cheat: 0, name: i.clone(), ty: Ty::U64, start: Expr::Lit(1), stop: Expr::Len(v.name.clone()), step: None, body };
        self.positioned = true;
        if l.mutable && l.frozen == 0 && self.rng.chance(35) {
            // the same through the item: for 'x' in lend.mut 'xs' { change 'x' = … }
            let x = self.name("x");
            let (plus, piv) = (Expr::Bin(BinOp::Add, Box::new(Expr::To(v.ty, Box::new(Expr::Var(x.clone())))), Box::new(Expr::Lit(1))), Iv::new(v.iv.lo + 1, v.iv.hi + 1));
            let (value, _) = self.fit(plus, piv, v.iv, v.ty);
            let set = Stmt::Assign { tier: Tier::Default, name: x.clone(), value };
            let mut body = vec![set];
            // after the item's last use, the list may be read, and shows the
            // change already made (owner: a, 257bde8)
            match self.rng.below(3) {
                0 => body.push(Stmt::Print { signed: false, value: Expr::Len(v.name.clone()) }),
                1 => {
                    let t = if v.ty.signed() { Ty::I64 } else { Ty::U64 };
                    body.push(Stmt::Print { signed: v.ty.signed(), value: Expr::To(t, Box::new(Expr::At(v.name.clone(), Box::new(Expr::Lit(1))))) });
                }
                _ => {}
            }
            return Stmt::ForEach { pos: None, item: x, list: v.name.clone(), body, mutable: true };
        }
        if l.mutable && l.frozen == 0 && self.rng.chance(50) {
            // 'xs'['i'] = ('xs'['i'] + 1), folded back into what the items promise
            let (plus, piv) = (Expr::Bin(BinOp::Add, at(), Box::new(Expr::Lit(1))), Iv::new(v.iv.lo + 1, v.iv.hi + 1));
            let (value, _) = self.fit(plus, piv, v.iv, v.ty);
            let set = Stmt::SetAt { tier: Tier::Default, name: v.name.clone(), pos: Expr::Var(i.clone()), value };
            return bounds(vec![set]);
        }
        let acc = self.name("v");
        let init = Stmt::Let { tier: Tier::Default, name: acc.clone(), mutability: Mutability::Mut, ty: Ty::I64.into(), value: Expr::Lit(0) };
        let item = Expr::To(Ty::I64, Box::new(Expr::Bin(BinOp::Mod, at(), Box::new(Expr::Lit(100)))));
        let add = Stmt::Assign { tier: Tier::Default, name: acc.clone(), value: Expr::Bin(BinOp::Add, Box::new(Expr::Var(acc.clone())), Box::new(item)) };
        Stmt::Block { tier: Tier::Default, body: vec![init, bounds(vec![add]), Stmt::Print { signed: true, value: Expr::Var(acc) }] }
    }

    /// `list.fill[v, n]` (owner, 2026-09-30): a literal count, or one
    /// computed and kept in 0..8 — or, when a risk is taken, one that may
    /// be below 0, so the count's check fires.
    fn list_fill(&mut self) -> Stmt {
        let elem = self.int_ty();
        let mutable = self.rng.chance(60);
        let iv = self.ghost(elem);
        self.risky = false;
        let (value, _) = self.value_in(elem, iv);
        let (count, civ) = if self.rng.chance(40) {
            let n = self.rng.range(0, 8);
            (Expr::Lit(n), Iv::point(n))
        } else {
            let t = *self.rng.pick(&[Ty::I32, Ty::I64, Ty::U8, Ty::U64]);
            let target = if t.signed() && self.take_risk() { Iv::new(-3, 8) } else { Iv::new(0, 8) };
            self.value_in(t, target)
        };
        let (lo, cap) = (civ.lo.max(0) as u64, civ.hi.max(0) as u64);
        let room = if mutable { self.rng.below(5) } else { 0 };
        let name = self.name("xs");
        let list = ListInfo { lo, cap: cap + room, len_now: cap, mutable, decl_mult: self.mult, frozen: 0, walk_pos: None, lent_mut: false };
        self.scopes.last_mut().unwrap().push(VarInfo { name: name.clone(), ty: elem, iv, assignable: false, list: Some(list), item: false, data: false, unseen: false, en: None, simd: None });
        Stmt::ListFill { name, mutable, elem, value, count }
    }

    /// A position in `v`: guarded unless the generator takes a risk.
    fn position(&mut self, v: &VarInfo) -> Expr {
        self.positioned = true;
        let l = v.list.clone().unwrap();
        if self.take_risk() {
            // Unguarded: may be 0 or past the end, so the check can fire.
            let top = l.cap as i128 + 2;
            return if self.rng.chance(50) { Expr::Lit(self.rng.range(0, top)) } else { self.value_in(Ty::U64, Iv::new(0, top)).0 };
        }
        if let Some(p) = &l.walk_pos {
            if l.lo == 0 || self.rng.chance(40) {
                return Expr::Var(p.clone());
            }
        }
        if self.rng.chance(35) {
            return Expr::Lit(self.rng.range(1, l.lo as i128));
        }
        // 1 + (e mod list.length[lend 'xs']): in 1..length whatever e is.
        let (e, _) = self.value_in(Ty::U64, Ty::U64.range());
        let m = Expr::Bin(BinOp::Mod, Box::new(e), Box::new(Expr::Len(v.name.clone())));
        if self.rng.chance(50) {
            Expr::Bin(BinOp::Add, Box::new(Expr::Lit(1)), Box::new(m))
        } else {
            Expr::Bin(BinOp::Add, Box::new(m), Box::new(Expr::Lit(1)))
        }
    }

    fn set_at(&mut self) -> Stmt {
        let v = self.rng.pick(&self.settable()).clone();
        self.risky = false;
        self.positioned = false;
        let pos = self.position(&v);
        let (value, _) = self.value_in(v.ty, v.iv);
        self.positioned |= self.uses_unseen(&value) || self.uses_unseen(&pos);
        let tier = self.tier(self.risky);
        Stmt::SetAt { tier, name: v.name, pos, value }
    }

    fn append(&mut self) -> Stmt {
        let v = self.rng.pick(&self.appendable()).clone();
        let mult = self.mult;
        let l = self.list_mut(&v.name);
        l.len_now += (mult / l.decl_mult.max(1)).max(1);
        // The value takes the item type as its context, as any argument
        // takes its parameter's (owner, 1967d08).
        let (value, _) = self.value_in(v.ty, v.iv);
        Stmt::Append { name: v.name, value }
    }

    fn for_each(&mut self, depth: u32) -> Stmt {
        let walkable = self.walkable();
        let changeable: Vec<VarInfo> = walkable.iter().filter(|v| v.list.as_ref().map_or(false, |l| l.mutable && l.frozen == 0)).cloned().collect();
        let v = if !changeable.is_empty() && self.rng.chance(60) { self.rng.pick(&changeable).clone() } else { self.rng.pick(&walkable).clone() };
        let cap = v.list.as_ref().unwrap().cap;
        // `for 'x' in lend.mut 'xs'`: the item is changed and written back
        // (owner, 2026-09-30); only over a list that may change and is not
        // being walked already.
        let l = v.list.as_ref().unwrap();
        let mutable = l.mutable && l.frozen == 0 && self.rng.chance(60);
        let item = self.name("x");
        let pos = self.rng.chance(50).then(|| self.name("i"));
        let saved = (self.mult, self.in_loop);
        self.mult = self.mult.saturating_mul(cap.max(1));
        self.in_loop = LoopKind::Loop;
        let old_pos = {
            let l = self.list_mut(&v.name);
            l.frozen += 1;
            l.lent_mut = mutable;
            std::mem::replace(&mut l.walk_pos, pos.clone())
        };
        self.scopes.push(vec![]);
        self.declare(item.clone(), v.ty, v.iv, mutable);
        self.scopes.last_mut().unwrap().last_mut().unwrap().item = true;
        self.mark_unseen(&item);
        if let Some(p) = &pos {
            self.declare(p.clone(), Ty::U64, Iv::new(1, cap.max(1) as i128), false);
            self.mark_unseen(p);
        }
        let nb = self.stmt_count() / 2 + 1;
        let mut body = vec![];
        if mutable {
            self.risky = false;
            self.positioned = true;
            let (value, _) = self.value_in(v.ty, v.iv);
            body.push(Stmt::Assign { tier: Tier::Default, name: item.clone(), value });
        }
        body.extend(self.stmts(nb, depth - 1));
        self.scopes.pop();
        {
            let l = self.list_mut(&v.name);
            l.frozen -= 1;
            l.lent_mut = false;
            l.walk_pos = old_pos;
        }
        (self.mult, self.in_loop) = saved;
        Stmt::ForEach { pos, item, list: v.name, body, mutable }
    }

    // ------------------------------------------------------------ borrows

    /// `return lend 'p';` or `return b[lend …];`, and what it can borrow.
    fn borrow_return(&mut self, index: usize, params: &[String]) -> (Expr, Vec<usize>) {
        let callees: Vec<BSig> = self.bsigs[..index].iter().filter(|b| !b.wrong).cloned().collect();
        if !callees.is_empty() && self.rng.chance(50) {
            let c = self.rng.pick(&callees).clone();
            let map: Vec<usize> = (0..c.arity).map(|_| self.rng.below(params.len() as u64) as usize).collect();
            let args = map.iter().map(|&m| Expr::Lend(params[m].clone())).collect();
            let mut deps: Vec<usize> = c.deps.iter().map(|&d| map[d]).collect();
            deps.sort();
            deps.dedup();
            return (Expr::Call(c.name, args), deps);
        }
        let k = self.rng.below(params.len() as u64) as usize;
        (Expr::Lend(params[k].clone()), vec![k])
    }

    fn borrow_fn(&mut self, index: usize, wrong: bool) -> BFunc {
        let name = format!("b{}", index + 1);
        let n = 1 + self.rng.below(3) as usize;
        let params: Vec<String> = ["a", "b", "c"][..n].iter().map(|s| s.to_string()).collect();
        let len = |p: &str| Box::new(Expr::Len(p.to_string()));
        let mut body = vec![];
        let mut deps = vec![];
        if self.rng.chance(35) {
            // Two ways out, chosen by the lists' lengths.
            let a = self.rng.pick(&params).clone();
            let cond = if self.rng.chance(50) {
                let b = self.rng.pick(&params).clone();
                Expr::Cmp(CmpOp::Ge, len(&a), len(&b))
            } else {
                Expr::Cmp(CmpOp::Gt, len(&a), Box::new(Expr::Lit(self.rng.range(0, 3))))
            };
            let (e, d) = self.borrow_return(index, &params);
            body.push(Stmt::If { arms: vec![(cond, vec![Stmt::Return(e)])], otherwise: None });
            deps.extend(d);
        }
        let (e, d) = self.borrow_return(index, &params);
        body.push(Stmt::Return(e));
        deps.extend(d);
        deps.sort();
        deps.dedup();
        // The signature: no groups (Precog reads the body), or @x on exactly
        // what the body can return and @y (or nothing) on the rest.
        // A strict file names the group (owner, 09044b6).
        // with two or more borrowed inputs the group is required (owner, 2026-10-07);
        // with one it is inferred
        let annotated = self.rng.chance(65) || self.strict || n >= 2;
        let mut groups: Vec<Option<String>> = (0..n)
            .map(|i| if !annotated { None } else if deps.contains(&i) { Some("x".into()) } else if self.rng.chance(70) { Some("y".into()) } else { None })
            .collect();
        let mut ret_group = annotated.then(|| "x".to_string());
        let mut unsaid = false;
        if wrong {
            match self.rng.below(3) {
                0 => {
                    // A borrow of a local, which dies at the return.
                    let loc = "loc".to_string();
                    let (v, _) = self.literal(self.belem);
                    body = vec![Stmt::ListLet { name: loc.clone(), mutable: false, elem: self.belem, items: vec![v] }, Stmt::Return(Expr::Lend(loc))];
                }
                1 => {
                    // One returned parameter left out of the result's group.
                    let d = *self.rng.pick(&deps);
                    for (i, g) in groups.iter_mut().enumerate() {
                        *g = if i == d { Some("y".into()) } else if deps.contains(&i) { Some("x".into()) } else { g.clone() };
                    }
                    ret_group = Some("x".into());
                }
                _ => unsaid = true,
            }
        }
        self.bsigs.push(BSig { name: name.clone(), arity: n, deps, wrong });
        BFunc { name, params: params.into_iter().zip(groups).collect(), elem: self.belem, ret_group, unsaid, body }
    }

    /// Print a use of a borrowed list: its first item or its length.
    fn use_borrow(&mut self, r: &str) -> Stmt {
        // A position in a borrowed list is proven only from its length,
        // which a strict file cannot see from here.
        if self.strict || self.rng.chance(40) {
            return Stmt::Print { signed: false, value: Expr::Len(r.to_string()) };
        }
        let at = Expr::At(r.to_string(), Box::new(Expr::Lit(1)));
        if self.belem.signed() {
            Stmt::Print { signed: true, value: Expr::To(Ty::I64, Box::new(at)) }
        } else {
            Stmt::Print { signed: false, value: Expr::To(Ty::U64, Box::new(at)) }
        }
    }

    /// A block of lists, borrows of them handed back through the borrow
    /// functions (some through earlier borrows), uses, and maybe a give, an
    /// append or a change of one list. `mistake`: Some(Some(i)) calls the
    /// wrong function b(i+1); Some(None) changes a list while a borrow of it
    /// is used after; None is right by construction.
    fn episode(&mut self, mistake: Option<Option<usize>>) -> Stmt {
        let elem = self.belem;
        let mut body = vec![];
        let nown = if matches!(mistake, Some(Some(_))) { 3 } else { 1 + self.rng.below(3) as usize };
        let mut owners = vec![];
        for _ in 0..nown {
            let o = self.name("o");
            let mutable = self.rng.chance(60) || mistake == Some(None);
            let k = 1 + self.rng.below(3);
            let items = (0..k).map(|_| self.literal(elem).0).collect();
            body.push(Stmt::ListLet { name: o.clone(), mutable, elem, items });
            owners.push((o, mutable));
        }
        // `m1[lend.mut 'oA', lend 'oB']`: two lists, before any borrow of
        // them exists — or, as the mistake, one list lent both ways. Not in
        // a `check`: the length `m1` leaves is a callee's fact the reads
        // below would lean on (seed 11647).
        if self.mutator && self.in_check == 0 {
            if mistake == Some(None) && self.rng.chance(40) {
                let o = owners[0].0.clone();
                body.push(Stmt::Src(format!("m1[lend.mut '{o}', lend '{o}'];")));
                body.push(Stmt::Print { signed: false, value: Expr::Len(o) });
                self.unevaluable = true;
                return Stmt::Block { tier: Tier::Default, body };
            }
            if mistake.is_none() && nown >= 2 && owners[0].1 && self.rng.chance(30) {
                body.push(Stmt::Src(format!("m1[lend.mut '{}', lend '{}'];", owners[0].0, owners[1].0)));
                self.unevaluable = true;
            }
        }
        // Each borrow: its name and the owners it may borrow.
        let mut borrows: Vec<(String, Vec<usize>)> = vec![];
        let usable: Vec<BSig> = match mistake {
            Some(Some(bad)) => vec![self.bsigs[bad].clone()],
            _ => self.bsigs.iter().filter(|b| !b.wrong).cloned().collect(),
        };
        let nb = if matches!(mistake, Some(Some(_))) { 1 } else { 1 + self.rng.below(3) as usize };
        for _ in 0..nb {
            let f = self.rng.pick(&usable).clone();
            let mut args = vec![];
            let mut arg_deps = vec![];
            for _ in 0..f.arity {
                if matches!(mistake, Some(Some(_))) {
                    // Distinct lists, so the mistake shows even to a check
                    // made when the call runs (seed 3143, 2026-09-29).
                    let i = args.len() % nown;
                    args.push(Expr::Lend(owners[i].0.clone()));
                    arg_deps.push(vec![i]);
                } else if !borrows.is_empty() && self.rng.chance(35) {
                    let (r, d) = self.rng.pick(&borrows).clone();
                    args.push(Expr::Lend(r));
                    arg_deps.push(d);
                } else {
                    let i = self.rng.below(nown as u64) as usize;
                    args.push(Expr::Lend(owners[i].0.clone()));
                    arg_deps.push(vec![i]);
                }
            }
            let mut deps: Vec<usize> = f.deps.iter().flat_map(|&d| arg_deps[d].clone()).collect();
            deps.sort();
            deps.dedup();
            let r = self.name("r");
            body.push(Stmt::Borrow { name: r.clone(), value: Expr::Call(f.name.clone(), args) });
            borrows.push((r, deps));
        }
        for _ in 0..self.rng.below(3) {
            let r = self.rng.pick(&borrows).0.clone();
            body.push(self.use_borrow(&r));
        }
        let kill = mistake == Some(None) || (mistake.is_none() && self.rng.chance(70));
        let mut after: Vec<String> = borrows.iter().map(|b| b.0.clone()).collect();
        if kill {
            // The owner to give away or change; for a mistake, one some borrow depends on.
            let pick: Vec<usize> = if mistake == Some(None) {
                (0..nown).filter(|i| borrows.iter().any(|b| b.1.contains(i))).collect()
            } else {
                (0..nown).collect()
            };
            if let Some(&o) = (!pick.is_empty()).then(|| self.rng.pick(&pick)) {
                let (oname, mutable) = owners[o].clone();
                let kinds = if mutable { 3 } else { 1 };
                body.push(match self.rng.below(kinds) {
                    0 => Stmt::Give { name: self.name("g"), from: oname.clone() },
                    1 => Stmt::Append { name: oname.clone(), value: self.literal(elem).0 },
                    _ => Stmt::SetAt { tier: Tier::Default, name: oname.clone(), pos: Expr::Lit(1), value: self.literal(elem).0 },
                });
                let dependent: Vec<String> = borrows.iter().filter(|b| b.1.contains(&o)).map(|b| b.0.clone()).collect();
                if mistake == Some(None) {
                    after = vec![self.rng.pick(&dependent).clone()];
                } else {
                    // Right by construction: only borrows that do not need it are used after.
                    after.retain(|r| !dependent.contains(r));
                }
            }
        }
        let nafter = if mistake == Some(None) { 1 } else { self.rng.below(3) as usize };
        if !after.is_empty() {
            for _ in 0..nafter {
                let r = self.rng.pick(&after).clone();
                body.push(self.use_borrow(&r));
            }
        }
        Stmt::Block { tier: Tier::Default, body }
    }

    // ------------------------------------------------------------ enums

    fn enum_def(&mut self, k: usize) {
        let n = 2 + self.rng.below(3) as usize;
        let mut cases = vec![];
        let mut ivs = vec![];
        // A `layout.C enum` (owner, 2026-09-27): a distinct number per case, no data.
        let c_enum = self.rng.chance(30);
        let mut values: Vec<i128> = vec![];
        while c_enum && values.len() < n {
            let v = if self.rng.chance(20) { self.rng.range(-2147483648, 2147483647) } else { self.rng.range(-300, 300) };
            if !values.contains(&v) {
                values.push(v);
            }
        }
        for i in 0..n {
            let (t, iv) = if !c_enum && self.rng.chance(55) {
                let t = self.int_ty();
                (Some(t), Some(self.ghost(t)))
            } else {
                (None, None)
            };
            cases.push((format!("c{}", i + 1), t));
            ivs.push(iv);
        }
        self.enums.push(EnumGen { def: EnumDef { name: format!("E{}", k + 1), cases, values: c_enum.then_some(values) }, ivs });
    }

    /// `E1.c2[v]` or `E1.c1`, its data within the case's promise.
    /// `whole`: the value is a declaration's or a change's whole value, the
    /// only place `… otherwise { … }` may stand.
    fn case_value(&mut self, ei: usize, whole: bool) -> Expr {
        let e = self.enums[ei].clone();
        let c = self.rng.below(e.def.cases.len() as u64) as usize;
        if let Some(vals) = &e.def.values {
            if whole && self.rng.chance(50) {
                // A number into the C enum: mostly one a case has, sometimes
                // one next to it, so the fallback runs.
                let w = vals[c] + if self.rng.chance(25) { 1 } else { 0 };
                let t = if Ty::I32.range().has(w) && self.rng.chance(60) { Ty::I32 } else { Ty::I64 };
                let (x, _) = if self.rng.chance(40) { (Expr::Lit(w), Iv::point(w)) } else { self.value_in(t, Iv::point(w)) };
                let fb = self.rng.below(vals.len() as u64) as usize;
                let fallback = Expr::Case(e.def.name.clone(), e.def.cases[fb].0.clone(), None);
                return Expr::ToEnum(e.def.name.clone(), Box::new(x), Box::new(fallback));
            }
        }
        let data = match (e.def.cases[c].1, e.ivs[c]) {
            (Some(t), Some(iv)) => Some(Box::new(self.value_in(t, iv).0)),
            _ => None,
        };
        Expr::Case(e.def.name.clone(), e.def.cases[c].0.clone(), data)
    }

    fn enum_let(&mut self) -> Stmt {
        let ei = self.rng.below(self.enums.len() as u64) as usize;
        let mutable = self.rng.chance(40);
        let value = self.case_value(ei, true);
        let name = self.name("e");
        let en = self.enums[ei].def.name.clone();
        self.scopes.last_mut().unwrap().push(VarInfo { name: name.clone(), ty: Ty::I64, iv: Iv::point(0), assignable: mutable, list: None, item: false, data: false, unseen: false, en: Some(ei), simd: None });
        Stmt::EnumLet { name, mutable, en, value }
    }

    fn enum_assign(&mut self) -> Stmt {
        let vs: Vec<VarInfo> = self.enum_vars().into_iter().filter(|v| v.assignable).collect();
        let v = self.rng.pick(&vs).clone();
        let value = self.case_value(v.en.unwrap(), true);
        Stmt::Assign { tier: Tier::Default, name: v.name, value }
    }

    /// What to match on: an enum name in scope, or a case built right there.
    fn scrutinee(&mut self) -> (Expr, usize) {
        let evs = self.enum_vars();
        if !evs.is_empty() && self.rng.chance(70) {
            let v = self.rng.pick(&evs).clone();
            return (Expr::Var(v.name), v.en.unwrap());
        }
        let ei = self.rng.below(self.enums.len() as u64) as usize;
        (self.case_value(ei, false), ei)
    }

    /// Open an arm's scope, with its data's name if the case has data.
    fn open_arm(&mut self, ei: usize, c: usize) -> Option<String> {
        self.scopes.push(vec![]);
        let e = self.enums[ei].clone();
        let (t, iv) = (e.def.cases[c].1?, e.ivs[c]?);
        let b = self.name("d");
        self.declare(b.clone(), t, iv, false);
        self.scopes.last_mut().unwrap().last_mut().unwrap().data = true;
        // Only the generator knows the data's range.
        self.mark_unseen(&b);
        Some(b)
    }

    /// `match … { … }` giving a value of type `t` within `target`; now and
    /// then an arm leaves instead (`return`, `break`, `continue`).
    fn match_value(&mut self, t: Ty, target: Iv) -> Expr {
        let (scrut, ei) = self.scrutinee();
        let mut arms = vec![];
        for c in 0..self.enums[ei].def.cases.len() {
            let bind = self.open_arm(ei, c);
            let leave = self.rng.chance(12) && (self.in_fn.is_some() || self.in_loop != LoopKind::None);
            let arm = if leave {
                let s = match self.in_fn {
                    Some((rt, riv)) if self.in_loop == LoopKind::None || self.rng.chance(50) => {
                        Stmt::Return(if rt == Ty::Bool { self.bool_expr(1) } else { self.value_in(rt, riv).0 })
                    }
                    _ => if self.rng.chance(50) { Stmt::Break } else { Stmt::Continue },
                };
                Arm { case: self.enums[ei].def.cases[c].0.clone(), bind, value: None, body: vec![s] }
            } else {
                let (v, _) = self.value_in(t, target);
                Arm { case: self.enums[ei].def.cases[c].0.clone(), bind, value: Some(v), body: vec![] }
            };
            self.scopes.pop();
            arms.push(arm);
        }
        Expr::Match(Box::new(scrut), arms)
    }

    fn match_stmt(&mut self, depth: u32) -> Stmt {
        let (scrut, ei) = self.scrutinee();
        let mut arms = vec![];
        for c in 0..self.enums[ei].def.cases.len() {
            let bind = self.open_arm(ei, c);
            let n = if self.rng.chance(25) { 0 } else { self.stmt_count() / 3 + 1 };
            // A match statement may be an arm's only statement: the outer
            // match decides that it is a statement (owner, a4bdac1).
            let body = self.stmts(n, depth - 1);
            self.scopes.pop();
            arms.push(Arm { case: self.enums[ei].def.cases[c].0.clone(), bind, value: None, body });
        }
        Stmt::MatchStmt { scrut, arms }
    }

    // ------------------------------------------------------------ simd

    /// The target's shapes (ARM64's 128 bits, the oracle's target).
    fn lanes(t: Ty) -> usize {
        128 / t.bits() as usize
    }

    /// A vector operand of lane type `t`: a name of that type, or a fill.
    /// Its lanes' interval.
    fn simd_operand(&mut self, t: Ty, _hint: Iv) -> (Expr, Iv) {
        let vs = self.simd_vars(Some(t));
        if !vs.is_empty() && self.rng.chance(65) {
            let v = self.rng.pick(&vs).clone();
            return (Expr::Var(v.name), v.iv);
        }
        let (e, iv) = self.literal(t);
        (Expr::SimdFill(Box::new(e)), iv)
    }

    /// A vector of lane type `t` whose every lane stays in `target`: a fill,
    /// or one lane-wise operator on two operands, switched to a wrapping or
    /// bitwise one where a lane could leave `t`, since each lane operation
    /// is checked at the lane's type (owner: b, fc1cea9) — unless a risk is
    /// taken.
    fn simd_value(&mut self, t: Ty, target: Iv) -> (Expr, Iv) {
        let r = t.range();
        if self.simd_vars(Some(t)).is_empty() || self.rng.chance(35) {
            let (e, iv) = self.value_in(t, target);
            return (Expr::SimdFill(Box::new(e)), iv);
        }
        let (a, ia) = self.simd_operand(t, target);
        use BinOp::*;
        let op = *self.rng.pick(&[Add, Sub, Mul, Div, Mod, AndBit, OrBit, Xor, Shl, Shr, WAdd, WSub, WMul]);
        let op = if !self.cfg.simd_wrap && matches!(op, WAdd | WSub | WMul) { Xor } else { op };
        let (b, ib, op) = match op {
            Div | Mod => {
                // a nonzero divisor, never -1 beside the minimum
                let k = if t.signed() && self.rng.chance(30) { -self.rng.range(2, 9) } else { self.rng.range(1, 9) };
                (Expr::SimdFill(Box::new(Expr::Lit(k))), Iv::point(k), op)
            }
            Shl | Shr => {
                let k = self.rng.range(0, t.bits() as i128 - 1);
                (Expr::SimdFill(Box::new(Expr::Lit(k))), Iv::point(k), op)
            }
            _ => {
                let (b, ib) = self.simd_operand(t, target);
                (b, ib, op)
            }
        };
        let out = match op {
            Add => corners(ia, ib, |x, y| Some(x + y)),
            Sub => corners(ia, ib, |x, y| Some(x - y)),
            Mul => corners(ia, ib, |x, y| x.checked_mul(y)),
            Div => corners(ia, ib, |x, y| Some(floor_div(x, y))),
            Mod => Some(if ib.lo > 0 { Iv::new(0, ib.hi - 1) } else { Iv::new(ib.lo + 1, 0) }),
            Shl => corners(ia, ib, |x, k| x.checked_mul(1i128 << k)),
            Shr if ia.lo >= 0 => Some(Iv::new(ia.lo >> ib.hi, ia.hi >> ib.lo)),
            AndBit | OrBit | Xor if ia.lo >= 0 && ib.lo >= 0 => {
                let top = ia.hi.max(ib.hi);
                let mask = if top == 0 { 0 } else { (1i128 << (128 - top.leading_zeros())) - 1 };
                Some(Iv::new(0, if op == AndBit { ia.hi.min(ib.hi) } else { mask }))
            }
            _ => Some(r),
        };
        let (op, iv) = match out {
            Some(iv) if iv.within(r) => (op, iv),
            // Left to overflow a lane on purpose: each lane operation is
            // checked at the lane type (owner: b, fc1cea9).
            _ if matches!(op, Add | Sub | Mul) && self.take_risk() => (op, r),
            // a lane could leave the type: wrap instead
            _ if self.cfg.simd_wrap => (*self.rng.pick(&[WAdd, WSub, WMul, Xor]), r),
            _ => (*self.rng.pick(&[AndBit, OrBit, Xor]), r),
        };
        (Expr::Bin(op, Box::new(a), Box::new(b)), iv)
    }

    fn simd_let(&mut self) -> Stmt {
        let t = self.int_ty();
        let n = Self::lanes(t);
        let mutable = self.rng.chance(35);
        // A vector that changes promises only its type.
        let target = if mutable { t.range() } else { self.ghost(t) };
        let (value, iv) = self.simd_value(t, target);
        let name = self.name("s");
        let promise = if mutable { t.range() } else { iv };
        self.scopes.last_mut().unwrap().push(VarInfo { name: name.clone(), ty: t, iv: promise, assignable: mutable, list: None, item: false, data: false, unseen: false, en: None, simd: Some(n) });
        Stmt::SimdLet { name, mutable, elem: t, lanes: n, value }
    }

    fn simd_assign(&mut self) -> Stmt {
        let vs: Vec<VarInfo> = self.simd_vars(None).into_iter().filter(|v| v.assignable).collect();
        let v = self.rng.pick(&vs).clone();
        let (value, _) = self.simd_value(v.ty, v.ty.range());
        Stmt::Assign { tier: Tier::Default, name: v.name, value }
    }

    /// `change 's'[k] = v;` (owner, 8d8a302): a literal lane, or rarely one
    /// past the last when a risk is taken.
    fn lane_write(&mut self) -> Stmt {
        let vs: Vec<VarInfo> = self.simd_vars(None).into_iter().filter(|v| v.assignable).collect();
        let v = self.rng.pick(&vs).clone();
        let n = v.simd.unwrap() as i128;
        self.risky = false;
        self.positioned = true;
        let k = if self.take_risk() { n + 1 } else { self.rng.range(1, n) };
        let (value, _) = self.value_in(v.ty, v.iv);
        Stmt::SetAt { tier: Tier::Default, name: v.name, pos: Expr::Lit(k), value }
    }

    /// One number from a vector of lane type `t`: a fold, or one lane.
    fn simd_read(&mut self, v: VarInfo, t: Ty) -> (Expr, Iv) {
        let n = v.simd.unwrap() as i128;
        let var = Box::new(Expr::Var(v.name.clone()));
        let sum = Iv::new(v.iv.lo * n, v.iv.hi * n);
        match self.rng.below(4) {
            0 if self.cfg.simd_lanes => {
                let k = self.rng.range(1, n);
                (Expr::At(v.name, Box::new(Expr::Lit(k))), v.iv)
            }
            // `sum` is exact and checked where it lands, `sum.lane` at the lane
            // type, `sum.wrap` wrapped there with no site (owner: b, 2026-10-07)
            1 if self.rng.chance(30) => (Expr::SimdFold("sum.wrap", var), t.range()),
            1 if sum.within(t.range()) || self.take_risk() => {
                let f = if self.rng.chance(50) { "sum.lane" } else { "sum" };
                (Expr::SimdFold(f, var), sum.meet(t.range()).unwrap_or(t.range()))
            }
            2 => (Expr::SimdFold("min", var), v.iv),
            _ => (Expr::SimdFold("max", var), v.iv),
        }
    }

    // ------------------------------------------------------------ deci

    /// A deci literal of at most `max` significant digits, sometimes with
    /// trailing zeros (kept, as IEEE keeps them).
    fn deci_lit(&mut self, max: usize) -> String {
        let n = 1 + self.rng.below(max.min(6) as u64) as usize;
        let mut digits: String = (0..n).map(|i| {
            let d = self.rng.below(10) as u8;
            (b'0' + if i == 0 && d == 0 { 1 } else { d }) as char
        }).collect();
        if n >= 2 && self.rng.chance(25) {
            digits.replace_range(n - 1.., "0");
        }
        let point = self.rng.below(n as u64 + 1) as usize;
        let mut t = if point == 0 { format!("0.{digits}") } else if point == n { format!("{digits}.0") } else { format!("{}.{}", &digits[..point], &digits[point..]) };
        // `0.xyz` has a leading zero that is not a digit of the value, but
        // stay clear of the edge: keep it within `max` characters of digits.
        if t.chars().filter(|c| c.is_ascii_digit()).count() > max {
            t = digits[..1].to_string() + ".0";
        }
        if self.rng.chance(20) { format!("-{t}") } else { t }
    }

    /// Text to read as a number: well-formed, badly formed, too long, too big.
    fn number_text(&mut self) -> String {
        const POOL: &[&str] = &["42", "-7", "0", "1.250", "-0.5", "2.5e3", "1e400", "abc", "", "12x", " 5", "+3", "3.", ".5", "0.12345678901234567", "99999999999", "-1", "1e-3", "inf", "nan", "00012", "1_000", "7.0"];
        if self.rng.chance(70) {
            POOL[self.rng.below(POOL.len() as u64) as usize].to_string()
        } else {
            let v = self.rng.range(-100000, 100000);
            if self.rng.chance(50) { v.to_string() } else { format!("{}.{}", v, self.rng.below(1000)) }
        }
    }

    /// A block of deci work: values from literals and from each other,
    /// conversions from integers, text, bins and other widths, and what is
    /// seen of them as integers (scaled `to.int64[…] otherwise -1`, a
    /// comparison, the length of its text), plus text read as integers.
    fn deci_episode(&mut self) -> Stmt {
        self.unevaluable = true;
        let w = *self.rng.pick(&[32u32, 64, 64, 128]);
        let ty = format!("d{w}");
        let digits = match w { 32 => 7, 64 => 16, _ => 34 };
        let mut body = vec![];
        let mut decis: Vec<String> = vec![];
        let first = self.name("d");
        body.push(Stmt::TypedLet { name: first.clone(), mutable: false, ty: ty.clone(), value: Expr::Num(self.deci_lit(digits)) });
        decis.push(first);
        for _ in 0..1 + self.rng.below(4) {
            let a = Expr::Var(self.rng.pick(&decis).clone());
            let value = match self.rng.below(9) {
                0..=4 => {
                    let op = self.fraction_op(true);
                    let b = if self.rng.chance(50) { Expr::Var(self.rng.pick(&decis).clone()) } else { Expr::Num(self.deci_lit(digits)) };
                    // a divisor of 0 stops both engines alike; keep it rare
                    let b = match (&b, op.division()) {
                        (Expr::Num(t), Some(_)) if t.trim_start_matches('-').chars().all(|c| c == '0' || c == '.') => Expr::Num("1.5".into()),
                        (Expr::Var(_), Some(_)) if !self.take_risk() => Expr::Num("2.5".into()),
                        _ => b,
                    };
                    Expr::Bin(op, Box::new(a), Box::new(b))
                }
                // a small exponent: values of at most 6 digits stay far inside
                // even deci32's range, since this may sit in `check` code
                5 => Expr::Bin(BinOp::Pow, Box::new(a), Box::new(Expr::Lit(self.rng.range(-3, 4)))),
                6 => Expr::Neg(Box::new(a)),
                7 => {
                    // from an integer in scope: `otherwise` only when its type
                    // could have more digits than the deci holds
                    let ints: Vec<VarInfo> = self.vars().filter(|v| v.ty != Ty::Bool).cloned().collect();
                    if ints.is_empty() {
                        Expr::Num(self.deci_lit(digits))
                    } else {
                        let v = self.rng.pick(&ints).clone();
                        let most = match v.ty { Ty::I8 | Ty::U8 => 3, Ty::I16 | Ty::U16 => 5, Ty::I32 | Ty::U32 => 10, Ty::I64 => 19, _ => 20 };
                        let fb = (most > digits).then(|| Box::new(Expr::Num(self.deci_lit(digits))));
                        Expr::Conv(ty.clone(), Box::new(Expr::Var(v.name)), fb)
                    }
                }
                // `.written` counts every digit written, trailing zeros
                // included (owner, 2026-10-09); plain goes by the value
                _ => {
                    let conv = if self.cfg.written && self.rng.chance(35) { format!("{ty}.written") } else { ty.clone() };
                    Expr::Conv(conv, Box::new(Expr::Text(self.number_text())), Some(Box::new(Expr::Num(self.deci_lit(digits)))))
                }
            };
            let name = self.name("d");
            let typed = !matches!(value, Expr::Conv(..));
            body.push(if typed { Stmt::TypedLet { name: name.clone(), mutable: false, ty: ty.clone(), value } } else { Stmt::AutoLet { name: name.clone(), value } });
            decis.push(name);
        }
        // between widths, and from a bin
        if self.rng.chance(40) {
            let (to, to_digits) = if w == 32 { ("d64", 16) } else { ("d32", 7) };
            let src = Expr::Var(self.rng.pick(&decis).clone());
            let name = self.name("d");
            let fb = Box::new(Expr::Num(self.deci_lit(to_digits)));
            body.push(Stmt::AutoLet { name: name.clone(), value: Expr::Conv(to.into(), Box::new(src), (to == "d32").then_some(fb)) });
            let z = self.name("z");
            body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("i64".into(), Box::new(Expr::Bin(BinOp::Mul, Box::new(Expr::Var(name)), Box::new(Expr::Num("1000".into())))), Some(Box::new(Expr::Lit(-1)))) });
            body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
        }
        if self.rng.chance(35) {
            let b = self.name("b");
            // a percentage is the number over 100, a bin (owner, 2026-10-16; c3918678)
            let bin = if self.rng.chance(40) {
                let v = *self.rng.pick(&["80", "12.5", "0.5", "250", "100", "7", "33.3"]);
                if self.rng.chance(50) { format!("{v}%") } else { format!("{v} percent") }
            } else {
                self.rng.pick(&["0.1", "0.2", "2.5", "-1.75", "3.0", "0.001", "123.456"]).to_string()
            };
            body.push(Stmt::TypedLet { name: b.clone(), mutable: false, ty: "f64".into(), value: Expr::Num(bin) });
            // a bin into a deci says which form (owner, 2026-10-02)
            let conv = if self.cfg.deci_useful && self.rng.chance(50) { format!("useful.{ty}") } else { format!("stored.{ty}") };
            let name = self.name("d");
            body.push(Stmt::AutoLet { name: name.clone(), value: Expr::Conv(conv, Box::new(Expr::Var(b)), None) });
            decis.push(name);
        }
        // an exact percentage, a deci of any digits (owner, 2026-10-16; c3918678)
        if self.rng.chance(30) {
            let name = self.name("d");
            let v = format!("{}.{}", self.rng.range(0, 999), self.rng.range(0, 99));
            let word = if self.rng.chance(50) { "%".to_string() } else { " percent".to_string() };
            body.push(Stmt::TypedLet { name: name.clone(), mutable: false, ty: ty.clone(), value: Expr::Num(format!("exactly {v}{word}")) });
            decis.push(name);
        }
        // a range type, sometimes pushed outside by a risk taken
        if w == 64 && self.rng.chance(30) {
            let name = self.name("d");
            let inside = format!("0.{}", self.rng.range(1, 99));
            let value = if self.take_risk() { Expr::Bin(BinOp::Mul, Box::new(Expr::Var(self.rng.pick(&decis).clone())), Box::new(Expr::Num("3.0".into()))) } else { Expr::Num(inside) };
            body.push(Stmt::TypedLet { name: name.clone(), mutable: false, ty: "d64[min: 0.0, max: 1.0]".into(), value });
            decis.push(name);
        }
        // printed by cord, its own text, stored or useful (Synapse 252fa188)
        if self.cfg.print && self.rng.chance(50) {
            let d = self.rng.pick(&decis).clone();
            let line = self.std_print(&d);
            body.push(Stmt::Src(line));
        }
        // what is seen: scaled into an integer, compared, and as text
        for _ in 0..1 + self.rng.below(3) {
            let d = Expr::Var(self.rng.pick(&decis).clone());
            match self.rng.below(3) {
                0 => {
                    let scale = *self.rng.pick(&["1", "10", "100", "1000", "1000000"]);
                    let z = self.name("z");
                    body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("i64".into(), Box::new(Expr::Bin(BinOp::Mul, Box::new(d), Box::new(Expr::Num(scale.into())))), Some(Box::new(Expr::Lit(-1)))) });
                    body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
                }
                1 => {
                    let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Lt, CmpOp::Ge]);
                    let c = Expr::Cmp(op, Box::new(d), Box::new(Expr::Num(self.deci_lit(digits))));
                    body.push(Stmt::If { arms: vec![(c, vec![Stmt::Print { signed: false, value: Expr::Lit(1) }])], otherwise: Some(vec![Stmt::Print { signed: false, value: Expr::Lit(0) }]) });
                }
                _ => {
                    let s = self.name("t");
                    // a deci into text says which form (owner, 2026-10-02)
                    let form = if self.cfg.deci_useful && self.rng.chance(50) { "useful.str.utf8" } else { "stored.str.utf8" };
                    body.push(Stmt::AutoLet { name: s.clone(), value: Expr::Conv(form.into(), Box::new(d), None) });
                    body.push(Stmt::Print { signed: false, value: Expr::StrBytes(s) });
                }
            }
        }
        // text and characters (design/conversion.md): into ASCII, bytes
        // read as UTF-8, codes and one-character text, seen as lengths and codes
        if self.rng.chance(60) {
            self.text_part(&mut body);
        }
        // text into an integer
        if self.rng.chance(50) {
            let t = *self.rng.pick(&INTS);
            let k = self.name("k");
            let fb = if t.signed() { -2 } else { 7 };
            body.push(Stmt::AutoLet { name: k.clone(), value: Expr::Conv(t.name().into(), Box::new(Expr::Text(self.number_text())), Some(Box::new(Expr::Lit(fb)))) });
            body.push(Stmt::Print { signed: t.signed(), value: Expr::To(if t.signed() { Ty::I64 } else { Ty::U64 }, Box::new(Expr::Var(k))) });
        }
        Stmt::Block { tier: Tier::Default, body }
    }

    /// A signed value divided by -1, printed: the answer is exact and
    /// checked where it lands (owner: a, 15c5c71), so `-128 div.trunc -1`
    /// on an int8 is 128 in `put_i`'s int64. An int64's minimum gives 2^63,
    /// which a uint64 holds (design claude, 2026-10-02): into `put_u`, or into
    /// `put_i`'s int64 as a risk taken, where it does not fit.
    /// An exact bare constant at an edge of the 64-bit types, written as a
    /// sum, product, quotient or power of bare numbers, landing in a
    /// `uint64` or `int64` name or printed: 2^63 (past int64), 2^64 - 1,
    /// 2^63 - 1 and -2^63. Later sites in the function come after it, so a
    /// prover that reads the constant at the wrong width and assumes the
    /// landing passed proves them vacuously (f4d99bd1).
    fn edge_landing(&mut self) -> Stmt {
        use BinOp::*;
        let l = |v: i128| Box::new(Expr::Lit(v));
        let b = |op: BinOp, x: i128, y: i128| Expr::Bin(op, l(x), l(y));
        let p63: i128 = 1 << 63;
        let mut forms: Vec<(i128, Expr)> = vec![
            (p63, b(Div, -p63, -1)),
            (p63, b(Mul, -p63, -1)),
            (p63, b(Add, p63 - 1, 1)),
            (p63, b(Mul, 4294967296, 2147483648)),
            (p63, Expr::Lit(p63)),
            (p63 + 1, b(Sub, p63 + 2, 1)),
            ((1 << 64) - 1, Expr::Lit((1 << 64) - 1)),
            ((1 << 64) - 1, b(Add, p63, p63 - 1)),
            ((1 << 64) - 1, b(Mul, 4294967295, 4294967297)),
            (p63 - 1, b(Sub, p63, 1)),
            (p63 - 1, b(Div, p63 * 2 - 2, 2)),
            (-p63, b(Sub, -p63 + 1, 1)),
            (-p63, b(Mul, p63 / 2, -2)),
            (-p63, Expr::Neg(Box::new(b(Add, p63 - 1, 1)))),
        ];
        if self.cfg.pow {
            forms.push((p63, b(Pow, 2, 63)));
            forms.push(((1 << 64) - 1, Expr::Bin(Sub, Box::new(b(Pow, 2, 64)), l(1))));
            forms.push((-p63, Expr::Neg(Box::new(b(Pow, 2, 63)))));
        }
        let (v, e) = forms[self.rng.below(forms.len() as u64) as usize].clone();
        let t = if v > i64::MAX as i128 { Ty::U64 } else if v < 0 { Ty::I64 } else if self.rng.chance(50) { Ty::U64 } else { Ty::I64 };
        if self.rng.chance(40) {
            return Stmt::Print { signed: t == Ty::I64, value: e };
        }
        let name = self.name("e");
        self.declare(name.clone(), t, Iv::point(v), false);
        Stmt::Let { tier: Tier::Default, name, mutability: Mutability::Plain, ty: t.into(), value: e }
    }

    fn min_by_minus_one(&mut self) -> Stmt {
        use BinOp::*;
        let t = *self.rng.pick(&[Ty::I8, Ty::I16, Ty::I32, Ty::I64]);
        let vs = self.vars_of(t);
        let a = if !vs.is_empty() && self.rng.chance(50) && t != Ty::I64 {
            Expr::Var(self.rng.pick(&vs).name.clone())
        } else {
            Expr::To(t, Box::new(Expr::Lit(t.range().lo)))
        };
        let ops: &[BinOp] = if self.cfg.roundings { &[Div, Mod, DivTrunc, ModTrunc, DivCeil, ModCeil, DivNear, ModNear] } else { &[Div, Mod] };
        let op = *self.rng.pick(ops);
        let signed = t != Ty::I64 || self.take_risk();
        Stmt::Print { signed, value: Expr::Bin(op, Box::new(a), Box::new(Expr::Lit(-1))) }
    }

    /// An operator on decis or brain16: `div.near` is ordinary division;
    /// `.floor` / `.ceil` / `.trunc` give a whole quotient, and their `mod`s
    /// what it leaves (`mod.near` is refused there; owner, 5a12c07).
    fn fraction_op(&mut self, with_mod: bool) -> BinOp {
        use BinOp::*;
        let more = if self.cfg.roundings { 1 } else { 0 };
        let m = if with_mod { 3 } else { 0 };
        let mm = if with_mod { more } else { 0 };
        let ops = [(Add, 4), (Sub, 4), (Mul, 4), (DivNear, 4), (Mod, m), (Div, more), (DivCeil, more), (DivTrunc, more), (ModCeil, mm), (ModTrunc, mm)];
        let w: Vec<u64> = ops.iter().map(|o| o.1).collect();
        ops[self.rng.weighted(&w)].0
    }

    /// A `brain16` episode (design/types.md): values from literals, integers,
    /// text, a deci and a bin, worked on with `+ - x div mod xx` (each step
    /// rounded), and seen as integers: `to.int64[…] otherwise -1`, a deci in
    /// either form scaled, a comparison, the length of its text. A risk
    /// taken: a value past `bin16`'s largest converted into one.
    /// A `Counter` walked by hand (`match` on its `next`), by `for … in
    /// lend.mut` (sometimes left early) and by `for … in give`; and `half`'s
    /// `optional` matched on a value in scope. Compared between the engines.
    fn iter_episode(&mut self) -> Stmt {
        self.unevaluable = true;
        let q = |n: &str| format!("'{n}'");
        let c = self.name("c");
        let now = self.rng.range(-5, 5);
        let stop = now + self.rng.range(-2, 8);
        let mut lines = vec![format!("let MUT{} := Counter[now = {now}, stop = {stop}];", q(&c))];
        if self.rng.chance(60) {
            let x = self.name("x");
            lines.push(format!("match Counter.next[lend.mut {}] {{ value {} {{ put_i[to.i64[{}]]; }} none {{ put_u[0]; }} }}", q(&c), q(&x), q(&x)));
        }
        if self.rng.chance(70) {
            let x = self.name("x");
            let stop_at = self.rng.range(now, stop.max(now) + 1);
            let brk = if self.rng.chance(50) { format!(" if {} equal_to {stop_at} {{ break; }}", q(&x)) } else { String::new() };
            lines.push(format!("for {} in lend.mut {} {{{brk} put_i[to.i64[{}]]; }}", q(&x), q(&c), q(&x)));
        }
        if self.rng.chance(60) {
            let x = self.name("x");
            lines.push(format!("for {} in give {} {{ put_i[to.i64[{}]]; }}", q(&x), q(&c), q(&x)));
        }
        let ints: Vec<VarInfo> = self.vars().filter(|v| matches!(v.ty, Ty::I8 | Ty::I16 | Ty::I32 | Ty::U8 | Ty::U16)).cloned().collect();
        if !ints.is_empty() && self.rng.chance(60) {
            let v = self.rng.pick(&ints).clone();
            let h = self.name("x");
            lines.push(format!("match half[to.i32[{}]] {{ value {} {{ put_i[to.i64[{}]]; }} none {{ put_u[1]; }} }}", q(&v.name), q(&h), q(&h)));
        }
        // `mut` only if a `lend.mut` changes it (owner, 2026-10-10)
        let changes = lines.iter().any(|l| l.contains(&format!("lend.mut {}", q(&c))));
        lines[0] = lines[0].replace("MUT", if changes { "mut " } else { "" });
        Stmt::Block { tier: Tier::Default, body: lines.into_iter().map(Stmt::Src).collect() }
    }

    /// A number at a brain16 half step, or a hair either side of it, where
    /// rounding through bin32 or bin64 first lands on the tie and goes the
    /// wrong way: rounded once, straight to the nearest brain16, it must not
    /// (types.md; the oracle rounded twice, 7457b90a, cd32091f). From an
    /// int64, a literal, a bin64 or text; its stored value is printed.
    fn brain_half_step(&mut self, body: &mut Vec<Stmt>) -> Expr {
        let k = self.rng.range(128, 255);
        let neg = self.rng.chance(25);
        let whole = self.rng.chance(50);
        // the tie (2k + 1) x 2^(p - 1), written exactly
        let (text, int) = if whole {
            let p = self.rng.range(27, 55);
            let tie = (2 * k + 1) << (p - 1);
            // 1 or a power of two below bin32's or bin64's resolution there
            let off = match self.rng.below(4) {
                0 => 0,
                1 => 1,
                2 => 1i128 << self.rng.range(0, (p - 26).min(30)),
                _ => -(1i128 << self.rng.range(0, (p - 26).min(30))),
            };
            let v = if self.rng.chance(50) { tie + off } else { tie - off };
            (v.to_string(), Some(v))
        } else {
            let q = self.rng.range(1, 30) as u32;
            // (2k + 1) / 2^q exactly: its digits are (2k + 1) x 5^q
            let digits = ((2 * k + 1) * 5i128.pow(q)).to_string();
            let digits = format!("{:0>w$}", digits, w = q as usize + 1);
            let (ip, fp) = digits.split_at(digits.len() - q as usize);
            let mut t = format!("{ip}.{fp}");
            // a hair below or above, past bin32's or bin64's digits
            match self.rng.below(3) {
                0 => {}
                n => {
                    t.push_str(&"0".repeat(self.rng.range(4, 24) as usize));
                    t = if n == 1 { dec_step(&t, -1) } else { dec_step(&t, 1) };
                }
            }
            (t, None)
        };
        let text = if neg { format!("-{text}") } else { text };
        let lit = if text.contains('.') { text.clone() } else { format!("{text}.0") };
        let w = self.name("w");
        let value = match (self.rng.below(4), int) {
            (0, Some(v)) if v <= i64::MAX as i128 => {
                let i = self.name("i");
                body.push(Stmt::Src(format!("let '{i}': i64 = {}{v};", if neg { "-" } else { "" })));
                Expr::Conv("brain16".into(), Box::new(Expr::Var(i)), None)
            }
            (1, _) => {
                let g = self.name("g");
                body.push(Stmt::TypedLet { name: g.clone(), mutable: false, ty: "f64".into(), value: Expr::Num(lit.clone()) });
                Expr::Conv("brain16".into(), Box::new(Expr::Var(g)), None)
            }
            (2, _) => Expr::Conv("brain16".into(), Box::new(Expr::Text(text.clone())), Some(Box::new(Expr::Num("1.0".into())))),
            _ => Expr::Num(lit),
        };
        let typed = !matches!(value, Expr::Conv(..));
        body.push(if typed { Stmt::TypedLet { name: w.clone(), mutable: false, ty: "brain16".into(), value } } else { Stmt::AutoLet { name: w.clone(), value } });
        body.push(Stmt::Src(format!("std::print.stored['{w}', newln];")));
        Expr::Var(w)
    }

    fn brain_episode(&mut self) -> Stmt {
        self.unevaluable = true;
        const LITS: &[&str] = &["0.1", "0.2", "0.5", "1.5", "2.5", "-1.75", "3.0", "12.75", "300.7", "0.001", "1000.0", "-0.3"];
        let names = ["brain16"]; // its other names were MTM's, gone (owner, 2026-10-07)
        let mut body = vec![];
        let mut bs: Vec<String> = vec![];
        let first = self.name("w");
        let lit = self.rng.pick(LITS).to_string();
        body.push(Stmt::TypedLet { name: first.clone(), mutable: false, ty: self.rng.pick(&names).to_string(), value: Expr::Num(lit) });
        bs.push(first);
        for _ in 0..1 + self.rng.below(4) {
            let a = Expr::Var(self.rng.pick(&bs).clone());
            // the .ieee family keeps IEEE's -0 where a plain operator gives 0
            // (owner, 2026-10-07; 72d6946d, 3806bfdf); written as text, since
            // a brain16 block is compared between the engines only
            let value = if self.cfg.ieee && self.rng.chance(20) {
                let x = self.rng.pick(&bs).clone();
                let y = if self.rng.chance(50) { format!("'{}'", self.rng.pick(&bs)) } else { format!("({})", self.rng.pick(LITS)) };
                let ops = ["plus", "minus", "times", "div.floor", "div.ceil", "div.trunc", "div.near", "mod.floor", "mod.ceil", "mod.trunc"];
                let src = match self.rng.below(12) {
                    0 => format!("negate.ieee '{x}'"),
                    1 => format!("'{x}' pow.ieee {}", self.rng.range(0, 2)),
                    _ => format!("'{x}' {}.ieee {y}", self.rng.pick(&ops)),
                };
                Expr::Src(format!("({src})"))
            } else if self.rng.chance(15) {
                self.brain_half_step(&mut body)
            } else { match self.rng.below(8) {
                0..=3 => {
                    let op = self.fraction_op(self.cfg.brain_mod);
                    let b = if self.rng.chance(50) { Expr::Var(self.rng.pick(&bs).clone()) } else { Expr::Num(self.rng.pick(LITS).to_string()) };
                    // a divisor of 0 stops both engines alike; keep it rare
                    let b = match (&b, op.division()) {
                        (Expr::Var(_), Some(_)) if !self.take_risk() => Expr::Num("2.5".into()),
                        _ => b,
                    };
                    Expr::Bin(op, Box::new(a), Box::new(b))
                }
                4 => Expr::Bin(BinOp::Pow, Box::new(a), Box::new(Expr::Lit(self.rng.range(0, 3)))),
                5 => {
                    let ints: Vec<VarInfo> = self.vars().filter(|v| v.ty != Ty::Bool).cloned().collect();
                    match ints.is_empty() {
                        true => Expr::Conv("brain16".into(), Box::new(Expr::Text(self.number_text())), Some(Box::new(Expr::Num("1.0".into())))),
                        false => Expr::Conv("brain16".into(), Box::new(Expr::Var(self.rng.pick(&ints).name.clone())), None),
                    }
                }
                6 => Expr::Conv("brain16".into(), Box::new(Expr::Text(self.number_text())), Some(Box::new(Expr::Num(self.rng.pick(LITS).to_string())))),
                _ => {
                    // from a deci or a bin, rounding
                    let (d, ty) = if self.rng.chance(50) { (self.name("d"), "d64") } else { (self.name("g"), "f64") };
                    body.push(Stmt::TypedLet { name: d.clone(), mutable: false, ty: ty.into(), value: Expr::Num(self.rng.pick(LITS).to_string()) });
                    Expr::Conv("brain16".into(), Box::new(Expr::Var(d)), None)
                }
            } };
            let name = self.name("w");
            let typed = !matches!(value, Expr::Conv(..));
            body.push(if typed { Stmt::TypedLet { name: name.clone(), mutable: false, ty: self.rng.pick(&names).to_string(), value } } else { Stmt::AutoLet { name: name.clone(), value } });
            bs.push(name);
        }
        if self.rng.chance(25) {
            let w = Expr::Var(self.rng.pick(&bs).clone());
            // bin16's largest is 65504: past it, a check site
            let w = if self.take_risk() { Expr::Bin(BinOp::Mul, Box::new(w), Box::new(Expr::Num("70000.0".into()))) } else { w };
            let (v, h) = (self.name("w"), self.name("h"));
            body.push(Stmt::TypedLet { name: v.clone(), mutable: false, ty: "brain16".into(), value: w });
            body.push(Stmt::AutoLet { name: h.clone(), value: Expr::Conv("f16".into(), Box::new(Expr::Var(v)), None) });
            let d = self.name("d");
            body.push(Stmt::AutoLet { name: d.clone(), value: Expr::Conv("useful.d64".into(), Box::new(Expr::Var(h)), None) });
            let z = self.name("z");
            body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("i64".into(), Box::new(Expr::Var(d)), Some(Box::new(Expr::Lit(-1)))) });
            body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
        }
        if self.cfg.print && self.rng.chance(50) {
            let w = self.rng.pick(&bs).clone();
            let line = self.std_print(&w);
            body.push(Stmt::Src(line));
        }
        for _ in 0..1 + self.rng.below(3) {
            let w = Expr::Var(self.rng.pick(&bs).clone());
            match self.rng.below(4) {
                0 => {
                    let z = self.name("z");
                    body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("i64".into(), Box::new(w), Some(Box::new(Expr::Lit(-1)))) });
                    body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
                }
                1 => {
                    let form = if self.rng.chance(50) { "useful.d64" } else { "stored.d64" };
                    let scale = *self.rng.pick(&["1", "100", "1000000"]);
                    let (d, z) = (self.name("d"), self.name("z"));
                    body.push(Stmt::AutoLet { name: d.clone(), value: Expr::Conv(form.into(), Box::new(w), None) });
                    body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("i64".into(), Box::new(Expr::Bin(BinOp::Mul, Box::new(Expr::Var(d)), Box::new(Expr::Num(scale.into())))), Some(Box::new(Expr::Lit(-1)))) });
                    body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
                }
                2 => {
                    let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Lt, CmpOp::Ge]);
                    let c = Expr::Cmp(op, Box::new(w), Box::new(Expr::Num(self.rng.pick(LITS).to_string())));
                    body.push(Stmt::If { arms: vec![(c, vec![Stmt::Print { signed: false, value: Expr::Lit(1) }])], otherwise: Some(vec![Stmt::Print { signed: false, value: Expr::Lit(0) }]) });
                }
                _ => {
                    let s = self.name("t");
                    let form = if self.rng.chance(50) { "useful.str.utf8" } else { "stored.str.utf8" };
                    body.push(Stmt::AutoLet { name: s.clone(), value: Expr::Conv(form.into(), Box::new(w), None) });
                    body.push(Stmt::Print { signed: false, value: Expr::StrBytes(s) });
                }
            }
        }
        Stmt::Block { tier: Tier::Default, body }
    }

    /// A list, a vector or an enum value printed whole by cord, as both
    /// engines print them (Synapse beeb7b1a).
    fn print_whole(&mut self) -> Stmt {
        let mut names: Vec<String> = self.lists().into_iter().map(|v| v.name).collect();
        names.extend(self.simd_vars(None).into_iter().map(|v| v.name));
        names.extend(self.enum_vars().into_iter().map(|v| v.name));
        let n = self.rng.pick(&names).clone();
        Stmt::Src(self.std_print(&n))
    }

    /// `std::print.useful['x', newln];` or `.stored`, or `.auto_newln` with
    /// no newln: the value's own text, compared between the engines.
    fn std_print(&mut self, name: &str) -> String {
        let form = if self.rng.chance(50) { "useful" } else { "stored" };
        // Synapse builds `auto_newln` only on `useful` (252fa188)
        if form == "useful" && self.rng.chance(40) {
            format!("std::print.{form}.auto_newln['{name}'];")
        } else {
            format!("std::print.{form}['{name}', newln];")
        }
    }

    fn text_part(&mut self, body: &mut Vec<Stmt>) {
        // two regional indicators make one flag, one grapheme (UAX #29; 5b4b5200)
        const TEXTS: &[&str] = &["hello", "h\u{e9}llo", "", "Z", "ZZ", "7", "a b", "\u{65e5}\u{672c}", "caf\u{e9}!", "\u{e9}", "\u{1f1f9}\u{1f1ed}\u{1f1ef}\u{1f1f5}!", "a\u{1f1fa}\u{1f1f8}b"];
        let q = |n: &str| format!("'{n}'");
        // `str.at.<unit>` and `str.slice.grapheme` by position, out of range
        // sometimes (owner, 2026-10-08); cord's (once stdkit), so not in CCA's runs yet
        if self.cfg.std_lib && self.rng.chance(30) {
            let t = TEXTS[self.rng.below(TEXTS.len() as u64) as usize];
            let n = t.chars().count() as i128;
            let s = self.name("t");
            body.push(Stmt::TypedLet { name: s.clone(), mutable: false, ty: "str.utf8".into(), value: Expr::Text(t.into()) });
            let r = self.name("t");
            // a position is uint64[min 1]: 0 is refused, past the end is
            // what `otherwise` catches (owner, 2026-10-17: a; 09061256)
            let k = self.rng.range(1, n + 2);
            // std/ has str.at by byte, point and grapheme; str.slice only with
            // --std-all until kayo reads std's from / to (2026-10-20)
            let line = match self.rng.below(if self.cfg.std_all { 4 } else { 3 }) {
                0 => format!("let {} := std::str.at.byte[lend {}, pos: {k}] otherwise 7;\nput_u[to.u64[{}]];", q(&r), q(&s), q(&r)),
                1 => format!("let {} := std::str.at.point[lend {}, pos: {k}] otherwise `?`;\nput_u[to.u64[char.code[{}]]];", q(&r), q(&s), q(&r)),
                // a grapheme here is a plain char: it is printed, not lent
                2 => format!("let {} := std::str.at.grapheme[lend {}, pos: {k}] otherwise `?`;\nstd::print.useful[{}, newln];", q(&r), q(&s), q(&r)),
                _ => {
                    let a = self.rng.range(1, n + 1);
                    let b = self.rng.range(a - 1, n + 2).max(1);
                    format!("let {} := std::str.slice.grapheme[lend {}, from: pos {a}, to: pos {b}] otherwise str.empty;\nput_u[str.bytes[lend {}]];", q(&r), q(&s), q(&r))
                }
            };
            for l in line.split('\n') {
                body.push(Stmt::Src(l.to_string()));
            }
            return;
        }
        match self.rng.below(5) {
            0 => {
                let t = TEXTS[self.rng.below(TEXTS.len() as u64) as usize];
                let (s, a) = (self.name("t"), self.name("t"));
                body.push(Stmt::TypedLet { name: s.clone(), mutable: false, ty: "str.utf8".into(), value: Expr::Text(t.into()) });
                body.push(Stmt::AutoLet { name: a.clone(), value: Expr::Conv("str.ascii".into(), Box::new(Expr::Var(s)), Some(Box::new(Expr::Text("?".into())))) });
                body.push(Stmt::Print { signed: false, value: Expr::StrBytes(a) });
            }
            1 => {
                // bytes: UTF-8 or not (a stray continuation byte, 255)
                let n = 1 + self.rng.below(6);
                let mut bytes: Vec<u8> = (0..n).map(|_| self.rng.range(97, 122) as u8).collect();
                match self.rng.below(4) {
                    0 => bytes.extend([195, 169]),
                    1 => bytes.push(255),
                    2 => bytes.push(self.rng.range(128, 191) as u8),
                    _ => {}
                }
                let list = if bytes.len() == 1 { format!("[{},]", bytes[0]) } else { format!("[{}]", bytes.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", ")) };
                let (l, t) = (self.name("l"), self.name("t"));
                body.push(Stmt::TypedLet { name: l.clone(), mutable: false, ty: "list[u8]".into(), value: Expr::Src(list) });
                body.push(Stmt::AutoLet { name: t.clone(), value: Expr::Conv("str.utf8".into(), Box::new(Expr::Var(l)), Some(Box::new(Expr::Text("?".into())))) });
                let count = if self.rng.chance(50) { "bytes" } else { "points" };
                body.push(Stmt::Print { signed: false, value: Expr::Src(format!("str.{count}[lend {}]", q(&t))) });
            }
            2 => {
                // a code into a character, not_a_char past the last point
                let code = *self.rng.pick(&[65i128, 97, 233, 26085, 1114111, 1114112, 55296]);
                let c = self.name("c");
                body.push(Stmt::AutoLet { name: c.clone(), value: Expr::Src(format!("char.point.from_code[{code}] otherwise `?`")) });
                body.push(Stmt::Print { signed: false, value: Expr::To(Ty::U64, Box::new(Expr::Src(format!("char.code[{}]", q(&c))))) });
            }
            3 => {
                // one character of text, or not_one_char
                let t = TEXTS[self.rng.below(TEXTS.len() as u64) as usize];
                let c = self.name("c");
                body.push(Stmt::AutoLet { name: c.clone(), value: Expr::Conv("char.point".into(), Box::new(Expr::Text(t.into())), Some(Box::new(Expr::Src("`?`".into())))) });
                body.push(Stmt::Print { signed: false, value: Expr::To(Ty::U64, Box::new(Expr::Src(format!("char.code[{}]", q(&c))))) });
            }
            _ => {
                // a digit's value is a text parse; other encodings never fail
                let d = *self.rng.pick(&["7", "0", "x", "9"]);
                let k = self.name("k");
                body.push(Stmt::AutoLet { name: k.clone(), value: Expr::Conv("i32".into(), Box::new(Expr::Conv("str.utf8".into(), Box::new(Expr::Src(format!("`{d}`"))), None)), Some(Box::new(Expr::Lit(-1)))) });
                body.push(Stmt::Print { signed: true, value: Expr::To(Ty::I64, Box::new(Expr::Var(k))) });
                let t = TEXTS[self.rng.below(TEXTS.len() as u64) as usize];
                let (s, w) = (self.name("t"), self.name("t"));
                body.push(Stmt::TypedLet { name: s.clone(), mutable: false, ty: "str.utf8".into(), value: Expr::Text(t.into()) });
                let enc = *self.rng.pick(&["str.utf16", "str.utf32"]);
                body.push(Stmt::AutoLet { name: w.clone(), value: Expr::Conv(enc.into(), Box::new(Expr::Var(s)), None) });
                body.push(Stmt::Print { signed: false, value: Expr::Src(format!("str.points[lend {}]", q(&w))) });
            }
        }
    }

    fn tier(&mut self, risky: bool) -> Tier {
        match self.rng.below(20) {
            0 | 1 if !risky && !self.positioned => Tier::Check,
            // A vouch, never on anything left unguarded on purpose.
            2 if !risky => if self.rng.chance(50) { Tier::Nocheck } else { Tier::NoChecks },
            _ => Tier::Default,
        }
    }

    fn let_stmt(&mut self) -> Stmt {
        let t = if self.rng.chance(12) { Ty::Bool } else { self.int_ty() };
        let mutability = *self.rng.pick(&[Mutability::Plain, Mutability::Plain, Mutability::Mut, Mutability::Immut]);
        let assignable = mutability != Mutability::Immut && self.rng.chance(70);
        // A strict file says `mut` or `immut` on every declaration.
        let mutability = if self.strict { if assignable { Mutability::Mut } else { Mutability::Immut } } else { mutability };
        self.risky = false;
        self.positioned = false;
        // `promise` covers every value the name will ever hold: each later
        // assignment is fitted into it too.
        let (tr, range) = self.maybe_range(t);
        let (value, promise) = if t != Ty::Bool && !self.enums.is_empty() && self.rng.chance(20) {
            let target = range.unwrap_or_else(|| self.ghost(t));
            (self.match_value(t, target), target)
        } else if t == Ty::Bool {
            let d = self.expr_depth();
            (self.bool_expr(d), Iv::new(0, 1))
        } else if let Some(r) = range {
            // A range type: every value it holds is within it.
            let (e, iv) = self.value_in(t, r);
            (e, if assignable { r } else { iv })
        } else if assignable {
            let g = self.ghost(t);
            let (e, _) = self.value_in(t, g);
            (e, g)
        } else {
            self.int_expr_typed(t)
        };
        let unseen = self.uses_unseen(&value);
        self.positioned |= unseen;
        let tier = self.tier(self.risky);
        let name = self.name("v");
        self.declare(name.clone(), t, promise, assignable);
        if unseen && tr.range.is_none() {
            self.mark_unseen(&name);
        }
        Stmt::Let { tier, name, mutability, ty: tr, value }
    }

    /// Names a declaration may shadow: plain values, not lists, enums,
    /// vectors, a walk's item or an arm's data.
    fn shadowable(&self) -> Vec<VarInfo> {
        if !self.cfg.shadow {
            return vec![];
        }
        // only the generator's own locals ('v…'): a loop's counter is the loop's,
        // and a parameter or a recursion's result is read by fixed code (a
        // function's `return 'pr'`, seed 44056)
        self.vars().filter(|v| !v.item && !v.data && v.name.starts_with('v')).cloned().collect()
    }

    /// `shadow 'x' := …;` / `shadow mut 'x': T = …;` over a visible name, in
    /// the same scope or an inner one (owner, 2026-10-08); or, as a mistake
    /// on purpose, the same with a plain `let`, which is refused.
    fn shadow_stmt(&mut self) -> Stmt {
        let old = self.rng.pick(&self.shadowable()).name.clone();
        let mistake = !self.shadow_mistake && self.rng.below(1000) < self.cfg.shadow_wrong;
        let s = self.let_stmt();
        let Stmt::Let { tier, name, mutability, ty, value } = s else { unreachable!() };
        // the new declaration takes the old name
        if let Some(v) = self.scopes.last_mut().unwrap().iter_mut().rev().find(|v| v.name == name) {
            v.name = old.clone();
        }
        let s = Stmt::Let { tier, name: old, mutability, ty, value };
        if mistake {
            self.shadow_mistake = true;
            s
        } else {
            Stmt::Shadow(Box::new(s))
        }
    }

    /// `const 'cn3': int16 = 40;` in a body (owner, 2026-10-10): a literal, or
    /// a const in scope plus a small number; a read-only name to the code
    /// after it. As a mistake on purpose, a change or a shadow of it follows.
    fn const_stmt(&mut self) -> Stmt {
        let t = self.int_ty();
        let consts: Vec<VarInfo> = self.vars().filter(|v| v.name.starts_with("cn") && v.ty == t).cloned().collect();
        let (value, iv) = match consts.is_empty() || self.rng.chance(50) {
            true => self.literal(t),
            false => {
                let c = self.rng.pick(&consts).clone();
                let r = t.range();
                let k = self.rng.range((r.lo - c.iv.lo).max(-5), (r.hi - c.iv.hi).min(5));
                (Expr::Bin(BinOp::Add, Box::new(Expr::Var(c.name)), Box::new(Expr::Lit(k))), Iv::new(c.iv.lo + k, c.iv.hi + k))
            }
        };
        let name = self.name("cn");
        let decl = Stmt::Const(Box::new(Stmt::Let { tier: Tier::Default, name: name.clone(), mutability: Mutability::Plain, ty: t.into(), value }));
        if !self.shadow_mistake && self.rng.below(1000) < self.cfg.shadow_wrong {
            // a const is never assigned to and never shadowed: refused
            self.shadow_mistake = true;
            let wrong = if self.rng.chance(50) {
                Stmt::Src(format!("change '{name}' = '{name}';"))
            } else {
                Stmt::Src(format!("shadow '{name}' := 1;"))
            };
            return Stmt::Block { tier: Tier::Default, body: vec![decl, wrong] };
        }
        self.declare(name, t, iv, false);
        decl
    }

    fn assign_stmt(&mut self) -> Stmt {
        let targets: Vec<VarInfo> = self.vars().filter(|v| v.assignable).cloned().collect();
        let v = self.rng.pick(&targets).clone();
        if self.cfg.compound && v.ty != Ty::Bool && self.rng.chance(35) {
            if let Some(st) = self.compound_change(&v) {
                return st;
            }
        }
        self.risky = false;
        self.positioned = false;
        let value = if v.ty == Ty::Bool {
            let d = self.expr_depth();
            self.bool_expr(d)
        } else {
            self.value_in(v.ty, v.iv).0
        };
        let value = if v.ty != Ty::Bool && !v.unseen && self.uses_unseen(&value) {
            // A seen name keeps what it holds visible: the guard is written
            // out, since a `check` earlier in a loop sees this value next trip.
            self.fit(value, v.ty.range(), v.iv, v.ty).0
        } else {
            value
        };
        self.positioned |= self.uses_unseen(&value);
        let tier = self.tier(self.risky);
        Stmt::Assign { tier, name: v.name, value }
    }

    /// `change 'x' = 'x' OP k;`, which the renderer writes `'x' += k;`,
    /// `'x'++;` and the rest (owner, 2026-10-14). The name keeps the range
    /// the rest of the program relies on, so the change is guarded where it
    /// could leave it: `if 'x' at_most hi - k { 'x' += k; }`. A name whose
    /// range is its whole type may instead take a risky change, unguarded,
    /// which stops the program if it overflows.
    fn compound_change(&mut self, v: &VarInfo) -> Option<Stmt> {
        use BinOp::*;
        let (lo, hi) = (v.iv.lo, v.iv.hi);
        let r = v.ty.range();
        let var = || Box::new(Expr::Var(v.name.clone()));
        let lit = |x: i128| Box::new(Expr::Lit(x));
        let at_most = |x: i128| Expr::Cmp(CmpOp::Le, var(), lit(x));
        let at_least = |x: i128| Expr::Cmp(CmpOp::Ge, var(), lit(x));
        let assign = |op: BinOp, k: i128| Stmt::Assign { tier: Tier::Default, name: v.name.clone(), value: Expr::Bin(op, var(), lit(k)) };
        let guarded = |cond: Expr, st: Stmt| Stmt::If { arms: vec![(cond, vec![st])], otherwise: None };
        if lo == r.lo && hi == r.hi && self.take_risk() {
            let op = *self.rng.pick(&[Add, Sub, Mul]);
            let k = if self.rng.chance(50) { 1 } else { self.rng.range(2, 100) };
            return Some(assign(op, k));
        }
        let isqrt = |n: i128| { let mut x = (n as f64).sqrt() as i128; while x * x > n { x -= 1; } while (x + 1) * (x + 1) <= n { x += 1; } x };
        let mut ops = vec![];
        if hi > lo {
            ops.extend([Add, Add, Sub, Sub]);
        }
        if lo >= 0 && lo * 2 <= hi {
            ops.extend([Mul, DivTrunc]);
        }
        let kmod = if lo < 0 { hi.min(-lo) + 1 } else if lo == 0 { hi + 1 } else { 0 };
        if kmod >= 2 {
            ops.push(ModTrunc);
        }
        let sq_lo = if lo <= 0 { 0 } else { let s = isqrt(lo); if s * s == lo { s } else { s + 1 } };
        if self.cfg.pow && lo >= 0 && sq_lo <= isqrt(hi) {
            ops.push(Pow);
        }
        if ops.is_empty() {
            return None;
        }
        let op = *self.rng.pick(&ops);
        let st = match op {
            Add | Sub => {
                let k = if self.rng.chance(50) { 1 } else { self.rng.range(1, (hi - lo).min(1000)) };
                if op == Add { guarded(at_most(hi - k), assign(Add, k)) } else { guarded(at_least(lo + k), assign(Sub, k)) }
            }
            Mul => {
                let k = self.rng.range(2, (hi / lo.max(1)).min(16).max(2));
                guarded(at_most(hi / k), assign(Mul, k))
            }
            DivTrunc => {
                let k = self.rng.range(2, (hi / lo.max(1)).min(16).max(2));
                if lo > 0 { guarded(at_least(lo * k), assign(DivTrunc, k)) } else { assign(DivTrunc, k) }
            }
            ModTrunc => assign(ModTrunc, self.rng.range(2, kmod.min(64))),
            _ => {
                let top = isqrt(hi);
                let cond = if sq_lo > 0 { Expr::And(Box::new(at_least(sq_lo)), Box::new(at_most(top))) } else { at_most(top) };
                guarded(cond, assign(Pow, 2))
            }
        };
        Some(st)
    }

    fn if_stmt(&mut self, depth: u32) -> Stmt {
        let narms = 1 + self.rng.below(3) as usize;
        let has_else = self.rng.chance(50);
        let mut arms = vec![];
        for _ in 0..narms {
            let c = { let d = self.expr_depth(); self.bool_expr(d) };
            let n = self.stmt_count() / 2 + 1;
            let mut body = self.block(n, depth - 1);
            if !has_else {
                if let Some(t) = self.terminal() {
                    body.push(t);
                }
            }
            arms.push((c, body));
        }
        let otherwise = has_else.then(|| {
            let n = self.stmt_count() / 2 + 1;
            self.block(n, depth - 1)
        });
        Stmt::If { arms, otherwise }
    }

    /// A statement that leaves: only ever last in a block, and only in an
    /// `if` with no `else`, so nothing after it is unreachable.
    fn terminal(&mut self) -> Option<Stmt> {
        if !self.rng.chance(20) {
            return None;
        }
        if self.in_loop != LoopKind::None && self.rng.chance(70) {
            return Some(if self.rng.chance(50) { Stmt::Break } else { Stmt::Continue });
        }
        if let Some((t, iv)) = self.in_fn {
            let e = if t == Ty::Bool { self.bool_expr(1) } else { self.value_in(t, iv).0 };
            return Some(Stmt::Return(e));
        }
        None
    }

    fn trips_allowed(&self) -> u64 {
        let left = self.budget.saturating_sub(self.spent) / self.mult.max(1);
        left.min(12)
    }

    fn for_stmt(&mut self, depth: u32) -> Stmt {
        let t = self.int_ty();
        let r = t.range();
        let step: i128 = *self.rng.pick(&[1, 1, 1, 2, 3, 7, -1, -1, -2, -3]);
        let a = step.abs();
        let n = self.rng.below(self.trips_allowed() + 1) as i128;
        let edge = self.rng.chance(35);
        // Choose the first and last value the counter takes, then the stop.
        let (start, stop, iv) = if n == 0 {
            let s = if edge { if step > 0 { r.hi } else { r.lo } } else { self.rng.range(r.lo.max(-50), r.hi.min(50)) };
            let beyond = if step > 0 { s - 1 - self.rng.range(0, 3) } else { s + 1 + self.rng.range(0, 3) };
            if !r.has(beyond) {
                // cannot go past the edge; take a one-trip loop instead
                (s, s, Iv::point(s))
            } else {
                (s, beyond, Iv::point(s))
            }
        } else {
            let span = a * (n - 1);
            let mut over = self.rng.range(0, a - 1);
            let last = if edge {
                if step > 0 { r.hi - self.rng.range(0, a - 1).min(r.hi - r.lo - span) } else { r.lo + self.rng.range(0, a - 1).min(r.hi - r.lo - span) }
            } else if step > 0 {
                self.rng.range((r.lo + span).max(-60), r.hi.min(60).max(r.lo + span))
            } else {
                self.rng.range(r.lo.max(-60).min(r.hi - span), (r.hi - span).min(60))
            };
            let first = if step > 0 { last - span } else { last + span };
            if !self.cfg.range_edge && !r.has(if step > 0 { last + a } else { last - a }) {
                over = 0; // the stop lands on the last value, so the next step is never needed
            }
            let stop = if step > 0 { (last + over).min(r.hi) } else { (last - over).max(r.lo) };
            (first, stop, Iv::new(first.min(last), first.max(last)))
        };
        let trips = if step > 0 { if stop >= start { (stop - start) / a + 1 } else { 0 } } else if start >= stop { (start - stop) / a + 1 } else { 0 };
        let name = self.name("i");
        let perm = self.rng.chance(15);
        let saved = (self.mult, self.in_loop);
        self.mult = self.mult.saturating_mul(trips.max(1) as u64);
        self.in_loop = LoopKind::Loop;
        self.scopes.push(vec![]);
        self.declare(name.clone(), t, iv, false);
        let nb = self.stmt_count() / 2 + 1;
        let body = self.stmts(nb, depth - 1);
        self.scopes.pop();
        (self.mult, self.in_loop) = saved;
        if perm {
            self.declare(name.clone(), t, iv.join(Iv::point(start)), false);
        }
        let step = if step == 1 && self.rng.chance(60) { None } else { Some(step) };
        let cheat = if self.cfg.no_cheat && self.rng.chance(15) { 1 + self.rng.below(2) as u8 } else { 0 };
        Stmt::For { perm, cheat, name, ty: t, start: Expr::Lit(start), stop: Expr::Lit(stop), step, body }
    }

    /// `while` or `loop`, driven by a counter the body cannot touch.
    /// Returns a block holding the counter's `let` and the loop.
    fn counted_loop(&mut self, depth: u32, is_loop: bool) -> Stmt {
        let t = self.int_ty();
        let k = self.rng.below(self.trips_allowed() + 1).min(t.max() as u64) as i128;
        let name = self.name("w");
        let counter = Stmt::Let { tier: Tier::Default, name: name.clone(), mutability: Mutability::Mut, ty: t.into(), value: Expr::Lit(k) };
        let dec = Stmt::Assign { tier: Tier::Default, name: name.clone(), value: Expr::Bin(BinOp::Sub, Box::new(Expr::Var(name.clone())), Box::new(Expr::Lit(1))) };
        self.scopes.push(vec![]);
        self.declare(name.clone(), t, Iv::new(0, k), false);
        let saved = (self.mult, self.in_loop);
        self.mult = self.mult.saturating_mul(k.max(1) as u64);
        self.in_loop = LoopKind::Loop;
        let nb = self.stmt_count() / 2 + 1;
        let rest = self.block(nb, depth - 1);
        (self.mult, self.in_loop) = saved;
        let var = || Box::new(Expr::Var(name.clone()));
        let the_loop = if is_loop {
            let stop = Expr::Cmp(if self.rng.chance(50) { CmpOp::Eq } else { CmpOp::Le }, var(), Box::new(Expr::Lit(0)));
            let mut body = vec![Stmt::If { arms: vec![(stop, vec![Stmt::Break])], otherwise: None }, dec];
            body.extend(rest);
            Stmt::Loop { body }
        } else {
            let mut cond = Expr::Cmp(CmpOp::Gt, var(), Box::new(Expr::Lit(0)));
            if self.rng.chance(25) {
                let extra = self.bool_expr(1);
                cond = Expr::And(Box::new(cond), Box::new(extra));
            }
            let mut body = vec![dec];
            body.extend(rest);
            Stmt::While { cond, body }
        };
        self.scopes.pop();
        Stmt::Block { tier: Tier::Default, body: vec![counter, the_loop] }
    }

    // ------------------------------------------------------------ expressions

    /// An expression of type `t` whose value lies in `target` (⊆ t's range).
    fn value_in(&mut self, t: Ty, target: Iv) -> (Expr, Iv) {
        if t == Ty::Bool {
            return ({ let d = self.expr_depth(); self.bool_expr(d) }, Iv::new(0, 1));
        }
        let d = self.expr_depth();
        let (e, iv) = self.int_expr(t, d);
        self.fit(e, iv, target, t)
    }

    /// Like `int_expr`, but never literal-only (so its type is its own, not
    /// its context's).
    fn int_expr_typed(&mut self, t: Ty) -> (Expr, Iv) {
        let d = self.expr_depth();
        let (e, iv) = self.int_expr(t, d);
        if is_literal_only(&e) {
            if let Some((v, viv)) = self.typed_leaf(t) {
                return self.combine_add(t, e, iv, v, viv);
            }
        }
        (e, iv)
    }

    /// `a + b` with `b` fitted so the sum stays in `t`; `a` is literal-only.
    fn combine_add(&mut self, t: Ty, a: Expr, ia: Iv, b: Expr, ib: Iv) -> (Expr, Iv) {
        let r = t.range();
        let (lo, hi) = (r.lo - ia.lo, r.hi - ia.hi);
        let Some(allowed) = (lo <= hi).then(|| Iv::new(lo, hi)).and_then(|x| x.meet(r)) else { return (b, ib) };
        let (b, ib) = self.fit(b, ib, allowed, t);
        (Expr::Bin(BinOp::Add, Box::new(a), Box::new(b)), Iv::new(ia.lo + ib.lo, ia.hi + ib.hi))
    }

    fn typed_leaf(&mut self, t: Ty) -> Option<(Expr, Iv)> {
        let same = self.vars_of(t);
        if !same.is_empty() {
            let v = self.rng.pick(&same).clone();
            return Some((Expr::Var(v.name), v.iv));
        }
        let others: Vec<VarInfo> = self.vars().filter(|v| v.ty != Ty::Bool).cloned().collect();
        if others.is_empty() {
            return None;
        }
        let v = self.rng.pick(&others).clone();
        Some(self.convert(Expr::Var(v.name), v.iv, v.ty, t))
    }

    /// `to.<to>[e]`, with `e` first folded into what `to` holds (unless risky).
    fn convert(&mut self, e: Expr, iv: Iv, from: Ty, to: Ty) -> (Expr, Iv) {
        let both = from.range().meet(to.range()).unwrap();
        let (e, iv) = if iv.within(to.range()) || self.take_risk() {
            (e, iv.meet(to.range()).unwrap_or(to.range()))
        } else {
            self.fit(e, iv, both, from)
        };
        (Expr::To(to, Box::new(e)), iv)
    }

    fn take_risk(&mut self) -> bool {
        if self.in_check > 0 {
            return false;
        }
        let r = self.rng.below(1000) < self.cfg.risk;
        if r {
            self.risky = true;
        }
        r
    }

    /// Fold `e` (of type `t`, value in `iv`) into `target`. The result's
    /// interval is within `target`, possibly narrower.
    fn fit(&mut self, e: Expr, iv: Iv, target: Iv, t: Ty) -> (Expr, Iv) {
        if iv.within(target) {
            return (e, iv);
        }
        let mut tg = target;
        if tg.lo == t.min() && t.signed() {
            tg.lo += 1; // `- 128` would need 128 as an int8 literal
            if tg.lo > tg.hi {
                return (Expr::Lit(t.min()), Iv::point(t.min()));
            }
        }
        if tg.width() > t.max() {
            let w = t.max();
            let lo = self.rng.range(tg.lo, tg.hi - w + 1);
            tg = Iv::new(lo, lo + w - 1);
        }
        if let Expr::Lit(_) = e {
            let v = self.rng.range(tg.lo, tg.hi);
            return (Expr::Lit(v), Iv::point(v));
        }
        let w = tg.width();
        let m = Expr::Bin(BinOp::Mod, Box::new(e), Box::new(Expr::Lit(w)));
        let e = if tg.lo == 0 {
            m
        } else if tg.lo > 0 {
            Expr::Bin(BinOp::Add, Box::new(m), Box::new(Expr::Lit(tg.lo)))
        } else {
            Expr::Bin(BinOp::Sub, Box::new(m), Box::new(Expr::Lit(-tg.lo)))
        };
        (e, tg)
    }

    fn literal(&mut self, t: Ty) -> (Expr, Iv) {
        let r = t.range();
        let pick = if self.cfg.edge && self.rng.chance(60) { 0 } else { self.rng.below(3) };
        let v = match pick {
            0 => {
                let edges = [r.lo, r.lo + 1, -1, 0, 1, 2, r.hi - 1, r.hi, r.hi / 2, r.hi / 2 + 1, r.lo / 2];
                let ok: Vec<i128> = edges.into_iter().filter(|v| r.has(*v)).collect();
                *self.rng.pick(&ok)
            }
            1 => self.rng.range(r.lo.max(-20), r.hi.min(20)),
            _ => self.rng.range(r.lo, r.hi),
        };
        (Expr::Lit(v), Iv::point(v))
    }

    fn call(&mut self, t: Ty) -> Option<(Expr, Iv)> {
        // Calls inside call arguments, at most 3 deep: a cheap function's
        // budget alone let one expression nest 2000 brackets (seed 32299).
        if self.call_nest >= 3 {
            return None;
        }
        let fits: Vec<Sig> = self.sigs[..self.callable]
            .iter()
            .filter(|s| s.ret == t && (self.in_check == 0 || self.strict || s.ret_ranged) && self.spent + s.cost.saturating_mul(self.mult) <= self.budget)
            .cloned()
            .collect();
        if fits.is_empty() {
            return None;
        }
        let s = self.rng.pick(&fits).clone();
        self.spent += s.cost.saturating_mul(self.mult);
        let mut args = vec![];
        self.call_nest += 1;
        for (_, pt, piv) in &s.params {
            let (a, _) = if *pt == Ty::Bool { (self.bool_expr(1), Iv::new(0, 1)) } else { self.value_in(*pt, *piv) };
            args.push(a);
        }
        self.call_nest -= 1;
        Some((Expr::Call(s.name.clone(), args), s.ret_iv))
    }

    fn leaf(&mut self, t: Ty) -> (Expr, Iv) {
        let same = self.vars_of(t);
        let others = self.vars().any(|v| v.ty != t && v.ty != Ty::Bool);
        let can_call = self.sigs[..self.callable].iter().any(|s| s.ret == t && (self.in_check == 0 || self.strict || s.ret_ranged));
        let readable = self.readable(Some(t));
        let items: Vec<VarInfo> = self.vars().filter(|v| v.ty == t && self.through_to(v)).cloned().collect();
        // A lane's range is the generator's own: never inside `check`.
        let simds = if self.in_check > 0 { vec![] } else { self.simd_vars(Some(t)) };
        let has_list = !self.lists().is_empty();
        let w = [
            if same.is_empty() { 0 } else { 6 },
            if others { 2 } else { 0 },
            3,
            if can_call { 1 } else { 0 },
            if readable.is_empty() { 0 } else { 3 },
            if t == Ty::U64 && has_list && self.in_check == 0 { 1 } else { 0 },
            if items.is_empty() { 0 } else { 4 },
            if simds.is_empty() { 0 } else { 3 },
        ];
        match self.rng.weighted(&w) {
            4 => {
                let v = self.rng.pick(&readable).clone();
                let p = self.position(&v);
                (Expr::At(v.name, Box::new(p)), v.iv)
            }
            7 => {
                let v = self.rng.pick(&simds).clone();
                self.simd_read(v, t)
            }
            6 => {
                let v = self.rng.pick(&items).clone();
                (Expr::To(t, Box::new(Expr::Var(v.name))), v.iv)
            }
            5 => {
                let ls = self.lists();
                let v = self.rng.pick(&ls).clone();
                let l = v.list.unwrap();
                (Expr::Len(v.name), Iv::new(l.lo as i128, l.cap as i128))
            }
            0 => {
                let v = self.rng.pick(&same).clone();
                (Expr::Var(v.name), v.iv)
            }
            1 => {
                let os: Vec<VarInfo> = self.vars().filter(|v| v.ty != t && v.ty != Ty::Bool).cloned().collect();
                let v = self.rng.pick(&os).clone();
                self.convert(Expr::Var(v.name), v.iv, v.ty, t)
            }
            3 => self.call(t).unwrap_or_else(|| self.literal(t)),
            _ => self.literal(t),
        }
    }

    fn int_expr(&mut self, t: Ty, depth: u32) -> (Expr, Iv) {
        if depth == 0 || self.rng.chance(30) {
            return self.leaf(t);
        }
        let s = t.signed();
        let has_var = !self.vars_of(t).is_empty();
        use BinOp::*;
        let more = if self.cfg.roundings { 1 } else { 0 };
        let ops: [(Option<BinOp>, u64); 22] = [
            (Some(Add), 6),
            (Some(Sub), 6),
            (Some(Mul), 5),
            (Some(Div), 3),
            (Some(Mod), 3),
            (Some(Pow), if self.cfg.pow { 1 } else { 0 }),
            (Some(AndBit), 1),
            (Some(OrBit), 1),
            (Some(Xor), 1),
            (Some(Shl), 1),
            (Some(Shr), 1),
            (Some(WAdd), if has_var { 1 } else { 0 }),
            (Some(WSub), if has_var { 1 } else { 0 }),
            (Some(WMul), if has_var { 1 } else { 0 }),
            (None, if s { 2 } else { 0 }),                  // negation
            (Some(Mod), 0),
            (Some(DivTrunc), more),
            (Some(ModTrunc), more),
            (Some(DivCeil), more),
            (Some(ModCeil), more),
            (Some(DivNear), more),
            (Some(ModNear), more),
        ];
        let w: Vec<u64> = ops.iter().map(|o| o.1).collect();
        let pick = ops[self.rng.weighted(&w)].0;
        let Some(op) = pick else { return self.neg(t, depth) };
        if matches!(op, AndBit | OrBit | Xor) && has_var && self.rng.chance(15) {
            // not.bit: of a name, or of any typed operand once the width may
            // come from context
            let (a, ia) = if self.cfg.width_from_context && self.rng.chance(50) {
                let (e, iv) = self.int_expr_typed(t);
                (if self.cfg.sized_bits && !matches!(e, Expr::Var(_) | Expr::To(..)) { Expr::To(t, Box::new(e)) } else { e }, iv)
            } else {
                let vs = self.vars_of(t);
                let v = self.rng.pick(&vs).clone();
                (Expr::Var(v.name), v.iv)
            };
            let iv = if s { Iv::new(-ia.hi - 1, -ia.lo - 1) } else { Iv::new(t.max() - ia.hi, t.max() - ia.lo) };
            return (Expr::NotBit(Box::new(a)), iv);
        }
        let (a, ia) = if matches!(op, WAdd | WSub | WMul) && !(self.cfg.width_from_context && self.rng.chance(50)) {
            let vs = self.vars_of(t);
            let v = self.rng.pick(&vs).clone();
            (Expr::Var(v.name), v.iv)
        } else if matches!(op, WAdd | WSub | WMul) {
            self.int_expr_typed(t)
        } else {
            self.int_expr(t, depth - 1)
        };
        let (b, ib) = self.int_expr(t, depth - 1);
        // A shift's width comes from the shifted number, or its context when
        // that is bare numbers alone, never from the count; with no width
        // anywhere `shl` and `shr` are refused (owner, 2026-10-07; 4866c834).
        // Bare numbers alone on the left become a typed name: their context
        // may give no width (a comparison, an enum's data), and the generator
        // does not follow which contexts do.
        let bare_left = matches!(op, Shl | Shr) && is_literal_only(&a);
        let (op, a, ia) = if bare_left {
            match self.typed_leaf(t) {
                Some((v, iv)) => (op, v, iv),
                None => (Add, a, ia),
            }
        } else {
            (op, a, ia)
        };
        self.binary(t, op, a, ia, b, ib)
    }

    fn neg(&mut self, t: Ty, depth: u32) -> (Expr, Iv) {
        let (a, ia) = self.int_expr(t, depth - 1);
        let r = t.range();
        let (a, ia) = if ia.lo > r.lo || self.take_risk() { (a, ia) } else { self.fit(a, ia, Iv::new(r.lo + 1, r.hi), t) };
        let iv = Iv::new(-ia.hi, -ia.lo).meet(r).unwrap_or(r);
        (Expr::Neg(Box::new(a)), iv)
    }

    fn binary(&mut self, t: Ty, op: BinOp, a: Expr, ia: Iv, b: Expr, ib: Iv) -> (Expr, Iv) {
        use BinOp::*;
        let bitty = matches!(op, AndBit | OrBit | Xor | Shl | Shr | WAdd | WSub | WMul);
        let size = |e: Expr| -> Expr {
            match e {
                Expr::Var(_) | Expr::To(..) | Expr::Lit(_) => e,
                e => Expr::To(t, Box::new(e)),
            }
        };
        let (a, b) = if bitty && self.cfg.sized_bits { (size(a), size(b)) } else { (a, b) };
        // Bare numbers on both sides leave the operator no width at all: no
        // ruling yet on what that means (design claude, 2026-09-24), so one
        // side becomes a typed name.
        let (a, ia) = if bitty && is_literal_only(&a) && is_literal_only(&b) {
            match self.typed_leaf(t) {
                Some((v, iv)) => (v, iv),
                // Nothing typed in scope: no operator at all (seed 444).
                None => return (a, ia),
            }
        } else {
            (a, ia)
        };
        let r = t.range();
        let bits = t.bits() as i128;
        let risk = self.take_risk();
        // Operands are sized again here, after any `fit`: its `(e mod.floor w) + lo`
        // is an exact answer too since 15c5c71, and has no size of its own.
        let sized = bitty && self.cfg.sized_bits;
        let mk = |a: Expr, b: Expr| if sized { Expr::Bin(op, Box::new(size(a)), Box::new(size(b))) } else { Expr::Bin(op, Box::new(a), Box::new(b)) };
        let clip = |iv: Option<Iv>| iv.and_then(|x| x.meet(r)).unwrap_or(r);
        match op {
            Add | Sub => {
                let f = |x: i128, y: i128| Some(if op == Add { x + y } else { x - y });
                let out = corners(ia, ib, &f).unwrap();
                if out.within(r) || risk {
                    return (mk(a, b), clip(Some(out)));
                }
                // allowed b given a
                let (lo, hi) = if op == Add { (r.lo - ia.lo, r.hi - ia.hi) } else { (ia.hi - r.hi, ia.lo - r.lo) };
                let (a, ia) = if lo > hi || Iv::new(lo.min(hi), hi.max(lo)).meet(r).is_none() {
                    let half = Iv::new(r.lo / 2, r.hi / 2);
                    self.fit(a, ia, half, t)
                } else {
                    (a, ia)
                };
                let (lo, hi) = if op == Add { (r.lo - ia.lo, r.hi - ia.hi) } else { (ia.hi - r.hi, ia.lo - r.lo) };
                let allowed = if lo <= hi { Iv::new(lo, hi).meet(r) } else { None };
                let Some(allowed) = allowed else {
                    // fall back to something that certainly fits
                    return (a, ia);
                };
                let (b, ib) = self.fit(b, ib, allowed, t);
                let out = corners(ia, ib, &f).unwrap();
                (mk(a, b), clip(Some(out)))
            }
            Mul => {
                let f = |x: i128, y: i128| x.checked_mul(y);
                let out = corners(ia, ib, &f);
                if out.map_or(false, |o| o.within(r)) || risk {
                    return (mk(a, b), clip(out));
                }
                let s = isqrt(r.hi);
                let bx = if t.signed() { Iv::new(-s, s) } else { Iv::new(0, s) };
                let (a, ia) = self.fit(a, ia, bx, t);
                let (b, ib) = self.fit(b, ib, bx, t);
                (mk(a, b), clip(corners(ia, ib, &f)))
            }
            Div | Mod | DivTrunc | ModTrunc | DivCeil | ModCeil | DivNear | ModNear => {
                let bad_zero = ib.has(0);
                let bad_min = t.signed() && ia.has(r.lo) && ib.has(-1);
                let (b, ib) = if (bad_zero || bad_min) && !risk {
                    let k = if self.rng.chance(50) { self.rng.range(1, 10) } else { self.rng.range(1, r.hi) };
                    let neg = t.signed() && !ia.has(r.lo) && self.rng.chance(25);
                    let tg = if neg { Iv::new(-k, -1) } else { Iv::new(1, k) };
                    self.fit(b, ib, tg, t)
                } else {
                    (b, ib)
                };
                if ib.has(0) {
                    // risky: division by zero possible
                    return (mk(a, b), r);
                }
                let out = if op.division() == Some(false) {
                    corners(ia, ib, |x, y| op.quotient(x, y))
                } else {
                    let (lo, hi) = op.mod_bounds((ia.lo, ia.hi), (ib.lo, ib.hi));
                    Some(Iv::new(lo, hi))
                };
                // `mod.ceil` / `mod.near` can leave the type (below 0 on an
                // unsigned one): a site, so `mod.floor` unless a risk is taken
                if op.division() == Some(true) && !risk && !out.map_or(false, |o| o.within(r)) {
                    let (lo, hi) = BinOp::Mod.mod_bounds((ia.lo, ia.hi), (ib.lo, ib.hi));
                    return (Expr::Bin(BinOp::Mod, Box::new(a), Box::new(b)), clip(Some(Iv::new(lo, hi))));
                }
                (mk(a, b), clip(out))
            }
            Pow => {
                let (a, ia) = if risk { (a, ia) } else { self.fit(a, ia, if t.signed() { Iv::new(-3, 3) } else { Iv::new(0, 3) }, t) };
                let mut emax = 0i128;
                while 3i128.pow(emax as u32 + 1) <= r.hi {
                    emax += 1;
                }
                let (b, ib) = if ib.lo >= 0 && ib.hi <= emax { (b, ib) } else { self.fit(b, ib, Iv::new(0, emax), t) };
                if ia.width() > 7 || ib.lo < 0 || ib.hi > 64 {
                    return (mk(a, b), r);
                }
                let mut lo = i128::MAX;
                let mut hi = i128::MIN;
                for x in ia.lo..=ia.hi {
                    for e in ib.lo..=ib.hi {
                        let v = x.checked_pow(e as u32).unwrap_or(i128::MAX);
                        lo = lo.min(v);
                        hi = hi.max(v);
                    }
                }
                (mk(a, b), clip(Some(Iv::new(lo, hi))))
            }
            AndBit | OrBit | Xor => {
                // A risk taken: an exact operand pushed past the operator's
                // width, where it is a narrowing site (CCA c1973f02).
                let (b, ib) = if risk && self.rng.chance(40) {
                    if self.rng.chance(50) {
                        let k = ib.lo - r.lo + self.rng.range(1, 20);
                        (Expr::Bin(Sub, Box::new(b), Box::new(Expr::Lit(k))), Iv::new(ib.lo - k, ib.hi - k))
                    } else {
                        let k = r.hi - ib.hi + self.rng.range(1, 20);
                        (Expr::Bin(Add, Box::new(b), Box::new(Expr::Lit(k))), Iv::new(ib.lo + k, ib.hi + k))
                    }
                } else {
                    (b, ib)
                };
                if ia.lo >= 0 && ib.lo >= 0 {
                    let top = ia.hi.max(ib.hi);
                    let mask = if top == 0 { 0 } else { (1i128 << (128 - top.leading_zeros())) - 1 };
                    let iv = if op == AndBit { Iv::new(0, ia.hi.min(ib.hi)) } else { Iv::new(0, mask) };
                    (mk(a, b), clip(Some(iv)))
                } else {
                    (mk(a, b), r)
                }
            }
            Shl | Shr => {
                let (b, ib) = if (ib.lo >= 0 && ib.hi < bits) || risk { (b, ib) } else {
                    let k = self.rng.range(0, bits - 1);
                    if self.rng.chance(60) { (Expr::Lit(k), Iv::point(k)) } else { self.fit(b, ib, Iv::new(0, bits - 1), t) }
                };
                if ib.lo < 0 || ib.hi >= bits {
                    return (mk(a, b), r);
                }
                if op == Shl {
                    // Bound the shift, not the operand, so the operand stays a name.
                    let fits = |k: i128| corners(ia, Iv::new(0, k), |x, s| x.checked_mul(1i128 << s)).map_or(false, |o| o.within(r));
                    // A risk taken: often a small, valid count on a value that
                    // may be too big to shift, so only the answer can leave the
                    // type (a wrong layer-3 proof hid there, seed 9148).
                    let (b, ib) = if risk && ib.hi >= 1 && self.rng.chance(50) {
                        let k = self.rng.range(1, 3.min(bits - 1));
                        (Expr::Lit(k), Iv::point(k))
                    } else {
                        (b, ib)
                    };
                    let (b, ib) = if fits(ib.hi) || risk { (b, ib) } else {
                        let mut k = 0;
                        while k + 1 < bits && fits(k + 1) {
                            k += 1;
                        }
                        self.fit(b, ib, Iv::new(0, k), t)
                    };
                    let out = corners(ia, ib, |x, k| x.checked_mul(1i128 << k));
                    (mk(a, b), clip(out))
                } else if ia.lo >= 0 {
                    (mk(a, b), Iv::new(ia.lo >> ib.hi, ia.hi >> ib.lo))
                } else {
                    (mk(a, b), r)
                }
            }
            WAdd | WSub | WMul => (mk(a, b), r),
        }
    }

    fn bool_expr(&mut self, depth: u32) -> Expr {
        let bools = self.vars_of(Ty::Bool);
        let svs = if self.in_check > 0 { vec![] } else { self.simd_vars(None) };
        if !svs.is_empty() && self.rng.chance(8) {
            let v = self.rng.pick(&svs).clone();
            let other = self.simd_operand(v.ty, v.iv);
            let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Ne, CmpOp::Lt, CmpOp::Gt, CmpOp::Le, CmpOp::Ge]);
            let f = if self.rng.chance(50) { "all" } else { "any" };
            return Expr::SimdFold(f, Box::new(Expr::Cmp(op, Box::new(Expr::Var(v.name)), Box::new(other.0))));
        }
        let evs = self.enum_vars();
        if !evs.is_empty() && self.rng.chance(8) {
            let v = self.rng.pick(&evs).clone();
            let c = self.case_value(v.en.unwrap(), false);
            let op = if self.rng.chance(50) { CmpOp::Eq } else { CmpOp::Ne };
            return Expr::Cmp(op, Box::new(Expr::Var(v.name)), Box::new(c));
        }
        if self.rng.chance(6) {
            if let Some(g) = self.guarded() {
                return g;
            }
        }
        if depth == 0 || self.rng.chance(35) {
            let w = [1, if bools.is_empty() { 0 } else { 3 }, 6];
            return match self.rng.weighted(&w) {
                0 => Expr::Bool(self.rng.chance(50)),
                1 => Expr::Var(self.rng.pick(&bools).name.clone()),
                _ => self.comparison(),
            };
        }
        match self.rng.below(5) {
            0 | 1 => {
                let (a, b) = (Box::new(self.bool_expr(depth - 1)), Box::new(self.bool_expr(depth - 1)));
                if self.rng.chance(30) { Expr::AndThen(a, b) } else { Expr::And(a, b) }
            }
            2 | 3 => {
                let (a, b) = (Box::new(self.bool_expr(depth - 1)), Box::new(self.bool_expr(depth - 1)));
                if self.rng.chance(30) { Expr::OrElse(a, b) } else { Expr::Or(a, b) }
            }
            _ => Expr::Not(Box::new(self.bool_expr(depth - 1))),
        }
    }

    /// A division that only its guard makes safe: `'v' !== 0 and.then
    /// (k div 'v') < m`, or `'v' == 0 or.else …` (owner, 2026-09-25). The
    /// right side is left unguarded on purpose; Precog must see the left.
    fn guarded(&mut self) -> Option<Expr> {
        let vs: Vec<VarInfo> = self.vars().filter(|v| v.ty != Ty::Bool).cloned().collect();
        if vs.is_empty() {
            return None;
        }
        let v = self.rng.pick(&vs).clone();
        let t = v.ty;
        let r = t.range();
        // The numerator: never the type's minimum, so `div -1` cannot overflow.
        let k = self.rng.range(r.lo.max(-50), r.hi.min(50));
        let op = if self.rng.chance(50) { BinOp::Div } else { BinOp::Mod };
        let var = || Box::new(Expr::Var(v.name.clone()));
        // Sometimes the excluded value sits inside the range, `'v' == j
        // or.else … div ('v' - j)`, where an interval cannot leave it out
        // (compiler claude, 2026-09-30); only below 64 bits, so 'v' - j
        // cannot pass the 64-bit limit in between.
        let j = if t.bits() < 64 && self.rng.chance(35) { self.rng.range(r.lo.max(-50), r.hi.min(50)) } else { 0 };
        if j != 0 {
            let q = Expr::Bin(op, Box::new(Expr::Lit(k)), Box::new(Expr::Bin(BinOp::Sub, var(), Box::new(Expr::Lit(j)))));
            let m = self.rng.range(-3, 3);
            let right = Box::new(Expr::Cmp(CmpOp::Lt, Box::new(q), Box::new(Expr::Lit(m))));
            return Some(if self.rng.chance(50) {
                Expr::AndThen(Box::new(Expr::Cmp(CmpOp::Ne, var(), Box::new(Expr::Lit(j)))), right)
            } else {
                Expr::OrElse(Box::new(Expr::Cmp(CmpOp::Eq, var(), Box::new(Expr::Lit(j)))), right)
            });
        }
        let q = Expr::Bin(op, Box::new(Expr::Lit(k)), var());
        let m = self.rng.range(-3, 3).max(r.lo);
        let cmp = *self.rng.pick(&[CmpOp::Lt, CmpOp::Ge, CmpOp::Eq, CmpOp::Ne]);
        let right = Box::new(Expr::Cmp(cmp, Box::new(q), Box::new(Expr::Lit(m))));
        Some(if self.rng.chance(50) {
            Expr::AndThen(Box::new(Expr::Cmp(CmpOp::Ne, var(), Box::new(Expr::Lit(0)))), right)
        } else {
            Expr::OrElse(Box::new(Expr::Cmp(CmpOp::Eq, var(), Box::new(Expr::Lit(0)))), right)
        })
    }

    fn comparison(&mut self) -> Expr {
        let t = self.int_ty();
        let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Ne, CmpOp::Lt, CmpOp::Gt, CmpOp::Le, CmpOp::Ge]);
        let d = self.expr_depth().min(2);
        let (a, _) = self.int_expr(t, d);
        // A comparison compares true values (owner, dc83c05), so its sides
        // may differ in type; in edge mode they often do.
        let tb = if self.cfg.edge && self.rng.chance(40) { self.int_ty() } else { t };
        let (b, _) = self.int_expr(tb, d);
        if is_literal_only(&a) && is_literal_only(&b) {
            match self.typed_leaf(t) {
                Some((v, _)) => return Expr::Cmp(op, Box::new(v), Box::new(b)),
                None => return Expr::Bool(self.rng.chance(50)),
            }
        }
        Expr::Cmp(op, Box::new(a), Box::new(b))
    }
}

