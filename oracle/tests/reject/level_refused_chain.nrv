op ~= ['a': bin64, 'b': bin64] -> bool { return 'a' equal_to 'b'; }
precedence.low_first {
    level 1 { equal_to, ~= } refused
}
MAIN {
    let 'x': bin64 = 1.0;
    let 'k' := 'x' ~= 'x' ~= 'x';
}
