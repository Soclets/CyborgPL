# A f16/f32 holds what its width holds: a value is rounded to it where it lands
# (DECISIONS 2026-09-14). `print` shows what is stored, `print.useful` the shortest text that
# gives the value back at its own width.
MAIN {
    let 'f': f32 = 0.1;
    let 'h': f16 = 0.1;
    let 'd': f64 = 0.1;
    std::print.stored['f', newln];     # 0.100000001490116119384765625
    std::print.stored['h', newln];     # 0.0999755859375
    std::print.stored['d', newln];
    std::print.useful['f', " ", 'h', " ", 'd', newln];     # all three: 0.1
    let 'big': f16 = 65519.0;                          # rounds down to f16's largest, 65504
    let 'g' := to.f32["0.1"] otherwise 0.0;            # a conversion lands at f32 too
    std::print.stored['big', " ", 'g', newln];
    # f32 math rounds after every step, as the hardware does (owner, 2026-09-24: a):
    # 16777216 plus 1 rounds back to 16777216 at f32, while f64 holds 16777218
    let 'x': f32 = 16777216.0;
    let 'y': f32 = 'x' + 1.0 + 1.0;
    let 'x64': f64 = 16777216.0;
    let 'y64': f64 = 'x64' + 1.0 + 1.0;
    std::print.useful['y', " ", 'y64', newln];
    # a literal is rounded once, from its digits, straight to its width (never to 64 bits first)
    let 'top': f32 = 340282356779733661637539395458142568447.0;   # just under the limit: f32's largest
    let 'near': f32 = 1.00000005960464477539062500001;            # just past a halfway point: rounds up
    std::print.stored['top', " ", 'near', newln];
    # text is rounded once from its digits too
    let 't16' := to.f16["65519.99999999999999999"] otherwise -1.0;    # just under the limit: 65504
    let 't32' := to.f32["1.00000005960464477539062500001"] otherwise -1.0;
    let 'tiny' := to.f16["2.98023223876953126e-8"] otherwise -1.0;   # just past half the smallest step
    std::print.stored['t16', " ", 't32', " ", 'tiny', newln];
}
