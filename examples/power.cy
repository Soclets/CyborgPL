# `pow`, and how it groups (to the right).
MAIN:
    std::println((2 ^ (3 ^ 2)));
    std::println(((2 ^ 3) ^ 2));
    let $kilo := 2 ^ 10;
    std::println($kilo);
    std::println((10 - 3 - 2));
;MAIN
