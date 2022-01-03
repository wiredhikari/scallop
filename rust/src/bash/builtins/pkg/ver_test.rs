use std::str::FromStr;

use pkgcraft::atom::Version;

use crate::bash::builtins::Builtin;
use crate::bash;
use crate::{Error, Result};

static LONG_DOC: &str = "\
Perform version testing as defined in the spec.

Returns 0 if the specified test is true, 1 otherwise.
Returns -1 if an error occurred.";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(args: &[&str]) -> Result<i32> {
    let pvr = bash::string_value("PVR").unwrap_or("");
    let (lhs, op, rhs) = match args.len() {
        2 if pvr.is_empty() => return Err(Error::new("$PVR is undefined")),
        2 => (pvr, args[0], args[1]),
        3 => (args[0], args[1], args[2]),
        n => return Err(Error::new(format!("only accepts 2 or 3 args, got {}", n))),
    };

    let ver_lhs = Version::from_str(lhs)?;
    let ver_rhs = Version::from_str(rhs)?;

    let ret = match op {
        "-eq" => ver_lhs == ver_rhs,
        "-ne" => ver_lhs != ver_rhs,
        "-lt" => ver_lhs < ver_rhs,
        "-gt" => ver_lhs > ver_rhs,
        "-le" => ver_lhs <= ver_rhs,
        "-ge" => ver_lhs >= ver_rhs,
        _ => return Err(Error::new(format!("invalid operator: {:?}", op))),
    };

    Ok(!ret as i32)
}

pub static BUILTIN: Builtin = Builtin {
    name: "ver_test",
    func: run,
    help: LONG_DOC,
    usage: "ver_test 1 -lt 2-r1",
};