MAIN:
    let $big := to.%bf16("3e38") ?? 0;
    let $more := $big x 10;
;MAIN
