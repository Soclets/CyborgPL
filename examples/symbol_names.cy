# A quoted name may hold symbols (owner, 2026-10-16: "Yeah, allow it"): any printable character but `'`, `.`
# and whitespace. Good naming depends on what you would use (style.md 17).
MAIN {
    let '80%' := 80;
    let 'ready?' := true;
    let 'a+b' := 2 + 3;
    std::print.useful['80%', " ", 'ready?', " ", 'a+b', newln];
}
