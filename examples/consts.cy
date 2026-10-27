# Constants, worked out while building, `pub` and not, used in MAIN.

public constant $max := 8;
constant $pi: float.binary.64_bit = 3.141_592_653_589_793;
constant $twice_max := $max times 2;
constant $greeting := "hello " concatenate "world";
public record P:
    x: integer.signed.32_bit;
end of record P
public function area [$r: float.binary.64_bit] -> float.binary.64_bit:
    return $pi times $r times $r;
end of function area
!MAIN:
    std::print.shortest($max, " ", $twice_max, " ", $greeting, " ", area(2.0), newline);
    let $s := $greeting;
    let $p := P[x = $max];
    for $i [from: 1, to: $max, by: 4]:
        std::print.shortest($i, " ");
    end of for
    std::print.stored(newline);
!end of MAIN
