# The root of the kit: its items are `geometry::…` from outside.


const $unit := 1.0;

fn helper [] -> str.utf8:
    ret "from the root";
;fn helper

module report:
    pub.kit proc line [let $label: str.utf8, let $v: f64]:
        std::print($label, ": ", $v, newln);
    ;proc line
;module report

MAIN:
    report.line("circle", shapes.circle.area(2.0));
    report.line("square", shapes.square.area(3.0));
    report.line("unit circle", shapes.circle.area($unit));
    std::print(shapes.describe(), newln);
    let $r: shapes.circle.Circle := shapes.circle.Circle[radius = 1.5];
    std::print($r.radius, " ", shapes.circle.$pi, newln);
;MAIN
