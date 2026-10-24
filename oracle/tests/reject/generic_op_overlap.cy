# A generic operator and a specific one for a type it covers: two meanings, an error — the specific
# one does not quietly win (owner, 2026-09-29: a).
interface Number <T> {
    op minus [$a: <T>, $b: <T>] -> <T>;
    op < [$a: <T>, $b: <T>] -> bool;
}
op ~= <T: Number> [$a: <T>, $b: <T>] -> bool { return $a < $b; }
op ~= [$a: i32, $b: i32] -> bool { return true; }
MAIN { }
