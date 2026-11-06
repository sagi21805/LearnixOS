unsafe extern "Rust" {
    pub unsafe fn kprint(args: ::core::fmt::Arguments<'_>);
}

/// Prints formatted text to the console, without a newline.
///
/// The text may hold ANSI color sequences, usually through the constants
/// and functions in `common::ansi`.
///
/// ```ignore
/// use common::ansi::{RED, RESET, green};
///
/// print!("{RED}Disk failed{RESET} with code {code}, ");
/// print!("retry {}", green("passed"));
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        unsafe { $crate::fmt::kprint(format_args!($($arg)*)) }
    }};
}

/// Same as [`print!`], with a newline at the end.
///
/// The format string can't be joined with `"\n"` by `concat!`, because
/// `format_args!` can't capture names like `{RED}` from an expanded
/// string.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

/// Prints a line with a green `OK` tag, in one print.
#[macro_export]
macro_rules! okprintln {
    ($($arg:tt)*) => {
        $crate::print!(
            "[\x1b[32m OK \x1b[0m]: {}\n",
            format_args!($($arg)*)
        )
    };
}

/// Prints a line with a red `FAIL` tag, in one print.
#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        $crate::print!(
            "[\x1b[31mFAIL\x1b[0m]: {}\n",
            format_args!($($arg)*)
        )
    };
}
