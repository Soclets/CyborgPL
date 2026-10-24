# a returned borrow cannot be of something that dies at the return (2026-09-29)
fn local [] -> lent str.utf8 { let $x := "tmp"; return lend $x; }
MAIN { let $r := local(); }
