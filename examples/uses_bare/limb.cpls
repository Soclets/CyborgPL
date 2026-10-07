# `greet` marks `hello` and `Note` bare.optional, `wave` and `Tone` bare.only, and `formal`, `Tally`
# and `merge` bare.never (owner, 2026-10-06).

fn describe ['t': Tone] -> str.utf8 {
    return match 't' { soft { is "soft"; } loud { is "loud"; } };
}

MAIN {
    print.useful[hello[], " ", greet::hello[], " ", wave[], " ", greet::formal[], newln];
    let 't' := greet::Tally[n = 2] greet::merge greet::Tally[n = 3];
    print.useful['t'.n, newln];
    let 'a' := Note[text = "bare"];
    let 'b': greet::Note = greet::Note[text = "with the path"];
    print.useful['a'.text, ", ", 'b'.text, ", ", describe[Tone.loud], newln];
}
