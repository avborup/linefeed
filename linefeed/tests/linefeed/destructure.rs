use indoc::indoc;

use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

eval_and_assert!(
    ident_destructuring_works,
    indoc! {r#"
        inp = "1-3 a: abcde";

        reqs, password = inp.split(": ");
        minmax, char = reqs.split(" ");
        min, max = [int(x) for x in minmax.split("-")];

        print(reqs);
        print(password);
        print(minmax);
        print(char);
        print(min, max);
    "#},
    equals(indoc! {r#"
        1-3 a
        abcde
        1-3
        a
        1 3
    "#}),
    empty()
);

eval_and_assert!(
    destructuring_string_works,
    indoc! {r#"
        a, b, c = "123";
        print(a, b, c);
    "#},
    equals(indoc! {r#"
        1 2 3
    "#}),
    empty()
);

eval_and_assert!(
    nested_destructuring_works,
    indoc! {r#"
        foo, (bar, baz), qux = [1, (2, 3), 4];
        print(qux, baz, bar, foo);
    "#},
    equals("4 3 2 1"),
    empty()
);
