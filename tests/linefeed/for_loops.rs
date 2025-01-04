use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
};

eval_and_assert!(
    for_loop_works_on_range,
    indoc::indoc! {r#"
        for i in 0..5 {
            print(i);
        };
    "#},
    equals("0 \n 1 \n 2 \n 3 \n 4"),
    empty()
);

eval_and_assert!(
    for_loop_works_in_reverse,
    indoc::indoc! {r#"
        for i in 5..0 {
            print(i);
        };
    "#},
    equals("5 \n 4 \n 3 \n 2 \n 1"),
    empty()
);

eval_and_assert!(
    for_loop_works_with_negative_start,
    indoc::indoc! {r#"
        for i in -2..3 {
            print(i);
        };
    "#},
    equals("-2 \n -1 \n 0 \n 1 \n 2"),
    empty()
);

eval_and_assert!(
    for_loop_works_with_negative_end,
    indoc::indoc! {r#"
        for i in 2..-3 {
            print(i);
        };
    "#},
    equals("2 \n 1 \n 0 \n -1 \n -2"),
    empty()
);

eval_and_assert!(
    for_loop_continue_works,
    indoc::indoc! {r#"
        for i in 0..10 {
            continue if i % 2 == 1;
            print(i);
        };
    "#},
    equals("0 \n 2 \n 4 \n 6 \n 8"),
    empty()
);

eval_and_assert!(
    for_loop_break_works,
    indoc::indoc! {r#"
        for i in 0..10 {
            break if i == 5;
            print(i);
        };
    "#},
    equals("0 \n 1 \n 2 \n 3 \n 4"),
    empty()
);

eval_and_assert!(
    mixed_for_and_while_loops,
    indoc::indoc! {r#"
        for i in 0..3 {
            print(i);
            while i > 0 {
                i = i - 1;
                print(i);
            };
        };
    "#},
    equals("0 \n 1 \n 0 \n 2 \n 1 \n 0"),
    empty()
);

eval_and_assert!(
    for_loop_works_with_empty_range,
    indoc::indoc! {r#"
        for i in 0..0 {
            print(i);
        };
    "#},
    empty(),
    empty()
);

eval_and_assert!(
    for_loop_works_on_list,
    indoc::indoc! {r#"
        for i in [1, "whoo", 3] {
            print(i);
        };
    "#},
    equals("1 \n whoo \n 3"),
    empty()
);

eval_and_assert!(
    for_loop_works_on_empty_list,
    indoc::indoc! {r#"
        for i in [] {
            print(i);
        };
    "#},
    empty(),
    empty()
);
