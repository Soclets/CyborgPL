# A generic operator and a specific one for a type it covers: two meanings, an error — the specific
# one does not quietly win (owner, 2026-09-29: a).
interface Number <T>:
    operator minus [$a: <T>, $b: <T>] -> <T>;
    operator less_than [$a: <T>, $b: <T>] -> boolean;
end of interface Number
operator ~= <T: Number> [$a: <T>, $b: <T>] -> boolean:
    return $a less_than $b;
end of operator ~=
operator ~= [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> boolean:
    return true;
end of operator ~=
!MAIN:
!end of MAIN
