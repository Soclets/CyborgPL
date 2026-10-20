extern.C fn exit ['code': i32];
MAIN {
    let 'c' := channel[i32];
    let mut 'got': i32 = 0;
    {
        concur with [lend 'c'] { channel.send[lend 'c', value 5]; }
        concur with [lend 'c', lend.mut 'got'] {
            let 'x' := channel.take[lend 'c'] otherwise 0;
            let 'y' := channel.take[lend 'c'] otherwise 100;
            change 'got' = 'x' plus 'y';
        }
    }
    no_check my.exit['got'];
}
