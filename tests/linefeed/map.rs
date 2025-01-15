use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    map_can_be_created,
    indoc! {r#"
        map = {};
        print(map);

        map = {1: 2};
        print(map);
    "#},
    equals(indoc! {r#"
        {}
        {1: 2}
    "#}),
    empty()
);

eval_and_assert!(
    map_can_be_accessed,
    indoc! {r#"
        map = {
            1: 2,
            "wow": "cool",
            (1, 2): -1,
        };

        print(map["wow"]);
        print(map[1]);
        print(map[(1, 2)]);
        print(map[3]);
    "#},
    equals(indoc! {r#"
        cool
        2
        -1
        null
    "#}),
    empty()
);

eval_and_assert!(
    map_can_be_updated,
    indoc! {r#"
        map = {
            1: 2,
            "wow": "cool",
            (1, 2): -1,
        };

        map["wow"] = "not cool";
        map[(1, 2)] = 0;

        print(map[1]);
        print(map["wow"]);
        print(map[(1, 2)]);
        print(map[3]);
    "#},
    equals(indoc! {r#"
        2
        not cool
        0
        null
    "#}),
    empty()
);

eval_and_assert!(
    map_can_be_iterated,
    indoc! {r#"
        map = { 1: 2 };

        for kv in map {
            print(kv);
        };
    "#},
    equals("(1, 2)"),
    empty()
);
