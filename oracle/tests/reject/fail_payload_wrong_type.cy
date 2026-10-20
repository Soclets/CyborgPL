fn f [] -> fail[why: i32] { fail why["x"]; }
MAIN {
    f[] otherwise { why 'w' { } };
}
