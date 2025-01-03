use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
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
    empty()
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
    empty()
);

eval_and_assert!(
    scope_function_is_local,
    indoc! {r#"
        x = 0;

        fn foo(x) {
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
    empty()
);

eval_and_assert!(
    scope_function_is_local_and_yields_error,
    indoc! {r#"
        fn foo() {
            x = "some value";
            print(x);
        };

        foo();
        print(x); # error
    "#},
    empty(), // compilation fails, so no output
    contains("Error: No such variable 'x' in scope")
);

eval_and_assert!(
    overwrite_variable_before_assignment_yields_error,
    indoc! {r#"
        x = x + 1; # error
    "#},
    empty(), // compilation fails, so no output
    contains("Error: No such variable 'x' in scope")
);
