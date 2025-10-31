use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    tuple_order_comparison_works,
    indoc! {r#"
        foo = (1, 2, 3);
        bar = (1, 2, 4);
        print(foo < bar);
        print(foo > bar);
        print(foo < foo);
        print(foo <= foo);
    "#},
    equals(indoc! {r#"
        true
        false
        false
        true
    "#}),
    empty()
);
