# A growing list needs Axon, the runtime, which is not built for a chip
# with no OS: refused before linking, naming what needs it.
MAIN {
    let mut 'xs': std::list[i32] = [1, 2];
    std::list.append[lend.mut 'xs', value: 3];
}
