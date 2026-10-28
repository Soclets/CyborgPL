rec B:
    let vx: f64;
;rec B
MAIN:
    var $bs = [B[vx = 1.0], B[vx = 2.0]];
    change $bs[pos: 1].vx to 2.0;
;MAIN
