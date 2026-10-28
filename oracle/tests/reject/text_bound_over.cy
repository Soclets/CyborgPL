MAIN:
    var $line: str.utf8[min: 0, max: 20 bytes] = "twelve chars";
    change $line ++= " and more";     # 21 bytes
;MAIN
