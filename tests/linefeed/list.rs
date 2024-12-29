use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
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
