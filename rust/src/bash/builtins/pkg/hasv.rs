use crate::bash::builtins::Builtin;
use super::has;
use crate::Result;

static LONG_DOC: &str = "The same as has, but also prints the first argument if found.";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(args: &[&str]) -> Result<i32> {
    let ret = has::run(args)?;
    if ret == 0 {
        println!("{}", args[0]);
    }

    Ok(ret)
}

pub static BUILTIN: Builtin = Builtin {
    name: "hasv",
    func: run,
    help: LONG_DOC,
    usage: "hasv needle ${haystack}",
};