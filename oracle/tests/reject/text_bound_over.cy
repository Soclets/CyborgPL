MAIN {
    let mut 'line': str.utf8[min 0, max 20 bytes] = "twelve chars";
    change 'line' = 'line' concat " and more";     # 21 bytes
}
