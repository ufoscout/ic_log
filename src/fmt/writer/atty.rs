/*
This internal module contains the terminal detection implementation.

If the `auto-color` feature is enabled then we detect whether we're attached to a particular TTY.
Otherwise, assume we're not attached to anything. This effectively prevents styles from being
printed.
*/
mod imp {
    pub(in crate::fmt) fn is_stdout() -> bool {
        false
    }

    pub(in crate::fmt) fn is_stderr() -> bool {
        false
    }
}

pub(in crate::fmt) use self::imp::*;
