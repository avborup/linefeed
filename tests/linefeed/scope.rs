use crate::helpers::{
    eval_and_assert,
    output::{contains, equals},
};

use indoc::indoc;

eval_and_assert!(
    scope_success,
    include_str!("../scope.lf"),
    equals(indoc! {r#"
        outer
        overwritten
        overwritten
    "#}),
    equals("")
);

eval_and_assert!(
    scope_block_overwrites_outer,
    indoc! {r#"
        outer = "outer";
        {
            print(outer); # outer
            outer = "inner";
        };
        print(outer); # inner
    "#},
    equals(indoc! {r#"
        outer
        inner
    "#}),
    equals("")
);

eval_and_assert!(
    scope_function_is_local,
    indoc! {r#"
        x = 0;

        foo = |x| {
            x = x + 1;
            print(x);
        };

        print(x); # 0
        foo(1); # 2
        print(x); # 0
    "#},
    equals(indoc! {r#"
        0
        2
        0
    "#}),
    equals("")
);

eval_and_assert!(
    scope_function_is_local_and_yields_error,
    indoc! {r#"
        foo = || {
            x = "some value";
            print(x);
        };

        foo();
        print(x); # error
    "#},
    equals("some value"),
    contains("Error: No such variable 'x' in scope")
);
