extern.C fn exit ['code': i32];
fn fetch [lent 'url': str.utf8] -> str.utf8 { return "page of " concat 'url'; }
MAIN {
    let 'results' := channel[str.utf8];
    let 'a' := "a.com";
    concur with [lend 'results', give 'a'] { channel.send[lend 'results', value fetch[lend 'a']]; }
    concur with [lend 'results'] { channel.send[lend 'results', value fetch["b.com"]]; }
    let mut 'total': i32 = 0;
    for 'i' from 1 to 2 by 1 {
        let 'page' := channel.take[lend 'results'] otherwise str.empty;
        change 'total' = 'total' plus str.bytes[lend 'page'];
    }
    let 'extra' := channel.take[lend 'results'] otherwise "closed!";
    no_check my.exit['total' plus str.bytes[lend 'extra']];
}
