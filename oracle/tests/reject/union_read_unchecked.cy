layout.C union Number { i: int32, f: bin32 }

MAIN {
    let 'n' := Number[i = 1];
    let 'x' := 'n'.i;
}
