# In between, the limit is the machine's 64 bits (2026-09-23)
MAIN {
    let 'a': i64 = 9000000000000000000;
    let 'p' := ('a' times 'a') div.floor 'a';
}
