use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
};

eval_and_assert!(
    match_works,
    indoc! {r#"
        fn foo(n) {
            match n {
                0 => "zero",
                1 => "one",
            }
        };

        print(foo(1));
        print(foo(0));
    "#},
    equals(indoc! {r#"
        one
        zero
    "#}),
    empty()
);

eval_and_assert!(
    match_errors_on_no_match,
    indoc! {r#"
        fn foo(n) {
            match n {
                0 => "zero",
                1 => "one",
            }
        };

        print(foo(1));
        print(foo(2));
    "#},
    equals("one"),
    contains("Error: No arm matched the valu")
);

eval_and_assert!(
    match_works_with_different_types,
    indoc! {r#"
        fn foo(x) {
            match x {
                1 => "one",
                "two" => 2,
                true => false,
                null => "nil",
            }
        };

        print(foo(1));
        print(foo("two"));
        print(foo(true));
        print(foo(null));
    "#},
    equals(indoc! {r#"
        one
        2
        false
        nil
    "#}),
    empty()
);
