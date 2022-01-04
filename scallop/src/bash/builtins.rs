use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::os::raw::{c_char, c_int};
use std::{mem, ptr};

use once_cell::sync::Lazy;

use crate::bash::bindings;
use crate::bash::IntoVec;
use crate::{bash, Result};

pub mod profile;

// pkgcraft specific builtins
#[cfg(feature = "pkgcraft")]
pub mod pkg;
// export pkgcraft builtins
#[cfg(feature = "pkgcraft")]
pub use pkg::*;

type BuiltinFn = fn(&[&str]) -> Result<i32>;

#[derive(Clone, Copy)]
pub struct Builtin {
    pub name: &'static str,
    pub func: BuiltinFn,
    pub help: &'static str,
    pub usage: &'static str,
}

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builtin").field("name", &self.name).finish()
    }
}

impl PartialEq for Builtin {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Builtin {}

impl Hash for Builtin {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Builtin {
    #[inline]
    pub fn run(self, args: &[&str]) -> Result<i32> {
        (self.func)(args)
    }
}

/// Convert a Builtin to its C equivalent.
impl From<Builtin> for bindings::Builtin {
    fn from(builtin: Builtin) -> bindings::Builtin {
        let name_str = CString::new(builtin.name).unwrap();
        let name = name_str.as_ptr();
        mem::forget(name_str);

        let short_doc_str = CString::new(builtin.usage).unwrap();
        let short_doc = short_doc_str.as_ptr();
        mem::forget(short_doc_str);

        let long_doc_str: Vec<CString> = builtin
            .help
            .split('\n')
            .map(|s| CString::new(s).unwrap())
            .collect();
        let mut long_doc_ptr: Vec<*const c_char> =
            long_doc_str.iter().map(|s| s.as_ptr()).collect();
        long_doc_ptr.push(ptr::null());
        let long_doc = long_doc_ptr.as_ptr();
        mem::forget(long_doc_str);
        mem::forget(long_doc_ptr);

        bindings::Builtin {
            name,
            function: run,
            flags: 1,
            long_doc,
            short_doc,
            handle: ptr::null_mut(),
        }
    }
}

static BUILTINS: Lazy<HashMap<&'static str, &'static Builtin>> = Lazy::new(|| {
    let mut builtins: Vec<&Builtin> = vec![&profile::BUILTIN];
    if cfg!(feature = "pkgcraft") {
        builtins.extend([
            &pkg::has::BUILTIN,
            &pkg::hasv::BUILTIN,
            &pkg::ver_cut::BUILTIN,
            &pkg::ver_rs::BUILTIN,
            &pkg::ver_test::BUILTIN,
        ]);
    }

    builtins.iter().map(|&b| (b.name, b)).collect()
});

/// Builtin function wrapper converting between rust and C types.
///
/// # Safety
/// This should only be used when registering an external rust bash builtin.
#[no_mangle]
pub(crate) unsafe extern "C" fn run(list: *mut bindings::WordList) -> c_int {
    // get the current running command name
    let cmd = bash::current_command();
    // find its matching rust function and execute it
    let builtin = BUILTINS.get(cmd).unwrap();
    let args = unsafe { list.into_vec().unwrap() };

    match builtin.run(args.as_slice()) {
        Ok(ret) => ret,
        Err(e) => {
            eprintln!("{}: error: {}", cmd, e);
            -1
        }
    }
}