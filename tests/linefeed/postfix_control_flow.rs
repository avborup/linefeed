use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    postfix_if_works,
    indoc! {r#"
        parity = |n| {
            return "neg" if n < 0;
            return "pos" if n > 0;
            "zero"
        };
        print(parity(0));
        print(parity(-1));
        print(parity(1));
    "#},
    equals(indoc! {r#"
        zero
        neg
        pos
    "#}),
    empty()
);

eval_and_assert!(
    postfix_unless_works,
    indoc! {r#"
        parity = |n| {
            return "neg" unless n >= 0;
            return "pos" unless n <= 0;
            "zero"
        };
        print(parity(0));
        print(parity(-1));
        print(parity(1));
    "#},
    equals(indoc! {r#"
        zero
        neg
        pos
    "#}),
    empty()
);
