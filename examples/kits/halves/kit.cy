# A kit's `pub coll.handler` (owner, 2026-10-23: "Should `pub` handlers be a thing? (the suggestor AI) No shit"),
# its member taking arguments (owner: "Yes"): another kit uses it through its path,
# `otherwise halves::half_failed.or_default(default: 0, label: "…")`.
public function half [$n: integer.signed.32_bit] -> integer.signed.32_bit or fail[odd, negative: integer.signed.32_bit]:
    if $n less_than 0:
        fail negative[$n];
    end of if
    if ($n divide.floor 2) times 2 not_equal_to $n:
        fail odd;
    end of if
    return $n divide.floor 2;
end of function half

public collection.handler half_failed:
    half_failed.or_default [$default: integer.signed.32_bit, $label: string.utf8]:
        !case odd:
            std::print.shortest($label, ": odd, using ", $default, newline); return $default;
        !end of case odd
        !case negative $v:
            std::print.shortest($label, ": ", $v, " is negative, using ", $default, newline); return $default;
        !end of case negative
    end of half_failed.or_default
end of collection.handler half_failed
