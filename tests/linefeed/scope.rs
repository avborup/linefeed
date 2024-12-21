use crate::helpers::eval_and_assert;

use indoc::indoc;

eval_and_assert!(
    scope_success,
    include_str!("../scope.lf"),
    indoc! {r#"
        outer
        overwritten
        overwritten
    "#},
    ""
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
    indoc! {r#"
        outer
        inner
    "#},
    ""
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
    indoc! {r#"
        0
        2
        0
    "#},
    ""
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
    "some value",
    ""
);
