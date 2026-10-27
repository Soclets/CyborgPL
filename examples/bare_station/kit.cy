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
    pub.kit proc write [let $b: int.unsign.8]:
        !no_check inline.ASM with [$b -> x0, clobber x9]:
            mov x9, #0x09000000
            strb w0, [x9]
        end of inline.ASM
    end of proc write

    # Power off through PSCI's SYSTEM_OFF (0x8400_0008), over `hvc`; QEMU ends with it.
    pub.kit proc off [let $code: int.unsign.8]:
        !no_check inline.ASM with [$code -> x1, clobber x0]:
            movz x0, #0x0008
            movk x0, #0x8400, lsl #16
            hvc #0
        end of inline.ASM
    end of proc off
end of module board

# ---------------------------------------------------------------------------------------------------
# The log, baked in, and the bytes it is read by
# ---------------------------------------------------------------------------------------------------

const $log := incl.bytes["data/log.txt"];

const $newline: int.unsign.8 := 10;
const $space: int.unsign.8 := 32;
const $hash: int.unsign.8 := 35;
const $minus: int.unsign.8 := 45;
const $zero: int.unsign.8 := 48;
const $nine: int.unsign.8 := 57;

# A temperature in tenths of a degree Celsius, as far as the sensor reads: -50.0 to 60.0.
coll.bounded temp:
    temp = int.sign.32[min: neg 500, max: 600];
end of coll.bounded temp

# A day is 24 hourly readings, so every count and sum of one has a bound the prover can see.
const $hours: int.sign.32 := 24;
coll.bounded day:
    day.hour = int.sign.32[min: 0, max: 23];
    day.count = int.sign.32[min: 0, max: 24];
    day.sum = int.sign.32[min: neg 12000, max: 14400];
    day.step = int.sign.32[min: neg 1100, max: 1100];
    day.tenths = int.sign.32[min: neg 2000, max: 2000];
end of coll.bounded day

# Where the reading of the log has got to: a position in it, from 1 (one past the end when it is done).
rec Cursor:
    var at: int.unsign.64[min: 1];
end of rec Cursor

# The next reading, past comment lines (`#` to the line's end) and blank ones; `done` at the log's end, and
# `bad_byte` with its position for a byte no reading has, or a number past what the sensor reads.
proc next_reading [lent.mut $c: Cursor] -> temp or fail[done, bad_byte: int.unsign.64]:
    let $n := arr.len($log);
    var $at = $c.at;
    var $first: int.unsign.8 = 0;
    loop:
        if $at gt $n:
            fail done;
        end of if
        let $b := $log[pos: $at];
        if $b eq $hash:
            while $at at_most $n and.then $log[pos: $at] not_eq $newline:
                change $at to $at plus 1;
            end of while
            continue;
        end of if
        if $b eq $newline or $b eq $space:
            change $at to $at plus 1; continue;
        end of if
        change $first to $b;
        break;
    end of loop
    let $negative := $first eq $minus;
    if $negative:
        change $at to $at plus 1;
    end of if
    var $value: int.sign.32 = 0;
    while $at at_most $n and.then $log[pos: $at] not_eq $newline:
        let $b := $log[pos: $at];
        if $b lt $zero:
            fail bad_byte[$at];
        end of if
        if $b gt $nine:
            fail bad_byte[$at];
        end of if
        if $value gt 600:
            fail bad_byte[$at];
        end of if
        change $value to $value times 10 plus to.int.sign.32($b minus $zero);
        change $at to $at plus 1;
    end of while
    change $c.at to $at;
    if $negative:
        change $value to neg $value;
    end of if
    if $value lt neg 500:
        fail bad_byte[$at];
    end of if
    if $value gt 600:
        fail bad_byte[$at];
    end of if
    let $t: temp := $value;
    ret $t;
end of proc next_reading

# Whether the log has nothing left to read.
proc log_ends [lent.mut $c: Cursor] -> bool:
    let $next := next_reading(lend.mut $c)-
    otherwise:
        !case done:
            ret true;
        !end of case done
        !case bad_byte $at:
            ret false;
        !end of case bad_byte
    end of otherwise;
    ret false;
end of proc log_ends

# The CRC-32 of the whole log (the one zlib and PNG use, reflected, polynomial 0xEDB8_8320), bit by bit: no table,
# so no memory beyond the program's own.
fn crc32 [] -> int.unsign.32:
    var $crc: int.unsign.32 = hex FFFF_FFFF;
    for $i [from: 1, to: arr.len($log), by: 1]:
        change $crc to $crc xor to.int.unsign.32($log[pos: $i]);
        for $k [from: 1, to: 8, by: 1]:
            if ($crc and.bit 1) eq 1:
                change $crc to ($crc shr 1) xor hex EDB8_8320;
            end of if
            else:
                change $crc to $crc shr 1;
            end of else
        end of for
    end of for
    ret $crc xor hex FFFF_FFFF;
end of fn crc32

# ---------------------------------------------------------------------------------------------------
# Printing with no heap: every figure goes out as numbers and literal text, never as built text
# ---------------------------------------------------------------------------------------------------

# Tenths of a degree as degrees: `-3.5`, `21.0`.
proc show_tenths [let $t: day.tenths]:
    var $a = $t;
    if $t lt 0:
        std::print.shortest("-");
        change $a to neg $t;
    end of if
    std::print.shortest($a div.floor 10, ".", $a mod.floor 10);
end of proc show_tenths

# An hour of the day as a clock time, `07:00`.
proc show_hour [let $h: day.hour]:
    if $h lt 10:
        std::print.shortest("0");
    end of if
    std::print.shortest($h, ":00");
end of proc show_hour

proc show_digit [let $d: int.unsign.32]:
    if $d lt 10:
        std::print.shortest($d); ret;
    end of if
    if $d eq 10:
        std::print.shortest("a"); ret;
    end of if
    if $d eq 11:
        std::print.shortest("b"); ret;
    end of if
    if $d eq 12:
        std::print.shortest("c"); ret;
    end of if
    if $d eq 13:
        std::print.shortest("d"); ret;
    end of if
    if $d eq 14:
        std::print.shortest("e"); ret;
    end of if
    std::print.shortest("f");
end of proc show_digit

# A 32-bit word in hex, its eight digits from the top: `0x0018_0803`.
proc show_hex [let $x: int.unsign.32]:
    std::print.shortest("0x");
    for $k [from: 1, to: 8, by: 1]:
        let $shift := to.int.unsign.32(32 minus 4 times $k);
        show_digit(($x shr $shift) and.bit 15);
        if $k eq 4:
            std::print.shortest("_");
        end of if
    end of for
end of proc show_hex

# ---------------------------------------------------------------------------------------------------
# The alarm: calm, warm over 22.0, hot over 25.0
# ---------------------------------------------------------------------------------------------------

union.tag Alarm:
    calm;
    warm;
    hot;
end of union.tag Alarm

fn alarm_for [let $t: temp] -> Alarm:
    if $t gt 250:
        ret Alarm.hot;
    end of if
    if $t gt 220:
        ret Alarm.warm;
    end of if
    ret Alarm.calm;
end of fn alarm_for

fn rank [let $a: Alarm] -> int.sign.32:
    let $r := match $a:
        case calm:
            is 0;
        end of case calm
        case warm:
            is 1;
        end of case warm
        case hot:
            is 2;
        end of case hot
    end of match;
    ret $r;
end of fn rank

proc show_alarm [let $a: Alarm]:
    match $a:
        case calm:
            std::print.shortest("calm");
        end of case calm
        case warm:
            std::print.shortest("warm");
        end of case warm
        case hot:
            std::print.shortest("hot");
        end of case hot
    end of match
end of proc show_alarm

# ---------------------------------------------------------------------------------------------------
# The day
# ---------------------------------------------------------------------------------------------------

interface Number:
    int.sign.32;
    int.unsign.32;
end of interface Number

fn larger <T: Number> [let $a: <T>, let $b: <T>] -> <T>:
    if $a gt $b:
        ret $a;
    end of if
    ret $b;
end of fn larger

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
end of rec Day

proc read_day [] -> Day:
    var $c = Cursor[at = 1];
    var $sum: day.sum = 0;
    var $low: temp = 600;
    var $low_hour: day.hour = 0;
    var $high: temp = neg 500;
    var $high_hour: day.hour = 0;
    var $rise: day.step = 0;
    var $rise_hour: day.hour = 1;
    var $warm: day.count = 0;
    var $hot: day.count = 0;
    var $changes: day.count = 0;
    var $prev: temp = 0;
    var $alarm = Alarm.calm;
    std::print.shortest("alarm:  ");
    show_alarm(copy $alarm);
    for $hour [from: 0, before: $hours, by: 1]:
        let $t := next_reading(lend.mut $c)-
        otherwise:
            case done:
                std::print.shortest(newln, "the log ends after ", $hour, " readings of a day's ", $hours, "; stopping", newln);
                std::exit(code: 2);
            end of case done
            case bad_byte $at:
                std::print.shortest(newln, "the log has a byte no reading has, at ", $at, "; stopping", newln);
                std::exit(code: 2);
            end of case bad_byte
        end of otherwise;
        if $t lt $low:
            change $low to $t; change $low_hour to $hour;
        end of if
        if $t gt $high:
            change $high to $t; change $high_hour to $hour;
        end of if
        if $hour gt 0:
            let $step := $t minus $prev;
            if larger($step, neg $step) gt larger($rise, neg $rise):
                change $rise to $step; change $rise_hour to $hour;
            end of if
        end of if
        let $now := alarm_for($t);
        if rank(copy $now) at_least 1:
            change $warm to $warm plus 1;
        end of if
        if rank(copy $now) eq 2:
            change $hot to $hot plus 1;
        end of if
        if rank(copy $now) not_eq rank(copy $alarm):
            std::print.shortest(" -> ");
            show_alarm(copy $now);
            std::print.shortest(" at ");
            show_hour($hour);
            change $changes to $changes plus 1;
            change $alarm to copy $now;
        end of if
        change $sum to $sum plus $t;
        change $prev to $t;
    end of for
    std::print.shortest(newln);
    # a day is exactly its readings: one more would be another day's
    if not log_ends(lend.mut $c):
        std::print.shortest("the log holds more than a day's ", $hours, " readings; stopping", newln);
        std::exit(code: 2);
    end of if
    ret Day[sum = $sum, low = $low, low_hour = $low_hour, high = $high, high_hour = $high_hour, rise = $rise, rise_hour = $rise_hour, warm_hours = $warm, hot_hours = $hot, changes = $changes];
end of proc read_day

# A status word for a control panel, four bytes from the top: readings, warm hours, hot hours, alarm changes.
fn status_word [lent $d: Day] -> int.unsign.32:
    var $w = to.int.unsign.32($hours) shl 24;
    change $w to $w or.bit (to.int.unsign.32($d.warm_hours) shl 16);
    change $w to $w or.bit (to.int.unsign.32($d.hot_hours) shl 8);
    change $w to $w or.bit to.int.unsign.32($d.changes);
    ret $w;
end of fn status_word

proc report [lent $d: Day]:
    std::print.shortest("readings: ", $hours, ", in tenths of a degree", newln);
    std::print.shortest("lowest:  ");
    show_tenths($d.low);
    std::print.shortest(" C at ");
    show_hour($d.low_hour);
    std::print.shortest(newln, "highest: ");
    show_tenths($d.high);
    std::print.shortest(" C at ");
    show_hour($d.high_hour);
    let $mean := $d.sum div.near $hours;
    std::print.shortest(newln, "mean:    ");
    show_tenths($mean);
    std::print.shortest(" C, ");
    show_tenths(($mean times 9) div.near 5 plus 320);
    std::print.shortest(" F", newln, "steepest hour: ");
    if $d.rise gt 0:
        std::print.shortest("+");
    end of if
    show_tenths($d.rise);
    std::print.shortest(" C, from ");
    show_hour($d.rise_hour minus 1);
    std::print.shortest(" to ");
    show_hour($d.rise_hour);
    std::print.shortest(newln, "warm for ", $d.warm_hours, " hours, hot for ", $d.hot_hours, newln);
end of proc report

!MAIN:
    std::print.shortest("station 7, booting on a chip with no operating system", newln);
    std::print.shortest("log: ", arr.len($log), " bytes, CRC-32 ");
    show_hex(crc32());
    std::print.shortest(newln);
    let $day := read_day();
    report(lend $day);
    std::print.shortest("status word: ");
    show_hex(status_word(lend $day));
    std::print.shortest(newln, "switching off", newln);
    std::exit(code: 0);
!end of MAIN
