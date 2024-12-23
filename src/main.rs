fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(filename).unwrap();
    linefeed::compile_and_run(src);
}
