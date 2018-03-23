use prelude::v1::*;

extern crate diff;



pub struct JsonDiff<'a> {
    left: &'a ::serde_json::Value,
    right: &'a ::serde_json::Value,

    pub padding: Cow<'a, str>,
    pub mode: JsonDiffMode
}

#[derive(PartialEq, Clone, Copy)]
pub enum JsonDiffMode {
    All,
    DiffOnly
}

impl<'a> JsonDiff<'a> {
    pub fn new(left: &'a ::serde_json::Value, right: &'a ::serde_json::Value) -> Self {
        JsonDiff {
            left: left,
            right: right,
            padding: "".into(),
            mode: JsonDiffMode::All
        }
    }
}

impl<'a> ::std::fmt::Display for JsonDiff<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        let left = format!("{:#}", self.left);
        let right = format!("{:#}", self.right);

        for diff in diff::lines(&left, &right) {
            match diff {
                diff::Result::Left(l)    => {
                    writeln!(f, "{}-{}", self.padding, l)?;
                },
                diff::Result::Both(l, _) => {
                    if self.mode == JsonDiffMode::All {
                        writeln!(f, "{} {}", self.padding, l)?;
                    }
                },
                diff::Result::Right(r)   => {
                    writeln!(f, "{}+{}", self.padding, r)?
                }
            }
        }

        Ok(())
    }
}

