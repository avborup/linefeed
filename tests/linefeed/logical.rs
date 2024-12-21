use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    and,
    indoc::indoc! {r#"
        print(true and true);
        print(true and false);
        print(false and true);
        print(false and false);
    "#},
    equals(indoc! {r#"
        true
        false
        false
        false
    "#}),
    empty()
);

eval_and_assert!(
    or,
    indoc::indoc! {r#"
        print(true or true);
        print(true or false);
        print(false or true);
        print(false or false);
    "#},
    equals(indoc! {r#"
        true
        true
        true
        false
    "#}),
    empty()
);

eval_and_assert!(
    not,
    indoc::indoc! {r#"
        print(not true);
        print(not false);
    "#},
    equals(indoc! {r#"
        false
        true
    "#}),
    empty()
);

eval_and_assert!(
    not_multiple,
    indoc::indoc! {r#"
        print(not not true);
        print(not not false);
        print(not not not true);
        print(not not not false);
    "#},
    equals(indoc! {r#"
        true
        false
        false
        true
    "#}),
    empty()
);

eval_and_assert!(
    logical_op_precedence,
    indoc::indoc! {r#"
        print(true and true or false);
        print(false and true or false);
        print(false or true and false);
        print(true or false and true);
    "#},
    equals(indoc! {r#"
        true
        false
        false
        false
    "#}),
    empty()
);
