pub const $pi: f64 := 3.141_592_653_589_793;
pub rec Circle:
    var radius: f64;
;rec Circle
pub fn area [$r: f64] -> f64:
    ret $pi x $r x $r;
;fn area
pub.kit fn name [] -> str.utf8:
    ret "circle";
;fn name
