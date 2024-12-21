use std::fmt::Display;

#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

pub enum OutputAssertion {
    Equals(String),
    Contains(String),
}

pub enum OutputSource {
    StdOut,
    StdErr,
}

impl Output {
    pub fn assert(&self, source: OutputSource, assertion: OutputAssertion) -> &Self {
        let source_label = match source {
            OutputSource::StdOut => "stdout",
            OutputSource::StdErr => "stderr",
        };

        let actual = match source {
            OutputSource::StdOut => &self.stdout,
            OutputSource::StdErr => &self.stderr,
        };

        match assertion {
            OutputAssertion::Equals(expected) => {
                assert_eq!(
                    normalise_string(actual),
                    normalise_string(&expected),
                    "{}",
                    format_failure(source_label, "equals", expected, actual)
                );
            }
            OutputAssertion::Contains(expected) => {
                assert!(
                    normalise_string(actual).contains(&normalise_string(&expected)),
                    "{}",
                    format_failure(source_label, "contains", expected, actual)
                );
            }
        }

        self
    }
}

fn format_failure(name: &str, kind: &str, expected: impl Display, actual: impl Display) -> String {
    format!(
        "Expected and actual {} differ:\n\n===== EXPECTED ({kind}) =====\n{}\n\n===== ACTUAL =====\n{}\n\n===== END =====\n",
        name, expected, actual
    )
}

fn normalise_string(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let without_ansi_escapes = String::from_utf8(strip_ansi_escapes::strip(s)).unwrap();

    without_ansi_escapes
        .trim()
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn contains(expected: impl Into<String>) -> OutputAssertion {
    OutputAssertion::Contains(expected.into())
}

pub fn equals(expected: impl Into<String>) -> OutputAssertion {
    OutputAssertion::Equals(expected.into())
}
