# This Repository Is Deprecated

This library has been renamed and moved to `wasmedge_process_interface`. Please follow the resulting [new wasmedge_process_interface](https://crates.io/crates/wasmedge_process_interface) crate for further development. No further development will take place in this repository.

# SSVM Process Interface

A Rust library that provides Rust to WebAssembly developers with syntax for running commands functionality when their Wasm is being executed on [SecondState's SSVM](https://github.com/second-state/SSVM).

From a high-level overview here, we are essentially building a process interface that will allow the native operating system (which SSVM is running on) to play a part in the runtime execution. Specifically, play a part in executing commands with arguments and environment values as part of Wasm execution. 

# How to use this library

## Rust dependency

Developers will add the [`ssvm_process_interface` crate](https://crates.io/crates/ssvm_process_interface) as a dependency to their `Rust -> Wasm` applications. For example, add the following line to the application's `Cargo.toml` file.
```
[dependencies]
ssvm_process_interface = "^0.1.3"
```

Developers will bring the `Command` modules of `ssvm_process_interface` into scope within their `Rust -> Wasm` application's code. For example, adding the following code to the top of their `main.rs` file. 
```
use ssvm_process_interface::Command;
```

## Execute commands with program name

Developers can then use syntax, such as the following, to execute commands such as using [`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html). After compilation, the output target Wasm file will contain imports of host functions about running external commands.

### Create a Command object
```rust
let mut cmd = Command::new("ls");
```
### Append arguments
```rust
let mut cmd = Command::new("ls");
cmd.arg("-al");
```
Or the following:
```rust
let cmd = Command::new("ls").arg("-al");
```
Or the following:
```rust
let cmd = Command::new("ls").arg("-alF").arg("..");
```
Or the following:
```rust
let cmd = Command::new("ls").args(&["-alF", ".."]);
```
### Append environment variables
```rust
let mut cmd = Command::new("printenv").arg("ONE").env("ONE", "1");
```
Or the following:
```rust
use std::collections::HashMap;
let mut cmd = Command::new("rusttest");
let mut hash: HashMap<String, String> = HashMap::new();
hash.insert(String::from("ENV1"), String::from("VALUE1"));
hash.insert(String::from("ENV2"), String::from("VALUE2"));
let mut cmd = Command::new("printenv").arg("ENV1").envs(hash);
```
### Append `stdin`
```rust
let mut cmd = Command::new("python3").stdin("print(\"HELLO PYTHON\")");
```
Or the following:
```rust
// Consider about the `\n` charactor in stdin strings.
let mut cmd = Command::new("python3").stdin("import time\n").stdin("print(time.time())");
```
### Specify execution timeout
```rust
// Timeout values are in milliseconds.
let mut cmd = Command::new("python3")
              .stdin("from time import sleep\n")
              .stdin("print('PYTHON start sleep 2s', flush=True)\n")
              .stdin("sleep(2)\n")
              .stdin("print('PYTHON end sleep 2s', flush=True)\n")
              .timeout(1000);
```
### Execution and get outputs

Please remember to check for the return status of the child process.

```rust
let out = Command::new("python3")
          .stdin("from time import sleep\n")
          .stdin("import sys\n")
          .stdin("print('stdout: PYTHON start sleep 2s', flush=True)\n")
          .stdin("print('stderr: PYTHON start sleep 2s', file=sys.stderr, flush=True)\n")
          .stdin("sleep(2)\n")
          .stdin("print('stdout: PYTHON end sleep 2s', flush=True)\n")
          .stdin("print('stderr: PYTHON end sleep 2s', file=sys.stderr, flush=True)\n")
          .timeout(1000)
          .output();

println!(" return code : {}", out.status);
println!(" stdout :");
print!("{}", str::from_utf8(&out.stdout).expect("GET STDOUT ERR"));
println!(" stderr :");
print!("{}", str::from_utf8(&out.stderr).expect("GET STDERR ERR"));
```

# Crates.io

The official crate is available at [crates.io](https://crates.io/crates/ssvm_process_interface).
