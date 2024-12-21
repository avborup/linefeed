macro_rules! assert_eq_normalised {
    ($expected:expr, $actual:expr) => {
        let expected = $expected.trim();
        let actual = $actual.trim();
        assert_eq!(expected, actual);
    };
}

macro_rules! eval_and_assert {
    ($name:ident, $src:expr, $expected:expr) => {
        #[test]
        fn $name() -> () {
            let mut sink = Vec::new();
            linefeed::run_with_output($src, &mut sink);
            let res_str = std::str::from_utf8(&sink).unwrap();
            assert_eq_normalised!(res_str, $expected);
        }
    };
}

eval_and_assert!(test_factorial, include_str!("factorial.lf"), "3628800");

eval_and_assert!(
    test_function_oneliners,
    include_str!("functions.lf"),
    "2\n2\nyes"
);
