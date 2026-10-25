# Fixes, family by family

The owner, 2026-10-24: every message is its name, `code:` and `fix:` as edited code, and every edit kayo can
write is shown under `or:`; `fix: none suggested` only where the fix lies outside the program's own code; until a
message's fix is written its sentence stays on `about:`. Each fix is proven by `every_fix_clears_its_error`
(tests/conformance.rs): the reject program's message, each fix applied as its replacements, and the same error no
longer fires on that line.

Status: **written** (every site of the name has its fixes), **partly** (some sites; the rest keep `about:`, said
why), **none suggested** (the fix is outside the program, said why), **keeps about** (the right edit is the
writer's choice — a value, a name, where code moves — so kayo writes none and the sentence stays on `about:`,
said why), **—** (not yet).

## spelling and syntax (132 names)

| name | sites | status | notes |
|---|---|---|---|
| `any_stands_alone` | 2 | keeps about | the edit is the writer's: which interface they meant beside `any` |
| `argument_missing` | 2 | keeps about | the value is the writer's |
| `asm_binding_form` | 5 | keeps about | an assembly binding's register and direction are the writer's |
| `assignment_in_value` | 2 | written | `=` → `equal_to` |
| `belongs_at_top_level` | 6 | keeps about | a whole item moved out of a body: kayo would need the enclosing item's start, which the parser does not carry to the message |
| `belongs_in_body` | 3 | keeps about | which body it goes in is the writer's |
| `body_missing` | 1 | keeps about | the body is the writer's |
| `borrow_group_form` | 4 | keeps about | which group each borrow is in is the writer's |
| `borrow_group_needed` | 1 | keeps about | which input the borrow comes from is the writer's |
| `braces_need_commas` | 2 | written | the `,` put in, or a case's `;` made one |
| `byte_order_needless` | 1 | keeps about | the path rewritten around the call: not an edit kayo writes yet |
| `byte_order_unsaid` | 2 | keeps about | which byte order the file has is the writer's |
| `call_in_brackets` | 3 | written | both brackets swapped |
| `capture_verb_unsaid` | 1 | written | `lend` / `copy` / `give` in front |
| `case_unhandled` | 2 | keeps about | what the missing case does is the writer's |
| `case_value_form` | 1 | written | `is ` in front and its `;` |
| `change_unmarked` | 1 | written | `change ` in front |
| `character_empty` | 1 | keeps about | which character is the writer's |
| `check_misplaced` | 9 | partly | inside a value: moved in front of the statement; not yet: on a `fn`, at the top level, alone |
| `cnc::c_capitalized` | 4 | written | `C` |
| `cnc::extern_language_unknown` | 1 | written | `C` |
| `cnc::extern_language_unsaid` | 2 | partly | `.C` after `extern`; a renamed C function's `= C.…` keeps about |
| `cnc::extern_with_definition` | 2 | keeps about | C owns it: what to keep of the line is the writer's |
| `cnc::header_not_found` | 1 | keeps about | the header's path is the writer's |
| `cnc::incl_c_form` | 3 | keeps about | which kind of header is the writer's |
| `cnc::lent_across_c` | 1 | keeps about | what C is passed instead is the writer's |
| `cnc::unknown_layout` | 1 | written | `C` |
| `collection_form` | 9 | partly | a member's `fn` → the collection's name; not yet: templates, versions, a handler's arms |
| `collection_kind_unsaid` | 2 | keeps about | `coll.fn` or `coll.handler` is the writer's |
| `collection_line_form` | 4 | keeps about | the label's word is the writer's |
| `collection_member_unprefixed` | 6 | keeps about | the collection's name in front: four of its six sites do not hold the collection's name |
| `comment_form` | 5 | partly | `#x` → `# x`; not yet: the generated marker |
| `comparisons_chained` | 1 | keeps about | a chain written as `and` repeats the middle value; not an edit kayo writes yet |
| `const_changed` | 1 | keeps about | whether the value is a const or a variable is the writer's |
| `const_marked_mut` | 1 | written | the word out |
| `conversion_form` | 6 | keeps about | the target type is the writer's |
| `counter_is_position` | 1 | keeps about | one name for counter and position: which name stays is the writer's |
| `counting_loop_form` | 3 | keeps about | the loop's head rewritten: its values' spans are not carried here |
| `declaration_form` | 17 | partly | `let` put in, `mut` / a type moved where it goes, `:=` ↔ `=` by whether a type is written, `let.mut` → `let mut`; not yet: C's variable forms, a generic's one `< >`, a declaration with more than a name |
| `digit_grouping` | 2 | written | one `_` between digits; `1,000` → `1_000` |
| `digit_outside_base` | 1 | keeps about | which digit was meant is the writer's |
| `enum_empty` | 1 | keeps about | the cases are the writer's |
| `fail_list_empty` | 1 | keeps about | the reasons are the writer's |
| `field_form` | 7 | partly | the `:`, name first, `;` / `let` out, `:=` → `=`; not yet: `mut` / `immut` placement |
| `foreign_spelling` | 4 | partly | `0x1F` → `hex 1F`, `mod` → `module`, `usize` → `u64` / `u32`, `String` → `str.utf8` / `str.ascii`; not yet: a module include line |
| `function_type_form` | 1 | written | the names out |
| `generated_marker_form` | 1 | keeps about | where the marker goes: kayo generate writes it |
| `generic_bound_missing` | 1 | written | `: any` after the name |
| `generic_bound_twice` | 1 | written | the bound out of the mention |
| `ieee_misused` | 2 | partly | `.ieee` out; not yet: the checker's |
| `include_form` | 3 | keeps about | what the file is made into is the writer's |
| `include_path_not_literal` | 2 | keeps about | the path is the writer's |
| `inline_block_form` | 4 | keeps about | what the block holds is the writer's |
| `inline_language_capitals` | 3 | written | the capitals |
| `interface_bound_needless` | 1 | keeps about | the bound out: the site holds the name, not the bound's span |
| `interface_empty` | 1 | keeps about | what it promises is the writer's |
| `interface_has_code` | 1 | keeps about | where the code goes is the writer's |
| `interface_types_one_per_line` | 1 | keeps about | the types split one per line: the site holds no span for each type |
| `keyword_name_bare` | 2 | keeps about | the path in front: which kit's is the writer's |
| `label_needs_colon` | 3 | partly | the `:` put in after the first word; next: every word in the brackets at once |
| `label_order_unsaid` | 2 | keeps about | `labels.in_order` or `labels.any_order` is the writer's promise |
| `lambda_types_unknown` | 2 | keeps about | the types are the writer's |
| `leading_zero` | 1 | written | the zeros taken off |
| `lent_at_call` | 1 | written | `lend` |
| `level_chain_unsaid` | 2 | keeps about | how a chain reads is the writer's |
| `level_does_not_chain` | 1 | keeps about | which side is parenthesised is the writer's |
| `level_numbered_twice` | 1 | keeps about | the new number is the writer's |
| `level_order_unsaid` | 2 | keeps about | which way the levels count is the writer's |
| `limb_not_listed` | 1 | keeps about | a line under `my limbs:` in CyJet.toml, and a lone file has no manifest to edit |
| `loop_words_form` | 2 | keeps about | the loop's head rewritten: its values' spans are not carried here |
| `main_form` | 4 | written | `pub` out, `fn MAIN … {` → `MAIN {`, a call to `MAIN` goes |
| `main_parameters_wrong` | 4 | partly | one parameter: `, $argv: ptr[ptr[u8]]` added, or none; not yet: three or more |
| `mcps::seq_and_para` | 1 | written | either word out |
| `mcps::whichever_empty` | 1 | keeps about | the arms are the writer's |
| `module_name_lowercase` | 1 | written | the name lowercased, where that makes a name |
| `money::exactly_form` | 1 | keeps about | the number is the writer's |
| `name_not_english` | 1 | keeps about | the fix is a limb in the manifest (unicode), or a new name |
| `name_quoted` | 13 | written | the name bare |
| `name_reserved` | 10 | keeps about | the new name is the writer's |
| `needless_lent` | 1 | written | `lent` out |
| `needs_parentheses` | 7 | partly | both groupings, under `or:`; not yet the `not` next to a comparison (the unary site) |
| `no_starting_value` | 1 | keeps about | the starting value is the writer's (with the owner: declaring a name) |
| `not_built_yet` | 22 | none suggested | kayo has not built it: the fix is kayo's (design claude, 2026-10-24) |
| `not_changeable` | 2 | keeps about | what is changed instead is the writer's |
| `not_decided` | 7 | none suggested | no ruling yet: the fix is the owner's |
| `not_whole_number` | 4 | keeps about | the count is the writer's |
| `old_spelling` | 44 | partly | every replacement the spellings table or the message names, as edits (moves too: `no_cheat for`, `<T: any>`, a slice's `from:`/`to:`); not yet: a `range.` loop, `level.when_chained`, `change *$p =`, `2 ! 3` (no edit means anything there) |
| `on_form` | 5 | keeps about | which narrowing is the writer's |
| `operator_cannot_fail` | 1 | keeps about | an `op` made a `fn` needs a function's name, the writer's |
| `operator_cannot_level` | 1 | keeps about | the operator out of a level list: the list's span is not carried |
| `operator_value_form` | 5 | partly | `op pow` for `op xx`; a keyword or another token keeps about (the writer's) |
| `operators_level_apart` | 1 | keeps about | which level each goes in is the writer's |
| `otherwise_empty` | 2 | keeps about | what happens on failure is the writer's |
| `otherwise_misplaced` | 7 | keeps about | where the failure is handled is the writer's |
| `parameter_form` | 6 | partly | `var` out, name first, `lent.str.utf8 $t` → `lent $t: str.utf8`; not yet: `lend` / an owning keyword |
| `parens_for_brackets` | 4 | written | both brackets swapped |
| `preset_wrong` | 4 | written | the close parameter |
| `print_weird_form` | 3 | partly | `.weird` first; not yet: arrows for `[ ]` |
| `quoted_name_form` | 4 | keeps about | the name's characters are the writer's |
| `range_as_value` | 2 | keeps about | what the value is instead is the writer's |
| `return_type_form` | 3 | partly | `:` → `->`; not yet: a return type on `fn`, a borrow's |
| `return_type_missing` | 5 | keeps about | the type is known only after checking, and the signature's span is not carried there |
| `said_twice` | 5 | keeps about | which line stays is the writer's |
| `shadow_form` | 1 | written | `let` out |
| `space_missing` | 7 | partly | the space put in; not yet: `<` standing alone in a type |
| `spelling_removed` | 27 | written | the mark, `#strict` or `lang:` goes (a line of its own whole); `var` → `let` / `const`; the `bare.` sites after `bare_mark` are unreachable since the marks went |
| `step_missing` | 1 | written | `, by: 1` before the `]` |
| `strict_misplaced` | 1 | written | the line goes |
| `strict_word_missing` | 2 | keeps about | `#strict` is gone; this is unreachable |
| `table_bytes_form` | 3 | keeps about | the size is the writer's |
| `tier_words_order` | 3 | partly | the two words swapped; `no_cheat` inside a `for` keeps about (it moves) |
| `top_level_needs_const` | 4 | written | `const` |
| `two_tiers` | 1 | keeps about | which tier the statement takes is the writer's |
| `two_verbs` | 2 | keeps about | which verb stays is the writer's |
| `type_incomplete` | 5 | keeps about | the element type is the writer's |
| `unclosed_block` | 16 | partly | the `}` above the next item, or a last line; not yet: a `[` never closed |
| `unclosed_quote` | 3 | written | closed at the end of its line, before a `;` |
| `unexpected_text` | 53 | keeps about | the grammar's expectation, many shapes: no one edit |
| `unicode::case_mark_kind_unknown` | 2 | keeps about | the kind is the writer's |
| `unicode::case_mark_misplaced` | 6 | keeps about | a move to `kit.cy` or the file's top, across files |
| `unicode::case_mark_order` | 1 | written | the two words swapped |
| `unknown_handler` | 1 | written | the close name |
| `unknown_inline_language` | 1 | written | the close language |
| `unknown_narrowing` | 3 | partly | `must`'s close form, `concur.detached`; not yet: a type's `.of` |
| `unknown_operator` | 6 | partly | `=` → `equal_to`; not yet: another kit's operator |
| `unknown_pub_narrowing` | 2 | written | `pub.kit` |
| `unknown_target` | 5 | keeps about | the chip or target is the writer's |
| `unknown_verb` | 4 | keeps about | `lent` or `lent.mut` is the writer's |
| `variable_name_form` | 5 | partly | `$` in front; not yet: a lone `$` |
| `variadic_form` | 2 | keeps about | a parameter moved last: the parameters' spans are not carried |
| `versions_outside_collection` | 3 | keeps about | a `coll.fn` block written around both, across the file |
| `wrap_needless` | 1 | written | `.wrap` out |

## names and paths (62 names)

| name | sites | status | notes |
|---|---|---|---|
| `ai::brain_width` | 1 | written | `brain16` |
| `bound_given_twice` | 1 | keeps about | which bound stays is the writer's |
| `bound_not_taken` | 2 | keeps about | the bound out, or another type: the writer's |
| `bound_outside_type` | 6 | keeps about | the bound or the type is the writer's |
| `bound_unit_misplaced` | 6 | written | the unit out |
| `bound_unit_unsaid` | 1 | written | each unit before the `]` |
| `bound_wrong_type` | 8 | keeps about | the bound's values are the writer's |
| `bounds_backwards` | 6 | written | the two sides swapped |
| `builtin_operator_redefined` | 3 | keeps about | the operator's new name is the writer's |
| `cnc::bit_field_form` | 2 | keeps about | the field's type is the writer's |
| `cnc::bit_field_too_wide` | 1 | keeps about | the width is the writer's |
| `cnc::bits_outside_layout` | 1 | keeps about | `layout.C` on the struct, or the bits out: the writer's |
| `cnc::c_cannot_fail` | 1 | keeps about | the wrapper is the writer's |
| `cnc::c_enum_carries_data` | 1 | keeps about | what the case carries instead is the writer's |
| `cnc::c_enum_number_missing` | 1 | keeps about | the number is C's header's, which only the writer reads |
| `cnc::c_enum_number_twice` | 1 | keeps about | which number each case has is the writer's |
| `cnc::c_str_counted_in_bytes` | 1 | keeps about | the bound in bytes is the writer's |
| `cnc::c_str_zero_misplaced` | 1 | written | the words out |
| `cnc::c_str_zero_unsaid` | 2 | partly | `exclude zero` / `include zero`; not yet: the parser's |
| `cnc::c_type_unknown` | 1 | keeps about | C's type is the writer's |
| `cnc::cannot_cross_to_c` | 7 | partly | a C variable's type `i32`; the rest keep about: a callback's, a generic's, a range's type is the writer's |
| `cnc::not_c_shaped` | 1 | keeps about | the field's C type is the writer's |
| `cnc::not_layout_c` | 1 | keeps about | `layout.C` on a declaration elsewhere: its span is not carried |
| `defined_twice` | 13 | keeps about | the new name is the writer's |
| `function_owner_wrong` | 1 | keeps about | what the function belongs to is the writer's |
| `kit_name_needs_path` | 2 | written | the kit's prefix in front, or a `use … as …;` line |
| `kit_not_listed` | 2 | keeps about | the kit's block in CyJet.toml needs its id or path, the writer's |
| `lane_count_misplaced` | 1 | keeps about | the type is the writer's |
| `length_bound_on_array` | 2 | written | the bound out |
| `main_not_at_root` | 1 | keeps about | a move to `kit.cy`, across files |
| `mcps::channel_bound_form` | 1 | keeps about | the bound is the writer's |
| `member_names_itself` | 1 | keeps about | which function it names is the writer's |
| `money::percent_number_type` | 1 | keeps about | `d` or `f` is the writer's |
| `money::percent_unsized` | 1 | keeps about | the number type is the writer's |
| `money::places_on_bin` | 1 | keeps about | `d` instead, with its width: the writer's |
| `my_after_kit_path` | 1 | written | `my.` out |
| `my_needs_path` | 2 | keeps about | the item reached is the writer's |
| `narrowing_not_taken` | 2 | written | the narrowing out |
| `no_such_width` | 1 | written | `f64` / `d128` |
| `not_a_map_key` | 4 | keeps about | the key type is the writer's |
| `not_a_module` | 3 | keeps about | the case is the writer's |
| `not_a_type` | 1 | keeps about | the type is the writer's |
| `not_public` | 5 | partly | `pub.kit` / `pub` in front of the declaration; another kit's: none suggested (its code is that kit's) |
| `operator_operand_count` | 2 | keeps about | which operands stay is the writer's |
| `parameter_type_missing` | 4 | keeps about | the type is the writer's |
| `simd_element_type` | 1 | keeps about | the lane type is the writer's |
| `simd_lane_count_mismatch` | 4 | partly | the shape the machine has; a mask's lanes keep about |
| `simd_lane_width` | 1 | keeps about | the width is the writer's |
| `std_name_needs_path` | 5 | written | `std::` in front |
| `std_not_listed` | 6 | keeps about | a block under `my kits:` in CyJet.toml, and a lone file has no manifest to edit |
| `test_fn_misused` | 5 | keeps about | the values move into the body: the writer's |
| `type_parameter_count` | 9 | keeps about | the parameters are the writer's |
| `type_parameter_form` | 3 | partly | `list[i32]`; not yet: `simd`'s |
| `unannounced` | 3 | keeps about | the announcement goes after the function's name, whose span is not carried here |
| `unknown_const` | 1 | written | the close const |
| `unknown_function` | 4 | written | the close name; no declaration (owner: (b)) |
| `unknown_interface` | 3 | keeps about | the interface is the writer's |
| `unknown_map_variant` | 1 | written | each variant |
| `unknown_text_unit` | 5 | written | the close unit or encoding |
| `unknown_type` | 1 | written | the close name |
| `wsmw::no_such_width` | 4 | keeps about | the width is the writer's |
| `wsmw::standard_width` | 1 | written | the standard width |

## types and values (161 names)

| name | sites | status | notes |
|---|---|---|---|
| `ai::brain_mixed` | 1 | keeps about | the conversion is the writer's |
| `ai::brain_to_deci_unsaid` | 1 | written | `to.useful.T` / `to.stored.T` |
| `already_declared` | 5 | partly | `shadow` for `let`; a loop variable's, a const's, a capture's new name keeps about (the writer's) |
| `argument_label_missing` | 9 | partly | the label put in, or a mistyped one replaced (`Edit::Label`); not yet: a handler's argument, a labelled argument of the own function |
| `argument_label_unknown` | 5 | partly | the close label; not yet: a version's, a handler's |
| `array_cannot_grow` | 2 | keeps about | `list` at the declaration, whose span is not carried to the call |
| `array_never_filled` | 1 | written | `list` |
| `borrow_outlives_owner` | 2 | keeps about | what is handed back instead is the writer's |
| `borrow_outside_group` | 3 | keeps about | which parameters the group marks is the writer's |
| `bound_not_satisfied` | 3 | keeps about | the type or the bound is the writer's |
| `bundle_loop_form` | 2 | keeps about | how the loop takes the elements is the writer's |
| `call_ambiguous` | 1 | keeps about | which version is meant is the writer's |
| `call_gives_nothing` | 3 | keeps about | what the line does instead is the writer's |
| `call_without_parens` | 3 | partly | `()` after it; not yet: a path alone as a statement |
| `cannot_change_immut` | 3 | keeps about | whether it is `mut` is the writer's |
| `carries_nothing` | 4 | partly | a bare case; `fail x[v]` and a named payload keep about (not yet) |
| `case_data_form` | 2 | keeps about | which value it carries is the writer's |
| `case_handled_twice` | 1 | keeps about | which arm stays is the writer's |
| `case_value_misplaced` | 1 | keeps about | where the value goes is the writer's |
| `case_value_missing` | 1 | keeps about | the value is the writer's |
| `changed_through_lend` | 2 | keeps about | how it is lent is the caller's |
| `cnc::c_str_unknown` | 1 | keeps about | which C string operation is the writer's |
| `cnc::c_variable_read_only` | 1 | keeps about | `mut` on C's variable is the writer's promise |
| `cnc::callback_captures` | 1 | keeps about | what C is handed instead is the writer's |
| `cnc::no_check_missing` | 2 | written | `no_check ` in front of the statement |
| `cnc::numbered_enum_not_c` | 3 | keeps about | `layout.C` or the numbers out is the writer's |
| `cnc::opaque_struct_held` | 3 | partly | `ptr.mut[T]`; a built or sized one keeps about |
| `cnc::pointer_target_wrong` | 2 | keeps about | the cast is the writer's |
| `cnc::spread_into_c` | 1 | keeps about | the arguments are the writer's |
| `concat_needs_text` | 4 | partly | `to.str.utf8(…)` around a number; another type keeps about (the writer's) |
| `const_not_known` | 5 | keeps about | the value is the writer's |
| `const_shadowed` | 4 | keeps about | the new name is the writer's |
| `conversion_target_unsaid` | 1 | written | `to.useful.T` / `to.stored.T` |
| `count_negative` | 1 | keeps about | the count is the writer's |
| `counter_shadowed` | 1 | keeps about | the new name is the writer's |
| `data_missing` | 4 | keeps about | the value is the writer's (owner: (b)) |
| `duplicate_map_key` | 1 | keeps about | which entry stays is the writer's |
| `element_given_away` | 1 | keeps about | `copy` or `list.remove` is the writer's |
| `empty_map_literal` | 1 | written | `map.empty` |
| `empty_untyped` | 3 | partly | `list.empty`; `ptr.null`'s type keeps about (the writer's) |
| `failure_case_unhandled` | 1 | keeps about | what each arm does is the writer's |
| `failure_handled_twice` | 1 | keeps about | which arm stays is the writer's |
| `failure_never_happens` | 3 | keeps about | the arm out: the arm's span is not carried |
| `failure_not_listed` | 4 | keeps about | the reason added to the signature, whose span is not carried |
| `failure_unhandled` | 6 | keeps about | what happens on failure is the writer's (owner: (b), no `otherwise` fallback) |
| `false_satisfies_claim` | 1 | keeps about | what it satisfies is the writer's |
| `field_missing` | 1 | keeps about | the field's value is the writer's (owner: (b)) |
| `field_name_in_call` | 2 | partly | `x = ` out; a map pair in a call keeps about (the writer's) |
| `field_unnamed` | 2 | keeps about | the field's name is the writer's |
| `file_not_readable` | 1 | keeps about | the path is the writer's |
| `format_holes_mismatch` | 3 | keeps about | which value or hole is meant is the writer's |
| `format_text_wrong` | 2 | partly | the text repaired (`\\%` → `%`, holes from `{1}`, a lone brace escaped, `#` out, a kind that does not fit taken off); not yet: mixed holes (which number each `{}` meant is the writer's) |
| `function_shape_wrong` | 4 | keeps about | the function is the writer's |
| `generic_filled_inconsistently` | 2 | keeps about | which type is meant is the writer's |
| `generic_size_unknown` | 1 | keeps about | the type is the writer's |
| `generic_unfilled` | 2 | keeps about | the type is the writer's |
| `given_twice` | 2 | keeps about | which value stays is the writer's |
| `handler_falls_through` | 1 | keeps about | the arm's value is the writer's |
| `handler_value_unused` | 1 | keeps about | the value out: its span is not carried |
| `include_not_const` | 1 | keeps about | the const is the writer's |
| `included_text_invalid` | 5 | keeps about | the file's encoding is the writer's |
| `index_out_of_range` | 3 | partly | check it before use: the statement under `if ($k at_least 1) and ($k at_most std::list.length(lend $xs)) {`, where the code is a named list's `[pos: …]`; not yet: a list that is not a name, a slice |
| `iterator_has_no_position` | 1 | keeps about | `pos` out: its span is not carried |
| `iterator_next_form` | 1 | keeps about | the `next` function is the writer's |
| `label_outside_call` | 4 | keeps about | where the label goes is the writer's |
| `labels_out_of_order` | 1 | keeps about | the arguments reordered: several moves at once |
| `lend_mut_needed` | 8 | partly | `lend` / a verb → `lend.mut`; not yet: a borrow handed back, a `for` over an iterator |
| `lend_size_mismatch` | 1 | keeps about | the type is the writer's |
| `lent_given_away` | 2 | written | `copy` |
| `lent_twice_in_call` | 1 | keeps about | which lend changes is the writer's |
| `list_array_mismatch` | 4 | keeps about | `list` or `array` is the writer's |
| `loop_variable_typed` | 1 | written | the type out |
| `main_cannot_fail` | 2 | keeps about | how MAIN handles it is the writer's |
| `map_entry_form` | 1 | keeps about | the value is the writer's |
| `match_needs_enum` | 1 | keeps about | the `if` is the writer's |
| `match_value_unused` | 1 | keeps about | where the value goes is the writer's |
| `mcps::channel_form` | 5 | keeps about | the channel's type is the writer's |
| `mcps::detached_lend` | 1 | keeps about | how the task takes it is the writer's |
| `mcps::send_needs_give` | 1 | partly | `give` / `copy` |
| `mcps::task_returns` | 1 | keeps about | how the task ends is the writer's |
| `mod_near_on_fraction` | 1 | keeps about | which operation is meant is the writer's |
| `money::bin_to_deci_unsaid` | 1 | written | `to.useful.T` / `to.stored.T` |
| `money::deci_mixed` | 2 | keeps about | which width is the writer's |
| `money::exact_into_bin` | 1 | keeps about | the type is the writer's |
| `money::exactly_needed` | 1 | written | `exactly ` in front |
| `money::percent_mixed` | 3 | keeps about | the conversion is the writer's |
| `money::power_not_whole` | 2 | keeps about | the exponent is the writer's |
| `money::too_many_digits` | 2 | keeps about | the digits are the writer's |
| `money::too_many_places` | 3 | keeps about | the places are the writer's |
| `must_misused` | 2 | keeps about | where the test goes is the writer's |
| `mut_never_changed` | 1 | written | `mut` out of its declaration |
| `needs_integer` | 11 | keeps about | the type is the writer's |
| `negative_exponent` | 1 | keeps about | the exponent is the writer's |
| `negative_unsigned` | 2 | keeps about | the type is the writer's |
| `no_check_missing` | 5 | partly | `no_check ` in front of the statement; not yet: a format text not written in the call |
| `no_implicit_conversion` | 1 | written | the declared type changed, or a conversion that cannot fail; one that can keeps about (owner: (b)) |
| `no_such_conversion` | 8 | keeps about | the conversion is the writer's |
| `no_version_fits` | 1 | keeps about | the arguments are the writer's |
| `not_a_function` | 7 | keeps about | the path is the writer's |
| `not_a_struct` | 2 | keeps about | the `match` is the writer's |
| `not_a_value` | 7 | keeps about | the value is the writer's |
| `not_declared` | 8 | written | the close name in scope; no declaration (owner, 2026-10-24: (b), a fix never makes up a value) |
| `not_indexable` | 2 | keeps about | the lookup and its `otherwise` are the writer's (owner: (b)) |
| `not_one_code_point` | 1 | keeps about | the type is the writer's |
| `not_text` | 4 | keeps about | how it becomes text is the writer's |
| `not_walkable` | 4 | keeps about | what the loop runs over is the writer's |
| `nothing_to_give` | 2 | partly | the verb out; not yet: the checker's other site |
| `nothing_to_shadow` | 1 | written | `let` for `shadow` |
| `operator_called_as_function` | 2 | partly | `a op b` for `op(a, b)`; a declared operator's other site keeps about |
| `operator_cannot_be_value` | 1 | keeps about | the function is the writer's |
| `operator_value_ambiguous` | 1 | keeps about | where it is handed is the writer's |
| `operator_wrong_type` | 9 | keeps about | the operand is the writer's |
| `otherwise_never_needed` | 10 | partly | the `otherwise` clause out; not yet: a list read |
| `outside_loop` | 2 | keeps about | how the code leaves is the writer's |
| `owned_value_bare` | 9 | partly | `give` / `copy` / `lend` in front, as the message lists; not yet: a `for`'s list, a spread, a conversion |
| `pointer_cast_target` | 2 | keeps about | the type is the writer's |
| `pointer_outside_boundary` | 1 | keeps about | the declaration is the writer's |
| `pointer_read_only` | 1 | keeps about | `ptr.mut` where it is made: the writer's |
| `position_unlabelled` | 7 | partly | `pos: ` in front; not yet: a slice's, an index in the parser |
| `print_form_unsaid` | 4 | partly | `.useful` / `.stored`; not yet: the `.weird` forms |
| `print_into_non_writer` | 1 | keeps about | the writer is the writer's |
| `returned_borrow_form` | 5 | partly | `lend` / `lend.mut`; a call's borrow keeps about |
| `returned_borrow_misused` | 1 | keeps about | how it is held is the writer's |
| `returned_borrow_unsaid` | 2 | keeps about | `lent ` before the return type, whose span is not carried |
| `rounding_unsaid` | 1 | written | the way meant first, then each |
| `shadow_sealed` | 1 | keeps about | the new name is the writer's |
| `shadow_unsaid` | 1 | written | `shadow` for `let` |
| `shuffle_not_fixed` | 2 | keeps about | the lanes are the writer's |
| `simd_lane_count_unknown` | 1 | keeps about | the type is the writer's |
| `size_unit_unsaid` | 1 | written | ` bytes` |
| `spread_length_unknown` | 1 | keeps about | the arguments are the writer's |
| `spread_not_variadic` | 1 | keeps about | the arguments are the writer's |
| `spread_outside_call` | 1 | keeps about | the call is the writer's |
| `spread_verb_wrong` | 4 | partly | the verb out, or `...copy` / `...give`; not yet: the parser's `...lend` |
| `text_not_ascii` | 1 | keeps about | the type or the text is the writer's |
| `text_order_undecided` | 2 | keeps about | not decided by the owner yet |
| `text_unit_unsaid` | 6 | partly | each unit; not yet: a `for` over a text, a text's `[…]` |
| `typed_literal_form` | 2 | written | `to.T($x)` |
| `unicode::no_single_code` | 2 | keeps about | which unit is the writer's |
| `union_one_field` | 1 | keeps about | which field stays is the writer's |
| `unknown_case` | 2 | written | the close name |
| `unknown_field` | 2 | written | the close name |
| `unknown_operation` | 6 | written | the close operation, where one is close |
| `unknown_pointer_operation` | 1 | written | the close operation |
| `unknown_rounding` | 3 | written | the five ways |
| `unknown_std_name` | 4 | keeps about | std has not written it: the fix is std's |
| `unknown_struct` | 1 | written | the close struct |
| `unlabelled_after_labelled` | 1 | keeps about | an argument moved: several moves at once |
| `unordered_map_walked` | 1 | keeps about | the map's kind is the writer's |
| `value_does_not_fit` | 11 | partly | a whole number declared too narrow: the narrowest type of its sign that holds it; not yet: worked-out values (bound where read, waits on the root) |
| `value_left_unused` | 4 | keeps about | what happens to the value is the writer's |
| `verb_on_fresh_value` | 4 | partly | the verb out of a call's result; not yet: the place-use sites |
| `verb_wrong_for_parameter` | 14 | partly | the verb the parameter takes, or the verb out; not yet: C's pointers, a borrow handed back |
| `versions_overlap` | 4 | keeps about | which version stays is the writer's |
| `width_unknown` | 11 | keeps about | the width is the writer's |
| `wrap_sizes_mixed` | 1 | keeps about | which size is the writer's |
| `writer_size_zero` | 1 | keeps about | the size is the writer's |
| `writer_way_unknown` | 1 | written | the close way |
| `written_not_literal` | 1 | keeps about | how it prints is the writer's |
| `wrong_argument_count` | 14 | keeps about | the arguments are the writer's |
| `wrong_type` | 51 | partly | the declared type changed, or a conversion that cannot fail (owner: (b)); the rest keep about: the value is simply wrong, or the conversion can fail |

## ownership and lends (2 names)

| name | sites | status | notes |
|---|---|---|---|
| `used_after_give` | 1 | partly | `give $s` → `copy $s` where it was given; none for a `for`'s source (an iterator is walked by giving it) or a give with no `give` word |
| `writer_unflushed` | 1 | written | the flush above the block's `}` |

## checks (Precog) (14 names)

| name | sites | status | notes |
|---|---|---|---|
| `bound_not_kept` | 1 | keeps about | bound the value where it is read: the message does not carry that place yet, and the bound itself is the writer's |
| `check_not_enough` | 1 | keeps about | bound the value where it is read: the message does not carry that place yet, and the bound itself is the writer's |
| `duration_negative` | 1 | written | check it before use: `if $a at_least $b {` |
| `float_too_large` | 1 | keeps about | bound the value where it is read: the message does not carry that place yet, and the bound itself is the writer's |
| `hash_disagrees_with_equal` | 1 | keeps about | the hash or the equality is the writer's |
| `length_bound_not_kept` | 1 | keeps about | bound the value where it is read: the message does not carry that place yet, and the bound itself is the writer's |
| `mcps::send_never_taken` | 1 | keeps about | who takes it is the writer's |
| `mcps::task_outlives_lend` | 1 | keeps about | how the task takes it is the writer's |
| `mcps::tasks_wait_on_each_other` | 1 | keeps about | the order tasks wait in is the writer's |
| `no_check_lied` | 1 | written | the `no_check` taken out |
| `shift_loses_every_bit` | 1 | partly | a `shr`'s count guarded by the width |
| `shift_overflows` | 1 | keeps about | a `shl`'s site asks of the count and the value at once: which one the writer means is theirs |
| `slice_out_of_range` | 1 | keeps about | which range is meant is the writer's |
| `too_wide_in_between` | 1 | keeps about | bound the value where it is read: the message does not carry that place yet, and the bound itself is the writer's |

## kits, files and the manifest (40 names)

| name | sites | status | notes |
|---|---|---|---|
| `build_file_missing` | 1 | keeps about | the build file is the writer's |
| `cnc::c_lib_name_form` | 2 | keeps about | the library's name is the writer's |
| `file_name_not_module` | 5 | keeps about | a rename, which no text edit expresses |
| `kit_not_found` | 2 | keeps about | which kit is meant is the writer's |
| `kit_prefix_clash` | 4 | none suggested | the prefixes are the depended-on kits' (owner's example) |
| `kits_depend_in_circle` | 2 | none suggested | the order kits depend in is not one program's code (owner's example) |
| `limb_retired` | 3 | partly | the line out; one now in unicode keeps about (the line rewritten: not yet) |
| `manifest_block_unclosed` | 2 | written | the `}` above the next line, or a last line |
| `manifest_default_unclear` | 3 | keeps about | which target is the default is the writer's |
| `manifest_field_misplaced` | 1 | keeps about | where the key goes is the writer's |
| `manifest_field_missing` | 6 | keeps about | the value is the writer's (owner: (b)) |
| `manifest_given_twice` | 7 | partly | the second line goes; not yet: a block given twice |
| `manifest_id_form` | 1 | keeps about | the id is the kit's creator's |
| `manifest_kit_named_in_block` | 1 | written | the line goes |
| `manifest_limb_block_name` | 1 | written | the limb's own name |
| `manifest_line_form` | 6 | keeps about | the line's shape is the writer's |
| `manifest_missing` | 2 | none suggested | the missing manifest is a depended-on kit's |
| `manifest_name_form` | 1 | keeps about | the name is the writer's |
| `manifest_section_form` | 2 | keeps about | the block is the writer's |
| `manifest_source_unclear` | 4 | keeps about | which source stays is the writer's |
| `manifest_unknown_field` | 4 | written | the close key |
| `manifest_unknown_section` | 1 | written | the close section name |
| `manifest_value_form` | 1 | written | `yes` / `no` |
| `manifest_value_quoted` | 1 | written | the quotes off |
| `manifest_version_too_long` | 1 | keeps about | the version is the writer's |
| `manifest_version_word` | 1 | written | `ver` |
| `name_ambiguous` | 2 | keeps about | which kit is meant is the writer's |
| `old_file_name` | 8 | keeps about | a rename, which no text edit expresses |
| `operator_level_given_twice` | 1 | keeps about | which level stays is the writer's |
| `path_not_readable` | 3 | keeps about | the path is the writer's |
| `power_except_misplaced` | 1 | keeps about | a line moved between manifest sections |
| `root_file_missing` | 2 | keeps about | the root file is the writer's |
| `target_function_wrong` | 5 | keeps about | the function is the writer's |
| `target_layout_missing` | 3 | keeps about | the linker script is the writer's |
| `target_not_buildable` | 2 | none suggested | the target is kayo's to build, not the program's (owner's example: past the target's limits) |
| `target_not_listed` | 1 | keeps about | which target is the writer's |
| `unicode::case_marks_disagree` | 1 | keeps about | which mark stays is the writer's |
| `unicode::names_differ_only_in_case` | 1 | keeps about | the new name is the writer's |
| `unknown_power` | 1 | written | the close power |
| `version_not_available` | 2 | written | std's `ver` |

## other: templates, uses, sources (12 names)

| name | sites | status | notes |
|---|---|---|---|
| `concise_form` | 1 | keeps about | the concise file is written by a tool, not by hand |
| `division_by_zero` | 1 | partly | check it before use: `if $b not_equal 0 {`, where the divisor is written plainly |
| `generated_code_stale` | 1 | written | what `kayo generate` writes, in place of what is there |
| `mcps::para_cannot_split` | 1 | written | `para` out: a sequential `for` |
| `missing_return` | 4 | keeps about | the value is the writer's |
| `no_truthiness` | 5 | keeps about | the comparison is the writer's |
| `step_zero` | 2 | written | a step of `1` |
| `template_runs_forever` | 1 | keeps about | the loop's end is the writer's |
| `template_too_many_members` | 1 | keeps about | the range is the writer's |
| `template_value_unknown` | 4 | keeps about | the value is the writer's |
| `use_form` | 5 | partly | ` as NAME` after the path; a use of this kit's own path goes; not yet: a `use` below other lines (it moves), a missing `;` |
| `use_name_taken` | 1 | keeps about | the new name is the writer's |

## Metal, `inline.MSL` (10 names)

| name | sites | status | notes |
|---|---|---|---|
| `metal::binding_verb_wrong` | 3 | written | the verb the value takes, `lend` / `lend.mut` or `copy` |
| `metal::cannot_bind` | 2 | keeps about | what is bound instead is the writer's |
| `metal::compile_failed` | 1 | none suggested | the fix is in the writer's Metal, which kayo hands on unchanged (design claude, 2026-10-25); Apple's message stays on `about:` |
| `metal::include_missing` | 1 | keeps about | the path is the writer's |
| `metal::include_outside_kit` | 1 | keeps about | where the shared file lives is the writer's |
| `metal::kernel_missing` | 1 | keeps about | the kernel is the writer's |
| `metal::run_failed` | 1 | none suggested | the GPU refused at run time: the machine's, not the program's |
| `metal::slot_bound_twice` | 1 | keeps about | which value takes which slot is the writer's |
| `metal::threads_unsaid` | 2 | keeps about | the count is the writer's |
| `metal::two_kernels` | 1 | keeps about | which kernel the block runs is the writer's |
