public constant $pi: float.binary.64_bit = 3.141_592_653_589_793;
public record Circle:
    radius: float.binary.64_bit;
end of record Circle
public function area [$r: float.binary.64_bit] -> float.binary.64_bit:
    return $pi times $r times $r;
end of function area
public.kit function name [] -> string.utf8:
    return "circle";
end of function name
