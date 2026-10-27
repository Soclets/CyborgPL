# The root of the kit: its items are `geometry::…` from outside.


constant $unit := 1.0;

function helper [] -> string.utf8:
    return "from the root";
end of function helper

module report:
    public.kit procedure line [$label: string.utf8, $v: float.binary.64_bit]:
        std::print.shortest($label, ": ", $v, newline);
    end of procedure line
end of module report

!MAIN:
    report.line("circle", shapes.circle.area(2.0));
    report.line("square", shapes.square.area(3.0));
    report.line("unit circle", shapes.circle.area($unit));
    std::print.shortest(shapes.describe(), newline);
    let $r: shapes.circle.Circle = shapes.circle.Circle[radius = 1.5];
    std::print.shortest($r.radius, " ", shapes.circle.$pi, newline);
!end of MAIN
