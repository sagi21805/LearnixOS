//! Named ANSI color sequences, for printing in color.

use core::fmt::{self, Debug, Display, Formatter};

/// Cancels every color, back to the default colors.
pub const RESET: &str = "\x1b[0m";
/// The default foreground color.
pub const DEFAULT_FOREGROUND: &str = "\x1b[39m";
/// The default background color.
pub const DEFAULT_BACKGROUND: &str = "\x1b[49m";

/// A value that prints in one color, then resets.
pub struct Colored<T> {
    color: &'static str,
    value: T,
}

impl<T: Display> Display for Colored<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.color)?;
        self.value.fmt(f)?;
        f.write_str(RESET)
    }
}

impl<T: Debug> Debug for Colored<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.color)?;
        self.value.fmt(f)?;
        f.write_str(RESET)
    }
}

/// A constant, a function and a macro for each color, from its SGR
/// parameter.
macro_rules! colors {
    ($($function:ident, $constant:ident = $parameter:literal;)*) => {
        $(
            #[doc = concat!("Selects `", stringify!($function), "`.")]
            pub const $constant: &str =
                concat!("\x1b[", $parameter, "m");

            #[inline]
            #[doc = concat!(
                "`value` in `", stringify!($function), "`, then a reset."
            )]
            pub const fn $function<T>(value: T) -> Colored<T> {
                Colored { color: $constant, value }
            }

            #[doc = concat!(
                "The string literal `text` in `",
                stringify!($function),
                "`, then a reset, built at compile time."
            )]
            pub macro $function($text:expr) {
                concat!("\x1b[", $parameter, "m", $text, "\x1b[0m")
            }
        )*
    };
}

colors! {
    black, BLACK = 30;
    red, RED = 31;
    green, GREEN = 32;
    yellow, YELLOW = 33;
    blue, BLUE = 34;
    magenta, MAGENTA = 35;
    cyan, CYAN = 36;
    white, WHITE = 37;

    bright_black, BRIGHT_BLACK = 90;
    bright_red, BRIGHT_RED = 91;
    bright_green, BRIGHT_GREEN = 92;
    bright_yellow, BRIGHT_YELLOW = 93;
    bright_blue, BRIGHT_BLUE = 94;
    bright_magenta, BRIGHT_MAGENTA = 95;
    bright_cyan, BRIGHT_CYAN = 96;
    bright_white, BRIGHT_WHITE = 97;

    bg_black, BG_BLACK = 40;
    bg_red, BG_RED = 41;
    bg_green, BG_GREEN = 42;
    bg_yellow, BG_YELLOW = 43;
    bg_blue, BG_BLUE = 44;
    bg_magenta, BG_MAGENTA = 45;
    bg_cyan, BG_CYAN = 46;
    bg_white, BG_WHITE = 47;
}
