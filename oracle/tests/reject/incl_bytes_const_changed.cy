MAIN {
    const 'b' := incl.bytes["incl_assets/cafe.txt"];
    change 'b'[pos: 1] = 0;
}
