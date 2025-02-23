use std::fmt::Display;

pub fn report_error(err: &impl Display) {}

pub fn report_if_fail<T, E: Display>(func: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    let result = func();

    if let Err(e) = &result {
        report_error(e);
    };

    result
}
