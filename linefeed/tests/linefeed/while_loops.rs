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

eval_and_assert!(
    while_loop_break_only_breaks_out_of_inner_loop,
    indoc::indoc! {r#"
        i = 0;
        while i < 2 {
            j = 0;

            while j <= 10 {
                if j == 2 {
                    break
                };
                j = j + 1;
                print("inner");
            };

            i = i + 1;
            print("outer");
        };
        print("done");
    "#},
    equals(indoc! {r#"
        inner
        inner
        outer
        inner
        inner
        outer
        done
    "#}),
    empty()
);

eval_and_assert!(
    break_outside_current_function_yields_error,
    indoc::indoc! {r#"
        fn inner() {
            break;
        };

        i = 0;
        while i <= 10 {
            inner();
            i = i + 1;
        };
    "#},
    empty(),
    contains("Error: Cannot break outside of loop")
);

eval_and_assert!(
    break_in_condition_expression,
    indoc::indoc! {r#"
        n = 5;
        while n > 0 {
            print(n);
            if (n == 3 and break) {
                print("unreachable");
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
    continue_works,
    indoc::indoc! {r#"
        i = -1;
        while (i = i + 1) <= 10 {
            continue if i % 2 == 1;
            print(i);
        };
    "#},
    equals(indoc! {r#"
        0
        2
        4
        6
        8
        10
    "#}),
    empty()
);

eval_and_assert!(
    while_loop_yields_last_iteration,
    indoc::indoc! {r#"
        i = -1;
        res = while (i = i + 1) < 5 {
          "iter"+i;
        };
        print(res);

        i = -1;
        res = while (i = i + 1) < 5 {
          break if i == 3;
          "iter"+i;
        };
        print(res);

        i = -1;
        res = while (i = i + 1) < 5 {
          continue if i == 3;
          "iter"+i;
        };
        print(res);
    "#},
    equals(indoc::indoc! {r#"
        iter4
        iter2
        iter4
    "#}),
    empty()
);
