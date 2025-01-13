use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    in_works_for_list,
    indoc! {r#"
        print(1 in [1, 2, 3]);
        print(0 in [1, 2, 3]);
        print(3 in [0, 2, 3]);
        print(1 in []);
    "#},
    equals(indoc! {r#"
        true
        false
        true
        false
    "#}),
    empty()
);

eval_and_assert!(
    in_works_for_range,
    indoc! {r#"
        print(1 in (1..3));
        print(2 in (1..3));
        print(0 in (1..3));
        print(3 in (1..3));
    "#},
    equals(indoc! {r#"
        true
        true
        false
        false
    "#}),
    empty()
);

eval_and_assert!(
    in_works_for_string,
    indoc! {r#"
        print("a" in "abc");
        print("d" in "abc");
        print("bc" in "abcd");
        print("a" in "");
    "#},
    equals(indoc! {r#"
        true
        false
        true
        false
    "#}),
    empty()
);

eval_and_assert!(
    in_works_for_tuple,
    indoc! {r#"
        print(1 in (1, 2, 3));
        print(0 in (1, 2, 3));
        print(3 in (0, 2, 3));
    "#},
    equals(indoc! {r#"
        true
        false
        true
    "#}),
    empty()
);

eval_and_assert!(
    in_works_inside_other_expressions,
    indoc! {r#"
        print(1 + 2 in [0, 3, 5]);

        visited, to_check = ([1, 2, 4], [2, 3, 5]);
        print([v in visited for v in to_check]);

        if 1 in [1, 2, 3] {
            print(true);
        };
    "#},
    equals(indoc! {r#"
        true
        [true, false, false]
        true
    "#}),
    empty()
);

// TODO:Dictionaries, sets
