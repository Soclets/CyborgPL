# The root of the kit: its items are `geometry::…` from outside.

bare.never [fn, struct];

const 'unit' := 1.0;

fn helper [] -> str.utf8 { return "from the root"; }

module report {
    pub.kit fn line ['label': str.utf8, 'v': f64] {
        std::print.useful['label', ": ", 'v', newln];
    }
}

MAIN {
    report.line["circle", shapes.circle.area[2.0]];
    report.line["square", shapes.square.area[3.0]];
    report.line["unit circle", shapes.circle.area['unit']];
    std::print.useful[shapes.describe[], newln];
    let 'r': shapes.circle.Circle = shapes.circle.Circle[radius = 1.5];
    std::print.useful['r'.radius, " ", shapes.circle.'pi', newln];
}
