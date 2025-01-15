use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    all_works,
    indoc! {r#"
        # Lists
        print(all([true, true, true]));
        print(all([true, false, true]));
        print(all([false, false]));
        print();

        # Multiple args
        print(all(true, true, true));
        print(all(true, false, true));
        print(all(false));
    "#},
    equals(indoc! {r#"
        true
        false
        false

        true
        false
        false
    "#}),
    empty()
);

eval_and_assert!(
    any_works,
    indoc! {r#"
        # Lists
        print(any([true, true, true]));
        print(any([true, false, true]));
        print(any([false, false]));
        print();

        # Multiple args
        print(any(true, true, true));
        print(any(true, false, true));
        print(any(false));
    "#},
    equals(indoc! {r#"
        true
        true
        false

        true
        true
        false
    "#}),
    empty()
);
