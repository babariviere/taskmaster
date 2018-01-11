//! Command parse

use libc::{self, c_char};
use std::env;
use std::ffi::CString;
use std::path::Path;
use std::ptr;

/// Split ident
fn split_ident(s: &str) -> Vec<String> {
    let mut strs = Vec::new();
    let mut holder = String::new();
    let mut tok = '\0';
    let mut prev = '\0';
    for c in s.chars() {
        match c {
            '\'' | '\"' => {
                if tok == c {
                    strs.push(holder);
                    holder = String::new();
                } else {
                    tok = c;
                }
            }
            c if c.is_whitespace() && tok == '\0' && !prev.is_whitespace() => {
                strs.push(holder);
                holder = String::new();
            }
            _ => holder.push(c),
        }
        prev = c;
    }
    if !holder.is_empty() {
        strs.push(holder);
    }
    strs
}

/// Resolve path
fn resolve_path(s: &str) -> String {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
            let path = Path::new(p).join(s);
            if path.exists() {
                return path.display().to_string();
            }
        }
    }
    s.to_owned()
}

fn array_to_c_array(args: &[CString]) -> Vec<*const c_char> {
    let mut args_p: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
    args_p.push(ptr::null());
    args_p
}

/// Command structure
pub struct Command {
    path: CString,
    args: Vec<CString>,
    env: Vec<CString>,
}

impl Command {
    /// Create a new command
    pub fn new(command_str: &str, mut env: Vec<String>) -> Command {
        let splitted = split_ident(command_str);
        let path = resolve_path(&splitted[0]);
        let mut envs = env::vars()
            .map(|(n, v)| format!("{}={}", n, v))
            .collect::<Vec<String>>();
        envs.append(&mut env);
        let splitted_c = splitted
            .into_iter()
            .filter_map(|s| CString::new(s).ok())
            .collect();
        let path_c = CString::new(path).unwrap();
        let envs_c = envs.into_iter()
            .filter_map(|s| CString::new(s).ok())
            .collect();
        Command {
            path: path_c,
            args: splitted_c,
            env: envs_c,
        }
    }

    /// Exec
    pub fn exec(&self) -> i32 {
        unsafe {
            return libc::execve(
                self.path.as_ptr(),
                array_to_c_array(self.args.as_slice()).as_ptr(),
                array_to_c_array(self.env.as_slice()).as_ptr(),
            );
        }
    }
}
