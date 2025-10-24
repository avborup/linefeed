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

eval_and_assert!(
    index_destructuring_works,
    indoc! {r#"
        swp = ["tmp", "second"];
        swp[0] = "first";
        print(swp);

        swp[0], swp[1] = swp[1], swp[0];
        print(swp);
    "#},
    equals(indoc! {r#"
        ["first", "second"]
        ["second", "first"]
    "#}),
    empty()
);

eval_and_assert!(
    multi_index_destructuring_works,
    indoc! {r#"
        foo = [0, [1]];
        foo[0], foo[1][0] = ("top", "bottom");
        print(foo);
    "#},
    equals(indoc! {r#"
        ["top", ["bottom"]]
    "#}),
    empty()
);

eval_and_assert!(
    ident_index_destructuring_works,
    indoc! {r#"
        foo = [0, [1]];
        index1, index2 = 0, 1;
        foo[index1], foo[index2][index1] = ("top", "bottom");
        print(foo);
    "#},
    equals(indoc! {r#"
        ["top", ["bottom"]]
    "#}),
    empty()
);
