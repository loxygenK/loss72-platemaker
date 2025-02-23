use std::fmt::Display;

use loss72_platemaker_core::log;

pub fn report_error(err: &impl Display) {
    log!(warn: "{}", err);
}

pub fn report_if_fail<T, E: Display>(func: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    let result = func();

    if let Err(e) = &result {
        report_error(e);
    };

    result
}
