# The root of the kit: its items are `geometry::…` from outside.


const $unit := 1.0;

fn helper [] -> str.utf8:
    ret "from the root";
end of fn helper

module report:
    pub.kit proc line [$label: str.utf8, $v: float.bin.64]:
        std::print.shortest($label, ": ", $v, newln);
    end of proc line
end of module report

!MAIN:
    report.line("circle", shapes.circle.area(2.0));
    report.line("square", shapes.square.area(3.0));
    report.line("unit circle", shapes.circle.area($unit));
    std::print.shortest(shapes.describe(), newln);
    let $r: shapes.circle.Circle = shapes.circle.Circle[radius = 1.5];
    std::print.shortest($r.radius, " ", shapes.circle.$pi, newln);
!end of MAIN
