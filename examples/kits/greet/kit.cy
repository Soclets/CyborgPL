# A kit for uses_paths: other kits write its names with `greet::` (owner, 2026-10-22), and its operator `merge` bare
# or with the path, and its glyph `<+>` the same.

public function hello [] -> string.utf8:
    return "hello";
end of function hello

public function wave [] -> string.utf8:
    return "wave";
end of function wave

public function formal [] -> string.utf8:
    return "good day";
end of function formal

public record Tally:
    n: integer.signed.32_bit;
end of record Tally

public operator merge [$a: Tally, $b: Tally] -> Tally:
    return Tally[n = $a.n plus $b.n];
end of operator merge

public record Note:
    text: string.utf8;
end of record Note

public union.tagged Tone:
    soft;
    loud;
end of union.tagged Tone

public operator <+> [$a: Tally, $b: Tally] -> Tally:
    return Tally[n = $a.n plus $b.n plus 1];
end of operator <+>
