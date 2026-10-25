# Finding 2026-10-25 (check metal, seed 13: the plain loop, not the GPU): `print.useful` of a bin is the
# shortest decimal that reads back as the same value, "Ryu-style" (owner, 2026-09-14). 5305.65625 as an f32
# has two such 8-digit forms, 5305.6562 and 5305.6563, and the value is exactly halfway between them. Ryu
# breaks that tie to the even digit, 5305.6562, as the oracle prints; kayo prints 5305.6563. Both print
# the stored value, 5305.65625, alike. (The tie rule is not written beyond "Ryu-style"; asked of design claude.)
MAIN {
    let $a: f32 = 5305.65625;
    let $b: f32 = 0.5;
    std::print.useful($a, " ", $a x $b, newln);
    std::print.stored($a, newln);
}
