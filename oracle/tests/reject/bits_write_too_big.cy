layout.C struct S { mode: u8[bits: 3] }

MAIN {
    let mut 's' := S[mode = 1];
    change 's'.mode = 8;
}
