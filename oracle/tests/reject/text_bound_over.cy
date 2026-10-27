!MAIN:
    variable $line: string.utf8[minimum: 0, maximum: 20 bytes] = "twelve chars";
    change $line = $line concatenate " and more";     # 21 bytes
!end of MAIN
