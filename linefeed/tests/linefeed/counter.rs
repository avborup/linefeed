use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    counter_can_be_created,
    indoc! {r#"
        counter = counter();
        counter.add("foo");
        print(counter);
    "#},
    equals(indoc! {r#"
        {"foo": 1}
    "#}),
    empty()
);

eval_and_assert!(
    counter_from_list,
    indoc! {r#"
        c = counter(["a", "b", "a", "c", "a"]);
        print(c);
    "#},
    equals(indoc! {r#"
        {"a": 3, "b": 1, "c": 1}
    "#}),
    empty()
);

eval_and_assert!(
    counter_from_string,
    indoc! {r#"
        c = counter("abacaba");
        print(c);
    "#},
    equals(indoc! {r#"
        {"a": 4, "b": 2, "c": 1}
    "#}),
    empty()
);

eval_and_assert!(
    counter_add_multiple_times,
    indoc! {r#"
        counter = counter();
        counter.add("foo");
        counter.add("bar");
        counter.add("foo");
        print(counter);
    "#},
    equals(indoc! {r#"
        {"bar": 1, "foo": 2}
    "#}),
    empty()
);

eval_and_assert!(
    counter_to_list,
    indoc! {r#"
        c = counter("aabbbc");
        print(list(c));
    "#},
    equals(indoc! {r#"
        [("a", 2), ("b", 3), ("c", 1)]
    "#}),
    empty()
);
