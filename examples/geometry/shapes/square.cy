public function area [$s: float.binary.64_bit] -> float.binary.64_bit:
    return $side times $s times $s;
end of function area
public.kit function name [] -> string.utf8:
    return "square";
end of function name
constant $side := 1.0;
