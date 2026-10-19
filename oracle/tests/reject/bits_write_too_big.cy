layout.C struct S { mode: uint8[bits 3] }

MAIN {
    let mut 's' := S[mode = 1];
    change 's'.mode = 8;
}
