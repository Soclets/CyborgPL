MAIN {
    let 'n' := to.i32[min 0, max 100]["42"] otherwise { not_a_number { std::exit[1]; } };
}
