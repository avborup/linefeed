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

eval_and_assert!(
    list_order_comparison_works,
    indoc! {r#"
        foo = [1, 2, 3];
        bar = [1, 2, 4];
        print(foo < bar);
        print(foo > bar);
        print(foo < foo);
        print(foo <= foo);
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
    flat_nested_lists,
    indoc! {r#"
        result = [[1, 2], [3, 4], [5]].flat();
        print(result);
    "#},
    equals("[1, 2, 3, 4, 5]"),
    empty()
);

eval_and_assert!(
    flat_with_strings,
    indoc! {r#"
        result = ["ab", "cd"].flat();
        print(result);
    "#},
    equals("[\"a\", \"b\", \"c\", \"d\"]"),
    empty()
);

eval_and_assert!(
    flat_empty_list,
    indoc! {r#"
        result = [].flat();
        print(result);
    "#},
    equals("[]"),
    empty()
);

eval_and_assert!(
    flat_with_ranges,
    indoc! {r#"
        result = [0..3, 5..7].flat();
        print(result);
    "#},
    equals("[0, 1, 2, 5, 6]"),
    empty()
);

eval_and_assert!(
    list_first,
    r#"print([1, 2, 3].first());"#,
    equals("1"),
    empty()
);

eval_and_assert!(
    list_last,
    r#"print([1, 2, 3].last());"#,
    equals("3"),
    empty()
);

eval_and_assert!(
    range_first,
    r#"print((0..5).first());"#,
    equals("0"),
    empty()
);

eval_and_assert!(range_last, r#"print((0..5).last());"#, equals("4"), empty());

eval_and_assert!(range_len, r#"print((0..5).len());"#, equals("5"), empty());

eval_and_assert!(
    range_reverse_first,
    r#"print((5..0).first());"#,
    equals("5"),
    empty()
);

eval_and_assert!(
    range_reverse_last,
    r#"print((5..0).last());"#,
    equals("1"),
    empty()
);

eval_and_assert!(
    transpose_2x3,
    r#"print([[1,2,3],[4,5,6]].transpose());"#,
    equals("[[1, 4], [2, 5], [3, 6]]"),
    empty()
);

eval_and_assert!(
    transpose_3x2,
    r#"print([[1,2],[3,4],[5,6]].transpose());"#,
    equals("[[1, 3, 5], [2, 4, 6]]"),
    empty()
);

eval_and_assert!(
    transpose_empty,
    r#"print([].transpose());"#,
    equals("[]"),
    empty()
);
