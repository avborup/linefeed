use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
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

// TODO: Consider if this would actually be a neat feature. E.g. to early-exit the program.
eval_and_assert!(
    return_at_top_level_shows_error,
    indoc! {r#"
        return;
    "#},
    empty(),
    contains("Illegal return outside of function")
);
