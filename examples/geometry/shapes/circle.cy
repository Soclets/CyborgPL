pub const $pi: float.bin.64 = 3.141_592_653_589_793;
pub rec Circle:
    var radius: float.bin.64;
end of rec Circle
pub fn area [let $r: float.bin.64] -> float.bin.64:
    ret $pi times $r times $r;
end of fn area
pub.kit fn name [] -> str.utf8:
    ret "circle";
end of fn name
