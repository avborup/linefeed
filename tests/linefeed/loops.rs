use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
};

eval_and_assert!(
    while_loop_works,
    indoc::indoc! {r#"
        n = 3;
        while n > 0 {
            print(n);
            n = n - 1;
        };
    "#},
    equals(indoc! {r#"
        3
        2
        1
    "#}),
    empty()
);

eval_and_assert!(
    assign_in_while_loop_works,
    indoc::indoc! {r#"
        i = 0;
        while i < 3 {
            print(i = i + 1)
        };

        i = 0;
        while (i = i + 1) <= 3 {
            print(i)
        };
    "#},
    equals(indoc! {r#"
        1
        2
        3
        1
        2
        3
    "#}),
    empty()
);

eval_and_assert!(
    while_loop_break_works,
    indoc::indoc! {r#"
        n = 5;
        while n > 0 {
            print(n);
            if n == 3 {
                break
            };
            n = n - 1;
        };
    "#},
    equals(indoc! {r#"
        5
        4
        3
    "#}),
    empty()
);

eval_and_assert!(
    while_loop_break_outside_loop_yields_error,
    indoc::indoc! {r#"
        if 1 == 3 {
            break
        };
    "#},
    empty(),
    contains("Error: Cannot break outside of loop")
);

// TODO: add test that you cannot break out of a loop that is not inside the current function
