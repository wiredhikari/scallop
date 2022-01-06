use std::sync::atomic::Ordering;

use super::NONFATAL;
use crate::builtins::Builtin;
use crate::{Error, Result};

static LONG_DOC: &str = "\
Displays a failure message provided in an optional argument and then aborts the build process.";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(args: &[&str]) -> Result<i32> {
    // handle nonfatal support
    let args = match args.len() {
        // TODO: check for EAPI support
        1 | 2 if args[0] == "-n" => {
            let nonfatal = NONFATAL.load(Ordering::Relaxed);
            if nonfatal {
                if args.len() == 2 {
                    println!("nonfatal die");
                    eprintln!("{}", args[1]);
                }
                return Ok(1);
            }
            &args[1..]
        }
        0 | 1 => args,
        n => return Err(Error::new(format!("takes up to 1 arg, got {}", n))),
    };

    if !args.is_empty() {
        eprintln!("{}", args[0]);
    }

    // TODO: output bash backtrace

    // TODO: this should probably call the exit builtin
    std::process::exit(1)
}

pub static BUILTIN: Builtin = Builtin {
    name: "die",
    func: run,
    help: LONG_DOC,
    usage: "die \"error message\"",
    exit_on_error: false,
};
