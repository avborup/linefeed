use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    string_upper_and_lower,
    indoc! {r#"
        my_str = "Hi Mom!";
        print(my_str.upper());
        print(my_str.lower());
        print(my_str);
    "#},
    equals(indoc! {r#"
        HI MOM!
        hi mom!
        Hi Mom!
    "#}),
    empty()
);

eval_and_assert!(
    chained_string_upper_and_lower,
    indoc! {r#"
        print("Hi Mom!".upper().lower());
    "#},
    equals("hi mom!"),
    empty()
);
