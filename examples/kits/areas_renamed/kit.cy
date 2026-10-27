# A copy of examples/kits/areas under the prefix `shapes`, as Synthia makes one for
# `kit.areas2.prefix = shapes` (owner, 2026-09-30); `my.` here is this kit's root.


public constant $pi: float.binary.64_bit = 3.14159;

public record Circle:
    radius: float.binary.64_bit;
end of record Circle

public function circle [lent $c: Circle] -> float.binary.64_bit:
    return $c.radius times $c.radius times my.$pi;
end of function circle

public.kit function internal [] -> integer.signed.32_bit:
    return 2;
end of function internal

function hidden [] -> integer.signed.32_bit:
    return 1;
end of function hidden

public module shapes:
    public function square [$s: integer.signed.32_bit] -> integer.signed.32_bit:
        return $s times $s;
    end of function square
end of module shapes

# A kit's own word operator with a narrowed form (owner, 2026-10-02/03): users write it bare or
# with this kit's path, `areas::mean.low`.
# its levels apply wherever `mean` is used, in other kits too (owner, 2026-10-03)
precedence.low_first:
    level 1 [ plus ] left_first
    level 2 [ times, mean, mean.low ] left_first
end of precedence.low_first

public operator mean [$a: float.binary.64_bit, $b: float.binary.64_bit] -> float.binary.64_bit:
    return ($a plus $b) divide.nearest 2.0;
end of operator mean
public operator mean.low [$a: float.binary.64_bit, $b: float.binary.64_bit] -> float.binary.64_bit:
    return ($a plus $b) divide.floor 2.0;
end of operator mean.low
