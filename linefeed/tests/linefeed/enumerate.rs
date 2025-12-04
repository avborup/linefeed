use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    enumerate_basic,
    indoc! {r#"
        list = ["a", "b", "c"];
        for i, val in list.enumerate() {
            print(i, val);
        };
    "#},
    equals("0 a \n 1 b \n 2 c"),
    empty()
);

eval_and_assert!(
    enumerate_empty_list,
    indoc! {r#"
        for i, val in [].enumerate() {
            print(i, val);
        };
    "#},
    empty(),
    empty()
);

eval_and_assert!(
    enumerate_single_element,
    indoc! {r#"
        for i, val in [42].enumerate() {
            print(i, val);
        };
    "#},
    equals("0 42"),
    empty()
);

eval_and_assert!(
    enumerate_with_nested_destructuring,
    indoc! {r#"
        tuples = [(1, 2), (3, 4), (5, 6)];
        for idx, (a, b) in tuples.enumerate() {
            print(idx, a, b);
        };
    "#},
    equals("0 1 2 \n 1 3 4 \n 2 5 6"),
    empty()
);

eval_and_assert!(
    enumerate_to_list,
    indoc! {r#"
        list = ["x", "y", "z"];
        enumerated = [pair for pair in list.enumerate()];
        print(enumerated);
    "#},
    equals("[(0, \"x\"), (1, \"y\"), (2, \"z\")]"),
    empty()
);

eval_and_assert!(
    enumerate_with_break,
    indoc! {r#"
        for i, val in [1, 2, 3, 4, 5].enumerate() {
            break if i == 3;
            print(i, val);
        };
    "#},
    equals("0 1 \n 1 2 \n 2 3"),
    empty()
);

eval_and_assert!(
    enumerate_with_continue,
    indoc! {r#"
        for i, val in [10, 20, 30, 40].enumerate() {
            continue if i % 2 == 1;
            print(i, val);
        };
    "#},
    equals("0 10 \n 2 30"),
    empty()
);

eval_and_assert!(
    enumerate_starts_at_zero,
    indoc! {r#"
        first_idx = null;
        for i, _ in ["a", "b"].enumerate() {
            first_idx = i if first_idx == null;
        };
        print(first_idx);
    "#},
    equals("0"),
    empty()
);

eval_and_assert!(
    enumerate_mixed_types,
    indoc! {r#"
        mixed = [1, "two", true, null, [3]];
        for i, val in mixed.enumerate() {
            print(i, val);
        };
    "#},
    equals("0 1 \n 1 two \n 2 true \n 3 null \n 4 [3]"),
    empty()
);

eval_and_assert!(
    enumerate_with_numbers,
    indoc! {r#"
        numbers = [100, 200, 300];
        for idx, num in numbers.enumerate() {
            print(idx, num);
        };
    "#},
    equals("0 100 \n 1 200 \n 2 300"),
    empty()
);

eval_and_assert!(
    enumerate_string,
    indoc! {r#"
        for i, c in "abc".enumerate() {
            print(i, c);
        };
    "#},
    equals("0 a \n 1 b \n 2 c"),
    empty()
);

eval_and_assert!(
    enumerate_empty_string,
    indoc! {r#"
        for i, c in "".enumerate() {
            print(i, c);
        };
    "#},
    empty(),
    empty()
);
