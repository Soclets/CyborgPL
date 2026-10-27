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
    public.kit procedure write [$b: integer.unsigned.8_bit]:
        !no_check inline.ASM with [$b -> x0, clobber x9]:
            mov x9, #0x09000000
            strb w0, [x9]
        end of inline.ASM
    end of procedure write

    # Power off through PSCI's SYSTEM_OFF (0x8400_0008), over `hvc`; QEMU ends with it.
    public.kit procedure off [$code: integer.unsigned.8_bit]:
        !no_check inline.ASM with [$code -> x1, clobber x0]:
            movz x0, #0x0008
            movk x0, #0x8400, lsl #16
            hvc #0
        end of inline.ASM
    end of procedure off
end of module board

# ---------------------------------------------------------------------------------------------------
# The log, baked in, and the bytes it is read by
# ---------------------------------------------------------------------------------------------------

constant $log := include.bytes["data/log.txt"];

constant $newline: integer.unsigned.8_bit = 10;
constant $space: integer.unsigned.8_bit = 32;
constant $hash: integer.unsigned.8_bit = 35;
constant $minus: integer.unsigned.8_bit = 45;
constant $zero: integer.unsigned.8_bit = 48;
constant $nine: integer.unsigned.8_bit = 57;

# A temperature in tenths of a degree Celsius, as far as the sensor reads: -50.0 to 60.0.
collection.bounded temp:
    temp = integer.signed.32_bit[minimum: negative 500, maximum: 600];
end of collection.bounded temp

# A day is 24 hourly readings, so every count and sum of one has a bound the prover can see.
constant $hours: integer.signed.32_bit = 24;
collection.bounded day:
    day.hour = integer.signed.32_bit[minimum: 0, maximum: 23];
    day.count = integer.signed.32_bit[minimum: 0, maximum: 24];
    day.sum = integer.signed.32_bit[minimum: negative 12000, maximum: 14400];
    day.step = integer.signed.32_bit[minimum: negative 1100, maximum: 1100];
    day.tenths = integer.signed.32_bit[minimum: negative 2000, maximum: 2000];
end of collection.bounded day

# Where the reading of the log has got to: a position in it, from 1 (one past the end when it is done).
record Cursor:
    at: integer.unsigned.64_bit[minimum: 1];
end of record Cursor

# The next reading, past comment lines (`#` to the line's end) and blank ones; `done` at the log's end, and
# `bad_byte` with its position for a byte no reading has, or a number past what the sensor reads.
procedure next_reading [lent.mutable $c: Cursor] -> temp or fail[done, bad_byte: integer.unsigned.64_bit]:
    let $n := array.length($log);
    variable $at := $c.at;
    variable $first: integer.unsigned.8_bit = 0;
    loop:
        if $at greater_than $n:
            fail done;
        end of if
        let $b := $log[position: $at];
        if $b equal_to $hash:
            while $at at_most $n and.then $log[position: $at] not_equal_to $newline:
                change $at = $at plus 1;
            end of while
            continue;
        end of if
        if $b equal_to $newline or $b equal_to $space:
            change $at = $at plus 1; continue;
        end of if
        change $first = $b;
        break;
    end of loop
    let $negative := $first equal_to $minus;
    if $negative:
        change $at = $at plus 1;
    end of if
    variable $value: integer.signed.32_bit = 0;
    while $at at_most $n and.then $log[position: $at] not_equal_to $newline:
        let $b := $log[position: $at];
        if $b less_than $zero:
            fail bad_byte[$at];
        end of if
        if $b greater_than $nine:
            fail bad_byte[$at];
        end of if
        if $value greater_than 600:
            fail bad_byte[$at];
        end of if
        change $value = $value times 10 plus to.integer.signed.32_bit($b minus $zero);
        change $at = $at plus 1;
    end of while
    change $c.at = $at;
    if $negative:
        change $value = negate $value;
    end of if
    if $value less_than negative 500:
        fail bad_byte[$at];
    end of if
    if $value greater_than 600:
        fail bad_byte[$at];
    end of if
    let $t: temp = $value;
    return $t;
end of procedure next_reading

# Whether the log has nothing left to read.
procedure log_ends [lent.mutable $c: Cursor] -> boolean:
    let $next := next_reading(lend.mutable $c) otherwise:
        !case done:
            return true;
        !end of case done
        !case bad_byte $at:
            return false;
        !end of case bad_byte
    end of otherwise;
    return false;
end of procedure log_ends

# The CRC-32 of the whole log (the one zlib and PNG use, reflected, polynomial 0xEDB8_8320), bit by bit: no table,
# so no memory beyond the program's own.
function crc32 [] -> integer.unsigned.32_bit:
    variable $crc: integer.unsigned.32_bit = hexadecimal FFFF_FFFF;
    for $i [from: 1, to: array.length($log), by: 1]:
        change $crc = $crc xor to.integer.unsigned.32_bit($log[position: $i]);
        for $k [from: 1, to: 8, by: 1]:
            if ($crc and.bit 1) equal_to 1:
                change $crc = ($crc shift_right 1) xor hexadecimal EDB8_8320;
            end of if
            else:
                change $crc = $crc shift_right 1;
            end of else
        end of for
    end of for
    return $crc xor hexadecimal FFFF_FFFF;
end of function crc32

# ---------------------------------------------------------------------------------------------------
# Printing with no heap: every figure goes out as numbers and literal text, never as built text
# ---------------------------------------------------------------------------------------------------

# Tenths of a degree as degrees: `-3.5`, `21.0`.
procedure show_tenths [$t: day.tenths]:
    variable $a := $t;
    if $t less_than 0:
        std::print.shortest("-");
        change $a = negate $t;
    end of if
    std::print.shortest($a divide.floor 10, ".", $a modulo.floor 10);
end of procedure show_tenths

# An hour of the day as a clock time, `07:00`.
procedure show_hour [$h: day.hour]:
    if $h less_than 10:
        std::print.shortest("0");
    end of if
    std::print.shortest($h, ":00");
end of procedure show_hour

procedure show_digit [$d: integer.unsigned.32_bit]:
    if $d less_than 10:
        std::print.shortest($d); return;
    end of if
    if $d equal_to 10:
        std::print.shortest("a"); return;
    end of if
    if $d equal_to 11:
        std::print.shortest("b"); return;
    end of if
    if $d equal_to 12:
        std::print.shortest("c"); return;
    end of if
    if $d equal_to 13:
        std::print.shortest("d"); return;
    end of if
    if $d equal_to 14:
        std::print.shortest("e"); return;
    end of if
    std::print.shortest("f");
end of procedure show_digit

# A 32-bit word in hex, its eight digits from the top: `0x0018_0803`.
procedure show_hex [$x: integer.unsigned.32_bit]:
    std::print.shortest("0x");
    for $k [from: 1, to: 8, by: 1]:
        let $shift := to.integer.unsigned.32_bit(32 minus 4 times $k);
        show_digit(($x shift_right $shift) and.bit 15);
        if $k equal_to 4:
            std::print.shortest("_");
        end of if
    end of for
end of procedure show_hex

# ---------------------------------------------------------------------------------------------------
# The alarm: calm, warm over 22.0, hot over 25.0
# ---------------------------------------------------------------------------------------------------

union.tagged Alarm:
    calm;
    warm;
    hot;
end of union.tagged Alarm

function alarm_for [$t: temp] -> Alarm:
    if $t greater_than 250:
        return Alarm.hot;
    end of if
    if $t greater_than 220:
        return Alarm.warm;
    end of if
    return Alarm.calm;
end of function alarm_for

function rank [$a: Alarm] -> integer.signed.32_bit:
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
    return $r;
end of function rank

procedure show_alarm [$a: Alarm]:
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
end of procedure show_alarm

# ---------------------------------------------------------------------------------------------------
# The day
# ---------------------------------------------------------------------------------------------------

interface Number:
    integer.signed.32_bit;
    integer.unsigned.32_bit;
end of interface Number

function larger <T: Number> [$a: <T>, $b: <T>] -> <T>:
    if $a greater_than $b:
        return $a;
    end of if
    return $b;
end of function larger

# Everything the day comes to, worked out in one pass over its 24 readings.
record Day:
    sum: day.sum;
    low: temp;
    low_hour: day.hour;
    high: temp;
    high_hour: day.hour;
    rise: day.step;
    rise_hour: day.hour;
    warm_hours: day.count;
    hot_hours: day.count;
    changes: day.count;
end of record Day

procedure read_day [] -> Day:
    variable $c := Cursor[at = 1];
    variable $sum: day.sum = 0;
    variable $low: temp = 600;
    variable $low_hour: day.hour = 0;
    variable $high: temp = negative 500;
    variable $high_hour: day.hour = 0;
    variable $rise: day.step = 0;
    variable $rise_hour: day.hour = 1;
    variable $warm: day.count = 0;
    variable $hot: day.count = 0;
    variable $changes: day.count = 0;
    variable $prev: temp = 0;
    variable $alarm := Alarm.calm;
    std::print.shortest("alarm:  ");
    show_alarm(copy $alarm);
    for $hour [from: 0, before: $hours, by: 1]:
        let $t := next_reading(lend.mutable $c) otherwise:
            case done:
                std::print.shortest(newline, "the log ends after ", $hour, " readings of a day's ", $hours, "; stopping", newline);
                std::exit(code: 2);
            end of case done
            case bad_byte $at:
                std::print.shortest(newline, "the log has a byte no reading has, at ", $at, "; stopping", newline);
                std::exit(code: 2);
            end of case bad_byte
        end of otherwise;
        if $t less_than $low:
            change $low = $t; change $low_hour = $hour;
        end of if
        if $t greater_than $high:
            change $high = $t; change $high_hour = $hour;
        end of if
        if $hour greater_than 0:
            let $step := $t minus $prev;
            if larger($step, negate $step) greater_than larger($rise, negate $rise):
                change $rise = $step; change $rise_hour = $hour;
            end of if
        end of if
        let $now := alarm_for($t);
        if rank(copy $now) at_least 1:
            change $warm = $warm plus 1;
        end of if
        if rank(copy $now) equal_to 2:
            change $hot = $hot plus 1;
        end of if
        if rank(copy $now) not_equal_to rank(copy $alarm):
            std::print.shortest(" -> ");
            show_alarm(copy $now);
            std::print.shortest(" at ");
            show_hour($hour);
            change $changes = $changes plus 1;
            change $alarm = copy $now;
        end of if
        change $sum = $sum plus $t;
        change $prev = $t;
    end of for
    std::print.shortest(newline);
    # a day is exactly its readings: one more would be another day's
    if not log_ends(lend.mutable $c):
        std::print.shortest("the log holds more than a day's ", $hours, " readings; stopping", newline);
        std::exit(code: 2);
    end of if
    return Day[sum = $sum, low = $low, low_hour = $low_hour, high = $high, high_hour = $high_hour, rise = $rise, rise_hour = $rise_hour, warm_hours = $warm, hot_hours = $hot, changes = $changes];
end of procedure read_day

# A status word for a control panel, four bytes from the top: readings, warm hours, hot hours, alarm changes.
function status_word [lent $d: Day] -> integer.unsigned.32_bit:
    variable $w := to.integer.unsigned.32_bit($hours) shift_left 24;
    change $w = $w or.bit (to.integer.unsigned.32_bit($d.warm_hours) shift_left 16);
    change $w = $w or.bit (to.integer.unsigned.32_bit($d.hot_hours) shift_left 8);
    change $w = $w or.bit to.integer.unsigned.32_bit($d.changes);
    return $w;
end of function status_word

procedure report [lent $d: Day]:
    std::print.shortest("readings: ", $hours, ", in tenths of a degree", newline);
    std::print.shortest("lowest:  ");
    show_tenths($d.low);
    std::print.shortest(" C at ");
    show_hour($d.low_hour);
    std::print.shortest(newline, "highest: ");
    show_tenths($d.high);
    std::print.shortest(" C at ");
    show_hour($d.high_hour);
    let $mean := $d.sum divide.nearest $hours;
    std::print.shortest(newline, "mean:    ");
    show_tenths($mean);
    std::print.shortest(" C, ");
    show_tenths(($mean times 9) divide.nearest 5 plus 320);
    std::print.shortest(" F", newline, "steepest hour: ");
    if $d.rise greater_than 0:
        std::print.shortest("+");
    end of if
    show_tenths($d.rise);
    std::print.shortest(" C, from ");
    show_hour($d.rise_hour minus 1);
    std::print.shortest(" to ");
    show_hour($d.rise_hour);
    std::print.shortest(newline, "warm for ", $d.warm_hours, " hours, hot for ", $d.hot_hours, newline);
end of procedure report

!MAIN:
    std::print.shortest("station 7, booting on a chip with no operating system", newline);
    std::print.shortest("log: ", array.length($log), " bytes, CRC-32 ");
    show_hex(crc32());
    std::print.shortest(newline);
    let $day := read_day();
    report(lend $day);
    std::print.shortest("status word: ");
    show_hex(status_word(lend $day));
    std::print.shortest(newline, "switching off", newline);
    std::exit(code: 0);
!end of MAIN
