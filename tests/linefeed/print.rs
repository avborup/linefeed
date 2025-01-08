use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    print_works,
    indoc! {r#"
        print("Hello, world!");
        print("Goodbye, world!");
    "#},
    equals(indoc! {r#"
        Hello, world!
        Goodbye, world!
    "#}),
    empty()
);

eval_and_assert!(
    print_expr_works,
    indoc! {r#"
        print(1 + 2);
        print(3 * 4);
    "#},
    equals(indoc! {r#"
        3
        12
    "#}),
    empty()
);

eval_and_assert!(
    print_multiple_works,
    indoc! {r#"
        print("Hello, world!", "Goodbye, world!");
    "#},
    equals(indoc! {r#"
        Hello, world! Goodbye, world!
    "#}),
    empty()
);

eval_and_assert!(
    print_no_args_works,
    indoc! {r#"
        print();
    "#},
    equals(""),
    empty()
);

eval_and_assert!(
    print_equals_last_argument,
    indoc! {r#"
        res = print(1, 2, 3);
        print(res);

        res_null = print();
        print(res_null);
    "#},
    equals(indoc! {r#"
        1 2 3
        3

        null
    "#}),
    empty()
);
