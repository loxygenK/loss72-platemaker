#[macro_export]
macro_rules! log {
    (job_start: $format:literal $(, $($value:expr),* $(,)? )?) => {
        log!(_job => "\n", "3", $format $(, $($value),*)?);
    };

    (job_end: $format:literal $(, $($value:expr),* $(,)? )?) => {
        log!(_job => "", "2", $format $(, $($value),*)?);
    };

    (section: $format:literal $(, $($value:expr),* $(,)? )?) => {
        log!(_status => "5", "*", "", $format $(, $($value),*)?);
    };

    (step: $format:literal $(, $($value:expr),* $(,)? )?) => {
        log!(_status => "4", "┃", "\x1b[38;5;153m", $format $(, $($value),*)?);
    };

    (ok: $format:literal $(, $($value:expr),* $(,)? )?) => {
        log!(_status => "2", "✓", "\x1b[1m", $format $(, $($value),*)?);
    };

    (warn: $format:literal $(, $($value:expr),* $(,)? )?) => {
        log!(_notice => "3", "warning", $format $(, $($value),*)?);
    };

    (_job => $prefix: literal, $escape:literal, $format:literal $(, $($value:expr),* $(,)?)?) => {
        println!(
            "{}",
            format!(
                concat!($prefix, "  \x1b[48;5;", $escape, ";38;5;0;1m ", "  ", $format, "   \x1b[m")
                $(,
                    $($value),*
                )?
            )
        );
    };

    (_status => $escape:literal, $tag:literal, $reset:literal, $format:literal $(, $($value:expr),* $(,)?)?) => {
        println!(
            "{}",
            format!(
                concat!("\x1b[38;5;", $escape, "m  ", $tag, $reset, "  ", $format, "\x1b[m")
                $(,
                    $($value),*
                )?
            )
        );
    };

    (_notice => $escape:literal, $step:literal, $format:literal $(, $($value:expr),* $(,)?)?) => {
        println!(
            "{}",
            format!(
                concat!("\x1b[38;5;", $escape, "m{}", ": ", $format, "\x1b[m"),
                $step $(,
                    $($value),*
                )?
            )
        );
    };
}
