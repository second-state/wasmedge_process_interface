//! How to use this crate
//! # Adding this as a dependency
//! ```rust, ignore
//! [dependencies]
//! rust_process_interface_library = "^0.1"
//! ```
//!
//! # Bringing this into scope
//! ```rust, ignore
//! use rust_process_interface_library::Command;
//! ```
//! # Tests
//! ```bash, ignore
//! cargo test --lib
//! ```

use std::ffi::CString;

/// The output of a finished process.
///
/// This is returned in a Result by the [`output`] method of a [`Command`].
pub struct Output {
    /// The status (exit code) of the process.
    pub status: i32,
    /// The data that the process wrote to stdout.
    pub stdout: Vec<u8>,
    /// The data that the process wrote to stderr.
    pub stderr: Vec<u8>,
}

pub mod ssvm_process {
    use std::os::raw::c_char;
    #[link(wasm_import_module = "ssvm_process")]
    extern "C" {
        pub fn ssvm_process_set_prog_name(name: *const c_char, len: u32);
        pub fn ssvm_process_set_arg(arg: *const c_char, len: u32);
        pub fn ssvm_process_set_env(
            env: *const c_char,
            env_len: u32,
            val: *const c_char,
            val_len: u32,
        );
        pub fn ssvm_process_set_stdin(buf: *const c_char, len: u32);
        pub fn ssvm_process_set_timeout(time_ms: u32);
        pub fn ssvm_process_run() -> i32;
        pub fn ssvm_process_get_exit_code() -> i32;
        pub fn ssvm_process_get_stdout_len() -> u32;
        pub fn ssvm_process_get_stdout(buf: *mut u8);
        pub fn ssvm_process_get_stderr_len() -> u32;
        pub fn ssvm_process_get_stderr(buf: *mut u8);
    }
}

pub struct Command {}

impl Command {
    pub fn new<S: AsRef<str>>(prog: S) -> Command {
        let cprog = CString::new(prog.as_ref()).expect("");
        unsafe {
            ssvm_process::ssvm_process_set_prog_name(cprog.as_ptr(), cprog.as_bytes().len() as u32);
        }
        Command {}
    }

    pub fn arg<S: AsRef<str>>(&mut self, arg: S) -> &mut Command {
        let carg = CString::new(arg.as_ref()).expect("");
        unsafe {
            ssvm_process::ssvm_process_set_arg(carg.as_ptr(), carg.as_bytes().len() as u32);
        }
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.arg(arg.as_ref());
        }
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Command
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let ckey = CString::new(key.as_ref()).expect("");
        let cval = CString::new(val.as_ref()).expect("");
        unsafe {
            ssvm_process::ssvm_process_set_env(
                ckey.as_ptr(),
                ckey.as_bytes().len() as u32,
                cval.as_ptr(),
                cval.as_bytes().len() as u32,
            );
        }
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Command
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (ref key, ref val) in vars {
            self.env(key.as_ref(), val.as_ref());
        }
        self
    }

    pub fn stdin<S: AsRef<str>>(&mut self, buf: S) -> &mut Command {
        let cbuf = CString::new(buf.as_ref()).expect("");
        unsafe {
            ssvm_process::ssvm_process_set_stdin(cbuf.as_ptr(), cbuf.as_bytes().len() as u32);
        }
        self
    }

    pub fn timeout(&mut self, time: u32) -> &mut Command {
        unsafe {
            ssvm_process::ssvm_process_set_timeout(time);
        }
        self
    }

    pub fn output(&mut self) -> Output {
        unsafe {
            let exit_code = ssvm_process::ssvm_process_run();
            let stdout_len = ssvm_process::ssvm_process_get_stdout_len();
            let stderr_len = ssvm_process::ssvm_process_get_stderr_len();
            let mut stdout_vec: Vec<u8> = vec![0; stdout_len as usize];
            let mut stderr_vec: Vec<u8> = vec![0; stderr_len as usize];
            let stdout_ptr = stdout_vec.as_mut_ptr();
            let stderr_ptr = stderr_vec.as_mut_ptr();
            ssvm_process::ssvm_process_get_stdout(stdout_ptr);
            ssvm_process::ssvm_process_get_stderr(stderr_ptr);

            Output {
                status: exit_code,
                stdout: stdout_vec,
                stderr: stderr_vec,
            }
        }
    }
}
