# `runnable` has no `mean`: a path picks only that kit's operator
MAIN {
    let 'x': f64 = 3.0;
    let 'm' := 'x' runnable::mean 'x';
}
