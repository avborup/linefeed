use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    append_called_repeatedly,
    indoc! {r#"
        list = [1, 2];
        list.append(3).append(4);
        print(list.append(5));
    "#},
    equals("[1, 2, 3, 4, 5]"),
    empty()
);

eval_and_assert!(
    shared_access_to_list,
    indoc! {r#"
        foo = [1, 2];
        bar = foo;
        foo.append(3);
        bar.append(4);
        print(bar);
    "#},
    equals("[1, 2, 3, 4]"),
    empty()
);

eval_and_assert!(
    append_fails_on_no_argument,
    indoc! {r#"
        foo = [1, 2];
        foo.append();
    "#},
    empty(),
    contains("Method append expects 1 arguments, but got 0")
);

eval_and_assert!(
    index_assign_works,
    indoc! {r#"
        foo = [1, 2, 3];
        foo[1] = "new";
        print(foo);
    "#},
    equals("[1, \"new\", 3]"),
    empty()
);

eval_and_assert!(
    index_assign_works_with_inline_expr,
    indoc! {r#"
        foo = [1, 2, 3];
        foo[foo.len() / 2] = "mellon";
        print(foo);

        foo = ["first", "second", "third"];
        foo[foo.len() - foo.len() - 1] = foo[1] + "-mellon";
        print(foo);
    "#},
    equals(indoc! {r#"
        [1, "mellon", 3]
        ["first", "second", "second-mellon"]
    "#}),
    empty()
);
