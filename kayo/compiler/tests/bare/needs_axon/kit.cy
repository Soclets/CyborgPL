# A growing list needs Axon, the runtime, which is not built for a chip
# with no OS: refused before linking, naming what needs it.
MAIN {
    let mut 'xs': list[i32] = [1, 2];
    list.append[lend.mut 'xs', value: 3];
}
