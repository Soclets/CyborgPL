# A format width names what it counts (owner, 2026-10-25). `cols` counts screen columns: a Chinese, Japanese or
# Korean character and most emoji take two, an accent added onto a letter none, and an East Asian Ambiguous one
# (`°`, `±`, Greek, Cyrillic) one; `cols.east_asian` counts that last kind as two, for a terminal set to those
# languages. `graphemes`, `points` and `bytes` count as a text's length does. The `|` shows where each hole ends.
MAIN:
    std::print.format("{align: left, width: 8 cols}|{width: 6 cols, places: 1}|{newln}", "tea", 2.5);
    std::print.format("{align: left, width: 8 cols}|{width: 6 cols, places: 1}|{newln}", "日本茶", 3.5);
    std::print.format("{align: left, width: 8 cols}|{width: 6 cols, places: 1}|{newln}", "café", 4.0);
    std::print.format("{align: left, width: 8 cols}|{width: 6 cols, places: 1}|{newln}", "👍🏽 ok", 1.0);
    std::print.format("{width: 6 cols}|{width: 6 cols.east_asian}|{newln}", "20°C", "20°C");
    # U+FE0F after an emoji asks for its picture, two columns: ❤ is 1, ❤️ and 1️⃣ are 2
    std::print.format("{width: 4 cols}|{width: 4 cols}|{width: 4 cols}|{newln}", "❤", "❤️", "1️⃣");
    std::print.format("{width: 6 clusters}|{width: 6 points}|{width: 6 bytes}|{newln}", "日本", "日本", "日本");
    std::print.format("{align: center, width: 9 cols, fill: `-`}|{base: hex.upper, zeros, width: 6 cols}|{sign}|{newln}", "mid", 255, 7);
;MAIN
