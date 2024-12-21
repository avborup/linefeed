use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    return_,
    include_str!("../return.lf"),
    equals(indoc! {r#"
        damn you hot boy
        you aint hot boy
    "#}),
    empty()
);

eval_and_assert!(
    return_evaluates_nothing_after,
    indoc! {r#"
        foo = || {
            return print("this is printed");
            print("this is not printed");
        };

        foo();
    "#},
    equals(indoc! {r#"
        this is printed
    "#}),
    empty()
);

