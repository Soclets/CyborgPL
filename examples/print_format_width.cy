# A format width names what it counts (owner, 2026-10-25). `cols` counts screen columns: a Chinese, Japanese or
# Korean character and most emoji take two, an accent added onto a letter none, and an East Asian Ambiguous one
# (`°`, `±`, Greek, Cyrillic) one; `cols.east_asian` counts that last kind as two, for a terminal set to those
# languages. `graphemes`, `points` and `bytes` count as a text's length does. The `|` shows where each hole ends.
!MAIN:
    std::print.format("{align: left, width: 8 columns}|{width: 6 columns, places: 1}|{newline}", "tea", 2.5);
    std::print.format("{align: left, width: 8 columns}|{width: 6 columns, places: 1}|{newline}", "日本茶", 3.5);
    std::print.format("{align: left, width: 8 columns}|{width: 6 columns, places: 1}|{newline}", "café", 4.0);
    std::print.format("{align: left, width: 8 columns}|{width: 6 columns, places: 1}|{newline}", "👍🏽 ok", 1.0);
    std::print.format("{width: 6 columns}|{width: 6 columns.east_asian}|{newline}", "20°C", "20°C");
    # U+FE0F after an emoji asks for its picture, two columns: ❤ is 1, ❤️ and 1️⃣ are 2
    std::print.format("{width: 4 columns}|{width: 4 columns}|{width: 4 columns}|{newline}", "❤", "❤️", "1️⃣");
    std::print.format("{width: 6 grapheme_clusters}|{width: 6 code_points}|{width: 6 bytes}|{newline}", "日本", "日本", "日本");
    std::print.format("{align: center, width: 9 columns, fill: `-`}|{base: hexadecimal.upper, zeros, width: 6 columns}|{sign}|{newline}", "mid", 255, 7);
!end of MAIN
