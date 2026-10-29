# A weather station's firmware, on a chip with no operating system (`aarch64-none`, owner 2026-10-09).
#
# There is nothing under this program: no OS, no files, no heap. Its day of readings is baked into it at build time
# (`incl.bytes`); it reads them byte by byte, checks the bytes with a CRC-32, works out the day's figures in whole
# tenths of a degree, runs an alarm through the day, packs a status word for a control panel, and talks over the
# board's serial port. When it is done it switches the chip off. The board's two jobs, a byte out of the serial port
# and power off, are this kit's own `board` module, which the target names in CyJet.toml as `std::print`'s output and
# `std::exit`'s way out. Build and run it:
#
#   kayo build examples/bare_station -o build/bare_station
#   qemu-system-aarch64 -M virt -cpu cortex-a72 -display none -serial stdio -kernel build/bare_station
#
# Along the way: structs and enums, failures with `otherwise`, a named bound (`coll.bound temp`), generics, bits
# (`xor`, `and.bit`, `or.bit`, `shr`, `shl`), inline assembly for the hardware, and nothing that needs a heap.

# ---------------------------------------------------------------------------------------------------
# The board: QEMU's `virt` machine
# ---------------------------------------------------------------------------------------------------

module board:
    # The serial port is a PL011 at 0x0900_0000: a byte written to its data register goes out.
    pub.kit proc write [$b: u8]:
        no_check inline.ASM with [$b -> x0, clobber x9]:
            mov x9, #0x09000000
            strb w0, [x9]
        ;inline.ASM
    ;proc write

    # Power off through PSCI's SYSTEM_OFF (0x8400_0008), over `hvc`; QEMU ends with it.
    pub.kit proc off [$code: u8]:
        no_check inline.ASM with [$code -> x1, clobber x0]:
            movz x0, #0x0008
            movk x0, #0x8400, lsl #16
            hvc #0
        ;inline.ASM
    ;proc off
;module board

# ---------------------------------------------------------------------------------------------------
# The log, baked in, and the bytes it is read by
# ---------------------------------------------------------------------------------------------------

const $log := incl.bytes["data/log.txt"];

const $newline: u8 := 10;
const $space: u8 := 32;
const $hash: u8 := 35;
const $minus: u8 := 45;
const $zero: u8 := 48;
const $nine: u8 := 57;

# A temperature in tenths of a degree Celsius, as far as the sensor reads: -50.0 to 60.0.
coll.bounded temp:
    temp = i32[min: -500, max: 600];
;coll.bounded temp

# A day is 24 hourly readings, so every count and sum of one has a bound the prover can see.
const $hours: i32 := 24;
coll.bounded day:
    day.hour = i32[min: 0, max: 23];
    day.count = i32[min: 0, max: 24];
    day.sum = i32[min: -12000, max: 14400];
    day.step = i32[min: -1100, max: 1100];
    day.tenths = i32[min: -2000, max: 2000];
;coll.bounded day

# Where the reading of the log has got to: a position in it, from 1 (one past the end when it is done).
rec Cursor:
    var at: u64[min: 1];
;rec Cursor

# The next reading, past comment lines (`#` to the line's end) and blank ones; `done` at the log's end, and
# `bad_byte` with its position for a byte no reading has, or a number past what the sensor reads.
proc next_reading [lent.mut $c: Cursor] -> temp or fail[done, bad_byte: u64]:
    let $n := arr.len($log);
    var $at = $c.at;
    var $first: u8 = 0;
    loop:
        if $at > $n:
            fail done;
        ;if
        let $b := $log[pos: $at];
        if $b == $hash:
            while $at <= $n and.then $log[pos: $at] != $newline:
                change $at to $at + 1;
            ;while
            continue;
        ;if
        if $b == $newline or $b == $space:
            change $at to $at + 1; continue;
        ;if
        change $first to $b;
        break;
    ;loop
    let $negative := $first == $minus;
    if $negative:
        change $at to $at + 1;
    ;if
    var $value: i32 = 0;
    while $at <= $n and.then $log[pos: $at] != $newline:
        let $b := $log[pos: $at];
        if $b < $zero:
            fail bad_byte[$at];
        ;if
        if $b > $nine:
            fail bad_byte[$at];
        ;if
        if $value > 600:
            fail bad_byte[$at];
        ;if
        change $value to $value x 10 + to.i32($b - $zero);
        change $at to $at + 1;
    ;while
    change $c.at to $at;
    if $negative:
        change $value to -$value;
    ;if
    if $value < -500:
        fail bad_byte[$at];
    ;if
    if $value > 600:
        fail bad_byte[$at];
    ;if
    let $t: temp := $value;
    ret $t;
;proc next_reading

# Whether the log has nothing left to read.
proc log_ends [lent.mut $c: Cursor] -> bool:
    let $next := next_reading(lend.mut $c)-
    otherwise:
        case done:
            ret true;
        ;case done
        case bad_byte $at:
            ret false;
        ;case bad_byte
    ;otherwise;
    ret false;
;proc log_ends

# The CRC-32 of the whole log (the one zlib and PNG use, reflected, polynomial 0xEDB8_8320), bit by bit: no table,
# so no memory beyond the program's own.
fn crc32 [] -> u32:
    var $crc: u32 = hex FFFF_FFFF;
    for $i [from: 1, to: arr.len($log), by: 1]:
        change $crc to $crc xor to.u32($log[pos: $i]);
        for $k [from: 1, to: 8, by: 1]:
            if ($crc and.bit 1) == 1:
                change $crc to ($crc shr 1) xor hex EDB8_8320;
            else:
                change $crc to $crc shr 1;
            ;if
        ;for
    ;for
    ret $crc xor hex FFFF_FFFF;
;fn crc32

# ---------------------------------------------------------------------------------------------------
# Printing with no heap: every figure goes out as numbers and literal text, never as built text
# ---------------------------------------------------------------------------------------------------

# Tenths of a degree as degrees: `-3.5`, `21.0`.
proc show_tenths [$t: day.tenths]:
    var $a = $t;
    if $t < 0:
        std::print("-");
        change $a to -$t;
    ;if
    std::print($a div.floor 10, ".", $a mod.floor 10);
;proc show_tenths

# An hour of the day as a clock time, `07:00`.
proc show_hour [$h: day.hour]:
    if $h < 10:
        std::print("0");
    ;if
    std::print($h, ":00");
;proc show_hour

proc show_digit [$d: u32]:
    if $d < 10:
        std::print($d); ret;
    ;if
    if $d == 10:
        std::print("a"); ret;
    ;if
    if $d == 11:
        std::print("b"); ret;
    ;if
    if $d == 12:
        std::print("c"); ret;
    ;if
    if $d == 13:
        std::print("d"); ret;
    ;if
    if $d == 14:
        std::print("e"); ret;
    ;if
    std::print("f");
;proc show_digit

# A 32-bit word in hex, its eight digits from the top: `0x0018_0803`.
proc show_hex [$x: u32]:
    std::print("0x");
    for $k [from: 1, to: 8, by: 1]:
        let $shift := to.u32(32 - 4 x $k);
        show_digit(($x shr $shift) and.bit 15);
        if $k == 4:
            std::print("_");
        ;if
    ;for
;proc show_hex

# ---------------------------------------------------------------------------------------------------
# The alarm: calm, warm over 22.0, hot over 25.0
# ---------------------------------------------------------------------------------------------------

union.tag Alarm:
    calm;
    warm;
    hot;
;union.tag Alarm

fn alarm_for [$t: temp] -> Alarm:
    if $t > 250:
        ret Alarm.hot;
    ;if
    if $t > 220:
        ret Alarm.warm;
    ;if
    ret Alarm.calm;
;fn alarm_for

fn rank [$a: Alarm] -> i32:
    let $r := match $a:
        case calm:
            is 0;
        ;case calm
        case warm:
            is 1;
        ;case warm
        case hot:
            is 2;
        ;case hot
    ;match;
    ret $r;
;fn rank

proc show_alarm [$a: Alarm]:
    match $a:
        case calm:
            std::print("calm");
        ;case calm
        case warm:
            std::print("warm");
        ;case warm
        case hot:
            std::print("hot");
        ;case hot
    ;match
;proc show_alarm

# ---------------------------------------------------------------------------------------------------
# The day
# ---------------------------------------------------------------------------------------------------

set Number:
    i32;
    u32;
;set Number

sig Ordered [<T>]:
    op < [lent $a: <T>, lent $b: <T>] -> bool;
    op > [lent $a: <T>, lent $b: <T>] -> bool;
    op <= [lent $a: <T>, lent $b: <T>] -> bool;
    op >= [lent $a: <T>, lent $b: <T>] -> bool;
    op == [lent $a: <T>, lent $b: <T>] -> bool;
;sig Ordered

fn larger [<T>: Number and Ordered] [$a: <T>, $b: <T>] -> <T>:
    if $a > $b:
        ret $a;
    ;if
    ret $b;
;fn larger

# Everything the day comes to, worked out in one pass over its 24 readings.
rec Day:
    var sum: day.sum;
    var low: temp;
    var low_hour: day.hour;
    var high: temp;
    var high_hour: day.hour;
    var rise: day.step;
    var rise_hour: day.hour;
    var warm_hours: day.count;
    var hot_hours: day.count;
    var changes: day.count;
;rec Day

proc read_day [] -> Day:
    var $c = Cursor[at = 1];
    var $sum: day.sum = 0;
    var $low: temp = 600;
    var $low_hour: day.hour = 0;
    var $high: temp = -500;
    var $high_hour: day.hour = 0;
    var $rise: day.step = 0;
    var $rise_hour: day.hour = 1;
    var $warm: day.count = 0;
    var $hot: day.count = 0;
    var $changes: day.count = 0;
    var $prev: temp = 0;
    var $alarm = Alarm.calm;
    std::print("alarm:  ");
    show_alarm($alarm);
    for $hour [from: 0, before: $hours, by: 1]:
        let $t := next_reading(lend.mut $c)-
        otherwise:
            case done:
                std::print(newln, "the log ends after ", $hour, " readings of a day's ", $hours, "; stopping", newln);
                std::exit(code: 2);
            ;case done
            case bad_byte $at:
                std::print(newln, "the log has a byte no reading has, at ", $at, "; stopping", newln);
                std::exit(code: 2);
            ;case bad_byte
        ;otherwise;
        if $t < $low:
            change $low to $t; change $low_hour to $hour;
        ;if
        if $t > $high:
            change $high to $t; change $high_hour to $hour;
        ;if
        if $hour > 0:
            let $step := $t - $prev;
            if larger($step, -$step) > larger($rise, -$rise):
                change $rise to $step; change $rise_hour to $hour;
            ;if
        ;if
        let $now := alarm_for($t);
        if rank($now) >= 1:
            change $warm to $warm + 1;
        ;if
        if rank($now) == 2:
            change $hot to $hot + 1;
        ;if
        if rank($now) != rank($alarm):
            std::print(" -> ");
            show_alarm($now);
            std::print(" at ");
            show_hour($hour);
            change $changes to $changes + 1;
            change $alarm to $now;
        ;if
        change $sum to $sum + $t;
        change $prev to $t;
    ;for
    std::print(newln);
    # a day is exactly its readings: one more would be another day's
    if not log_ends(lend.mut $c):
        std::print("the log holds more than a day's ", $hours, " readings; stopping", newln);
        std::exit(code: 2);
    ;if
    ret Day[sum = $sum, low = $low, low_hour = $low_hour, high = $high, high_hour = $high_hour, rise = $rise, rise_hour = $rise_hour, warm_hours = $warm, hot_hours = $hot, changes = $changes];
;proc read_day

# A status word for a control panel, four bytes from the top: readings, warm hours, hot hours, alarm changes.
fn status_word [lent $d: Day] -> u32:
    var $w = to.u32($hours) shl 24;
    change $w to $w or.bit (to.u32($d.warm_hours) shl 16);
    change $w to $w or.bit (to.u32($d.hot_hours) shl 8);
    change $w to $w or.bit to.u32($d.changes);
    ret $w;
;fn status_word

proc report [lent $d: Day]:
    std::print("readings: ", $hours, ", in tenths of a degree", newln);
    std::print("lowest:  ");
    show_tenths($d.low);
    std::print(" C at ");
    show_hour($d.low_hour);
    std::print(newln, "highest: ");
    show_tenths($d.high);
    std::print(" C at ");
    show_hour($d.high_hour);
    let $mean := $d.sum div.near $hours;
    std::print(newln, "mean:    ");
    show_tenths($mean);
    std::print(" C, ");
    show_tenths(($mean x 9) div.near 5 + 320);
    std::print(" F", newln, "steepest hour: ");
    if $d.rise > 0:
        std::print("+");
    ;if
    show_tenths($d.rise);
    std::print(" C, from ");
    show_hour($d.rise_hour - 1);
    std::print(" to ");
    show_hour($d.rise_hour);
    std::print(newln, "warm for ", $d.warm_hours, " hours, hot for ", $d.hot_hours, newln);
;proc report

MAIN:
    std::print("station 7, booting on a chip with no operating system", newln);
    std::print("log: ", arr.len($log), " bytes, CRC-32 ");
    show_hex(crc32());
    std::print(newln);
    let $day := read_day();
    report(lend $day);
    std::print("status word: ");
    show_hex(status_word(lend $day));
    std::print(newln, "switching off", newln);
    std::exit(code: 0);
;MAIN
