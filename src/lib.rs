//! How to use this crate
//! # Adding this as a dependency
//! ```rust, ignore
//! [dependencies]
//! wasmedge_process_interface = "^0.2.1"
//! ```
//!
//! # Bringing this into scope
//! ```rust, ignore
//! use wasmedge_process_interface::Command;
//! ```
//! # Tests
//! ```bash, ignore
//! cargo test --lib
//! ```

use std::collections::HashMap;
use std::env;
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

pub mod wasmedge_process {
    use std::os::raw::c_char;
    #[link(wasm_import_module = "wasmedge_process")]
    extern "C" {
        pub fn wasmedge_process_set_prog_name(name: *const c_char, len: u32);
        pub fn wasmedge_process_add_arg(arg: *const c_char, len: u32);
        pub fn wasmedge_process_add_env(
            env: *const c_char,
            env_len: u32,
            val: *const c_char,
            val_len: u32,
        );
        pub fn wasmedge_process_add_stdin(buf: *const c_char, len: u32);
        pub fn wasmedge_process_set_timeout(time_ms: u32);
        pub fn wasmedge_process_run() -> i32;
        pub fn wasmedge_process_get_exit_code() -> i32;
        pub fn wasmedge_process_get_stdout_len() -> u32;
        pub fn wasmedge_process_get_stdout(buf: *mut u8);
        pub fn wasmedge_process_get_stderr_len() -> u32;
        pub fn wasmedge_process_get_stderr(buf: *mut u8);
    }
}

pub struct Command {
    /// The program name.
    pub name: String,
    /// The argument list.
    pub args_list: Vec<String>,
    /// The environment map.
    pub envp_map: HashMap<String, String>,
    /// The timeout value (milliseconds).
    pub timeout_val: u32,
    /// Buffered stdin.
    pub stdin_str: Vec<u8>,
}

impl Command {
    pub fn new<S: AsRef<str>>(prog: S) -> Command {
        let mut envp: HashMap<String, String> = HashMap::new();
        for (key, value) in env::vars() {
            envp.insert(key, value);
        }
        Command {
            name: String::from(prog.as_ref()),
            args_list: vec![],
            envp_map: envp,
            timeout_val: 10000,
            stdin_str: vec![],
        }
    }

    pub fn arg<S: AsRef<str>>(&mut self, arg: S) -> &mut Command {
        self.args_list.push(String::from(arg.as_ref()));
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

    pub fn args_clear(&mut self) -> &mut Command {
        self.args_list.clear();
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Command
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.envp_map
            .insert(String::from(key.as_ref()), String::from(val.as_ref()));
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
        self.stdin_str
            .extend(CString::new(buf.as_ref()).expect("").as_bytes());
        self
    }

    pub fn stdin_u8(&mut self, buf: u8) -> &mut Command {
        self.stdin_str.push(buf);
        self
    }

    pub fn stdin_u8vec<S: AsRef<[u8]>>(&mut self, buf: S) -> &mut Command {
        self.stdin_str.extend(buf.as_ref());
        self
    }

    pub fn timeout(&mut self, time: u32) -> &mut Command {
        self.timeout_val = time;
        self
    }

    pub fn output(&mut self) -> Output {
        let cprog = CString::new((&self.name).as_bytes()).expect("");
        // Set program name.
        unsafe {
            wasmedge_process::wasmedge_process_set_prog_name(
                cprog.as_ptr(),
                cprog.as_bytes().len() as u32,
            );
        }

        // Set arguments.
        for arg in &self.args_list {
            let carg = CString::new(arg.as_bytes()).expect("");
            unsafe {
                wasmedge_process::wasmedge_process_add_arg(
                    carg.as_ptr(),
                    carg.as_bytes().len() as u32,
                );
            }
        }

        // Set environments.
        for (key, val) in &self.envp_map {
            let ckey = CString::new(key.as_bytes()).expect("");
            let cval = CString::new(val.as_bytes()).expect("");
            unsafe {
                wasmedge_process::wasmedge_process_add_env(
                    ckey.as_ptr(),
                    ckey.as_bytes().len() as u32,
                    cval.as_ptr(),
                    cval.as_bytes().len() as u32,
                );
            }
        }

        let exit_code: i32;
        let stdout_len: u32;
        let stderr_len: u32;
        unsafe {
            // Set timeout.
            wasmedge_process::wasmedge_process_set_timeout(self.timeout_val);

            // Set stdin.
            wasmedge_process::wasmedge_process_add_stdin(
                self.stdin_str.as_ptr() as *const i8,
                self.stdin_str.len() as u32,
            );

            // Run.
            exit_code = wasmedge_process::wasmedge_process_run();

            // Get outputs.
            stdout_len = wasmedge_process::wasmedge_process_get_stdout_len();
            stderr_len = wasmedge_process::wasmedge_process_get_stderr_len();
        }
        let mut stdout_vec: Vec<u8> = vec![0; stdout_len as usize];
        let mut stderr_vec: Vec<u8> = vec![0; stderr_len as usize];
        let stdout_ptr = stdout_vec.as_mut_ptr();
        let stderr_ptr = stderr_vec.as_mut_ptr();
        unsafe {
            wasmedge_process::wasmedge_process_get_stdout(stdout_ptr);
            wasmedge_process::wasmedge_process_get_stderr(stderr_ptr);
        }

        Output {
            status: exit_code,
            stdout: stdout_vec,
            stderr: stderr_vec,
        }
    }
}

// Test
// Please use the following command so that the print statements are shown during testing
// cargo test -- --nocapture
//

#[cfg(test)]
mod tests {
    use super::Command;
    #[test]
    fn test_arg() {
        let mut cmd = Command::new("rusttest");
        cmd.arg("val1").arg("val2");
        assert_eq!(cmd.args_list[0], "val1");
        assert_eq!(cmd.args_list[1], "val2");
    }
    #[test]
    fn test_args() {
        let mut cmd = Command::new("rusttest");
        cmd.args(&["val1", "val2"]);
        assert_eq!(cmd.args_list[0], "val1");
        assert_eq!(cmd.args_list[1], "val2");
    }
    #[test]
    fn test_arg_args() {
        let mut cmd = Command::new("rusttest");
        cmd.arg("val1").arg("val2").args(&["val3", "val4"]);
        assert_eq!(cmd.args_list[0], "val1");
        assert_eq!(cmd.args_list[1], "val2");
        assert_eq!(cmd.args_list[2], "val3");
        assert_eq!(cmd.args_list[3], "val4");
    }
    #[test]
    fn test_args_clear() {
        let mut cmd = Command::new("rusttest");
        cmd.arg("val1").arg("val2").args(&["val3", "val4"]);
        cmd.args_clear();
        assert_eq!(cmd.args_list.len(), 0);
    }
    #[test]
    fn test_env() {
        let mut cmd = Command::new("rusttest");
        cmd.env("ENV1", "VALUE1").env("ENV2", "VALUE2");
        assert_eq!(cmd.envp_map["ENV1"], "VALUE1");
        assert_eq!(cmd.envp_map["ENV2"], "VALUE2");
    }
    #[test]
    fn test_envs() {
        use std::collections::HashMap;
        let mut cmd = Command::new("rusttest");
        let mut hash: HashMap<String, String> = HashMap::new();
        hash.insert(String::from("ENV1"), String::from("VALUE1"));
        hash.insert(String::from("ENV2"), String::from("VALUE2"));
        cmd.envs(hash);
        assert_eq!(cmd.envp_map["ENV1"], "VALUE1");
        assert_eq!(cmd.envp_map["ENV2"], "VALUE2");
    }
    #[test]
    fn test_env_envs() {
        use std::collections::HashMap;
        let mut cmd = Command::new("rusttest");
        let mut hash: HashMap<String, String> = HashMap::new();
        hash.insert(String::from("ENV1"), String::from("VALUE1"));
        hash.insert(String::from("ENV2"), String::from("VALUE2"));
        cmd.env("ENV3", "VALUE3").env("ENV4", "VALUE4").envs(hash);
        assert_eq!(cmd.envp_map["ENV1"], "VALUE1");
        assert_eq!(cmd.envp_map["ENV2"], "VALUE2");
        assert_eq!(cmd.envp_map["ENV3"], "VALUE3");
        assert_eq!(cmd.envp_map["ENV4"], "VALUE4");
    }
    #[test]
    fn test_stdin() {
        use std::str;
        let mut cmd = Command::new("rusttest");
        cmd.stdin("hello").stdin(" ").stdin("world");
        assert_eq!(
            str::from_utf8(&cmd.stdin_str).expect("ERROR"),
            "hello world"
        );
    }
    #[test]
    fn test_stdin_u8() {
        let mut cmd = Command::new("rusttest");
        cmd.stdin("Test").stdin_u8(0).stdin_u8(100).stdin_u8(255);
        assert_eq!(cmd.stdin_str, vec![84, 101, 115, 116, 0, 100, 255]);
    }
    #[test]
    fn test_stdin_u8vec() {
        let mut cmd = Command::new("rusttest");
        let v = vec![5, 6, 7];
        cmd.stdin("Test")
            .stdin_u8vec(&v)
            .stdin_u8(100)
            .stdin_u8(255);
        assert_eq!(cmd.stdin_str, vec![84, 101, 115, 116, 5, 6, 7, 100, 255]);
        assert_eq!(v, vec![5, 6, 7]);
    }
    #[test]
    fn test_timeout() {
        let mut cmd = Command::new("rusttest");
        cmd.timeout(666666);
        assert_eq!(cmd.timeout_val, 666666);
    }
}
