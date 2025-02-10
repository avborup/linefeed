use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    function_oneliners,
    include_str!("../functions.lf"),
    equals(indoc! {r#"
        2
        2
        yes
        hello
        3
    "#}),
    empty()
);

eval_and_assert!(
    recursive_fibonacci,
    indoc! {r#"
        fn fib(n) {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        };
        print(fib(4));
        print(fib(5));
        print(fib(6));
    "#},
    equals(indoc! {r#"
        3
        5
        8
    "#}),
    empty()
);
