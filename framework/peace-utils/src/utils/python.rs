use colored::Colorize;
use pyo3::{types::PyBytes, PyErr, Python};
use std::time::Instant;

#[inline]
/// I have implemented Rust's rijndael-256-cbc algorithm
/// and do not need to use this python method anymore.
/// But keep this code as an example of Rust calling python.
/// (https://github.com/Pure-Peace/simple-rijndael)
///
/// Initialize some methods into the global python interpreter
///
/// ~~NOTE: This is a temporary solution.~~
/// ~~Some problems cannot be solved temporarily.~~
/// ~~When the problem is solved, Python may be removed..~~
///
pub fn python_rijndael_init() {
    info!("{}", "Initialing Python3...".bold().bright_blue());
    let code = include_str!("../ext/rijndael.py");
    let gil = Python::acquire_gil();
    let py = gil.python();
    if let Err(err) = py.run(code, None, None) {
        error!("[Python] Failed to initial python3, err: {:?}", err);
        panic!()
    };
}

#[inline]
/// I have implemented Rust's rijndael-256-cbc algorithm
/// and do not need to use this python method anymore.
/// But keep this code as an example of Rust calling python.
/// (https://github.com/Pure-Peace/simple-rijndael)
///
/// ~~Because Rust does not have an implementation of the rijndael algorithm,~~
/// ~~it is temporarily solved with the built-in python3 interpreter.~~
pub fn submit_modular_decrypt(
    osu_version: i32,
    iv: &Vec<u8>,
    score: &Vec<u8>,
) -> Result<Vec<String>, PyErr> {
    debug!("[SubmitModular] Python decrypt start");
    let start = Instant::now();
    let gil = Python::acquire_gil();
    let python = gil.python();
    let module = python.import("__main__")?;

    let decryp_result = module
        .call_method1(
            "rijndael_cbc_decrypt",
            (
                format!("osu!-scoreburgr---------{}", osu_version),
                PyBytes::new(python, iv),
                PyBytes::new(python, score),
            ),
        )?
        .extract()?;
    let end = start.elapsed();
    debug!(
        "[SubmitModular] Python decrypt success, time spent: {:?}",
        end
    );
    return Ok(decryp_result);
}
