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

module board {
    # The serial port is a PL011 at 0x0900_0000: a byte written to its data register goes out.
    pub.kit fn write [$b: u8] {
        no_check inline.ASM {
            mov x9, #0x09000000
            strb w0, [x9]
        } with [$b -> x0, clobber x9];
    }

    # Power off through PSCI's SYSTEM_OFF (0x8400_0008), over `hvc`; QEMU ends with it.
    pub.kit fn off [$code: u8] {
        no_check inline.ASM {
            movz x0, #0x0008
            movk x0, #0x8400, lsl #16
            hvc #0
        } with [$code -> x1, clobber x0];
    }
}

# ---------------------------------------------------------------------------------------------------
# The log, baked in, and the bytes it is read by
# ---------------------------------------------------------------------------------------------------

const $log := incl.bytes["data/log.txt"];

const $newline: u8 = 10;
const $space: u8 = 32;
const $hash: u8 = 35;
const $minus: u8 = 45;
const $zero: u8 = 48;
const $nine: u8 = 57;

# A temperature in tenths of a degree Celsius, as far as the sensor reads: -50.0 to 60.0.
coll.bound temp {
    temp = i32[min: -500, max: 600];
}

# A day is 24 hourly readings, so every count and sum of one has a bound the prover can see.
const $hours: i32 = 24;
coll.bound day {
    day.hour = i32[min: 0, max: 23];
    day.count = i32[min: 0, max: 24];
    day.sum = i32[min: -12000, max: 14400];
    day.step = i32[min: -1100, max: 1100];
    day.tenths = i32[min: -2000, max: 2000];
}

# Where the reading of the log has got to: a position in it, from 1 (one past the end when it is done).
struct Cursor {
    at: u64[min: 1]
}

# The next reading, past comment lines (`#` to the line's end) and blank ones; `done` at the log's end, and
# `bad_byte` with its position for a byte no reading has, or a number past what the sensor reads.
fn next_reading [lent.mut $c: Cursor] -> temp or fail[done, bad_byte: u64] {
    let $n := array.length($log);
    let $at := $c.at;
    let $first: u8 = 0;
    loop {
        if $at > $n { fail done; }
        let $b := $log[pos: $at];
        if $b equal_to $hash {
            while $at at_most $n and.then $log[pos: $at] not_equal $newline { change $at = $at + 1; }
            continue;
        }
        if $b equal_to $newline or $b equal_to $space { change $at = $at + 1; continue; }
        change $first = $b;
        break;
    }
    let $negative := $first equal_to $minus;
    if $negative { change $at = $at + 1; }
    let $value: i32 = 0;
    while $at at_most $n and.then $log[pos: $at] not_equal $newline {
        let $b := $log[pos: $at];
        if $b < $zero { fail bad_byte[$at]; }
        if $b > $nine { fail bad_byte[$at]; }
        if $value > 600 { fail bad_byte[$at]; }
        change $value = $value x 10 + to.i32($b - $zero);
        change $at = $at + 1;
    }
    change $c.at = $at;
    if $negative { change $value = -$value; }
    if $value < -500 { fail bad_byte[$at]; }
    if $value > 600 { fail bad_byte[$at]; }
    let $t: temp = $value;
    return $t;
}

# Whether the log has nothing left to read.
fn log_ends [lent.mut $c: Cursor] -> bool {
    let $next := next_reading(lend.mut $c) otherwise {
        done { return true; }
        bad_byte $at { return false; }
    };
    return false;
}

# The CRC-32 of the whole log (the one zlib and PNG use, reflected, polynomial 0xEDB8_8320), bit by bit: no table,
# so no memory beyond the program's own.
fn crc32 [] -> u32 {
    let $crc: u32 = hex FFFF_FFFF;
    for $i [from: 1, to: array.length($log), by: 1] {
        change $crc = $crc xor to.u32($log[pos: $i]);
        for $k [from: 1, to: 8, by: 1] {
            if ($crc and.bit 1) equal_to 1 {
                change $crc = ($crc shr 1) xor hex EDB8_8320;
            } else {
                change $crc = $crc shr 1;
            }
        }
    }
    return $crc xor hex FFFF_FFFF;
}

# ---------------------------------------------------------------------------------------------------
# Printing with no heap: every figure goes out as numbers and literal text, never as built text
# ---------------------------------------------------------------------------------------------------

# Tenths of a degree as degrees: `-3.5`, `21.0`.
fn show_tenths [$t: day.tenths] {
    let $a := $t;
    if $t < 0 {
        std::print.useful("-");
        change $a = -$t;
    }
    std::print.useful($a div.floor 10, ".", $a mod.floor 10);
}

# An hour of the day as a clock time, `07:00`.
fn show_hour [$h: day.hour] {
    if $h < 10 { std::print.useful("0"); }
    std::print.useful($h, ":00");
}

fn show_digit [$d: u32] {
    if $d < 10 { std::print.useful($d); return; }
    if $d equal_to 10 { std::print.useful("a"); return; }
    if $d equal_to 11 { std::print.useful("b"); return; }
    if $d equal_to 12 { std::print.useful("c"); return; }
    if $d equal_to 13 { std::print.useful("d"); return; }
    if $d equal_to 14 { std::print.useful("e"); return; }
    std::print.useful("f");
}

# A 32-bit word in hex, its eight digits from the top: `0x0018_0803`.
fn show_hex [$x: u32] {
    std::print.useful("0x");
    for $k [from: 1, to: 8, by: 1] {
        let $shift := to.u32(32 - 4 x $k);
        show_digit(($x shr $shift) and.bit 15);
        if $k equal_to 4 { std::print.useful("_"); }
    }
}

# ---------------------------------------------------------------------------------------------------
# The alarm: calm, warm over 22.0, hot over 25.0
# ---------------------------------------------------------------------------------------------------

enum Alarm {
    calm,
    warm,
    hot
}

fn alarm_for [$t: temp] -> Alarm {
    if $t > 250 { return Alarm.hot; }
    if $t > 220 { return Alarm.warm; }
    return Alarm.calm;
}

fn rank [$a: Alarm] -> i32 {
    let $r := match $a {
        calm { is 0; }
        warm { is 1; }
        hot { is 2; }
    };
    return $r;
}

fn show_alarm [$a: Alarm] {
    match $a {
        calm { std::print.useful("calm"); }
        warm { std::print.useful("warm"); }
        hot { std::print.useful("hot"); }
    }
}

# ---------------------------------------------------------------------------------------------------
# The day
# ---------------------------------------------------------------------------------------------------

interface Number {
    i32
    u32
}

fn larger <T: Number> [$a: <T>, $b: <T>] -> <T> {
    if $a > $b { return $a; }
    return $b;
}

# Everything the day comes to, worked out in one pass over its 24 readings.
struct Day {
    sum: day.sum,
    low: temp,
    low_hour: day.hour,
    high: temp,
    high_hour: day.hour,
    rise: day.step,
    rise_hour: day.hour,
    warm_hours: day.count,
    hot_hours: day.count,
    changes: day.count
}

fn read_day [] -> Day {
    let mut $c := Cursor[at = 1];
    let $sum: day.sum = 0;
    let $low: temp = 600;
    let $low_hour: day.hour = 0;
    let $high: temp = -500;
    let $high_hour: day.hour = 0;
    let $rise: day.step = 0;
    let $rise_hour: day.hour = 1;
    let $warm: day.count = 0;
    let $hot: day.count = 0;
    let $changes: day.count = 0;
    let $prev: temp = 0;
    let $alarm := Alarm.calm;
    std::print.useful("alarm:  ");
    show_alarm(copy $alarm);
    for $hour [from: 0, before: $hours, by: 1] {
        let $t := next_reading(lend.mut $c) otherwise {
            done {
                std::print.useful(newln, "the log ends after ", $hour, " readings of a day's ", $hours, "; stopping", newln);
                std::exit(code: 2);
            }
            bad_byte $at {
                std::print.useful(newln, "the log has a byte no reading has, at ", $at, "; stopping", newln);
                std::exit(code: 2);
            }
        };
        if $t < $low { change $low = $t; change $low_hour = $hour; }
        if $t > $high { change $high = $t; change $high_hour = $hour; }
        if $hour > 0 {
            let $step := $t - $prev;
            if larger($step, -$step) > larger($rise, -$rise) { change $rise = $step; change $rise_hour = $hour; }
        }
        let $now := alarm_for($t);
        if rank(copy $now) at_least 1 { change $warm = $warm + 1; }
        if rank(copy $now) equal_to 2 { change $hot = $hot + 1; }
        if rank(copy $now) not_equal rank(copy $alarm) {
            std::print.useful(" -> ");
            show_alarm(copy $now);
            std::print.useful(" at ");
            show_hour($hour);
            change $changes = $changes + 1;
            change $alarm = copy $now;
        }
        change $sum = $sum + $t;
        change $prev = $t;
    }
    std::print.useful(newln);
    # a day is exactly its readings: one more would be another day's
    if not log_ends(lend.mut $c) {
        std::print.useful("the log holds more than a day's ", $hours, " readings; stopping", newln);
        std::exit(code: 2);
    }
    return Day[sum = $sum, low = $low, low_hour = $low_hour, high = $high, high_hour = $high_hour, rise = $rise, rise_hour = $rise_hour, warm_hours = $warm, hot_hours = $hot, changes = $changes];
}

# A status word for a control panel, four bytes from the top: readings, warm hours, hot hours, alarm changes.
fn status_word [lent $d: Day] -> u32 {
    let $w := to.u32($hours) shl 24;
    change $w = $w or.bit (to.u32($d.warm_hours) shl 16);
    change $w = $w or.bit (to.u32($d.hot_hours) shl 8);
    change $w = $w or.bit to.u32($d.changes);
    return $w;
}

fn report [lent $d: Day] {
    std::print.useful("readings: ", $hours, ", in tenths of a degree", newln);
    std::print.useful("lowest:  ");
    show_tenths($d.low);
    std::print.useful(" C at ");
    show_hour($d.low_hour);
    std::print.useful(newln, "highest: ");
    show_tenths($d.high);
    std::print.useful(" C at ");
    show_hour($d.high_hour);
    let $mean := $d.sum div.near $hours;
    std::print.useful(newln, "mean:    ");
    show_tenths($mean);
    std::print.useful(" C, ");
    show_tenths(($mean x 9) div.near 5 + 320);
    std::print.useful(" F", newln, "steepest hour: ");
    if $d.rise > 0 { std::print.useful("+"); }
    show_tenths($d.rise);
    std::print.useful(" C, from ");
    show_hour($d.rise_hour - 1);
    std::print.useful(" to ");
    show_hour($d.rise_hour);
    std::print.useful(newln, "warm for ", $d.warm_hours, " hours, hot for ", $d.hot_hours, newln);
}

MAIN {
    std::print.useful("station 7, booting on a chip with no operating system", newln);
    std::print.useful("log: ", array.length($log), " bytes, CRC-32 ");
    show_hex(crc32());
    std::print.useful(newln);
    let $day := read_day();
    report(lend $day);
    std::print.useful("status word: ");
    show_hex(status_word(lend $day));
    std::print.useful(newln, "switching off", newln);
    std::exit(code: 0);
}
