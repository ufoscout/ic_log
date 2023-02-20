mod termcolor;

use self::termcolor::BufferWriter;
use std::{fmt, io, mem};

pub(super) mod glob {
    pub use super::termcolor::glob::*;
    pub use super::*;
}

pub(super) use self::termcolor::Buffer;

/// Log target, either `stdout`, `stderr`.
pub enum Target {
    /// Logs will be sent to standard output.
    Stdout,
    /// Logs will be sent to standard error.
    Stderr,
}

impl Default for Target {
    fn default() -> Self {
        Target::Stderr
    }
}

impl fmt::Debug for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Stdout => "stdout",
                Self::Stderr => "stderr",
            }
        )
    }
}

/// Log target, either `stdout`, `stderr` or a custom pipe.
///
/// Same as `Target`, except the pipe is wrapped in a mutex for interior mutability.
pub(super) enum WritableTarget {
    /// Logs will be sent to standard output.
    Stdout,
    /// Logs will be sent to standard error.
    Stderr,
}

impl From<Target> for WritableTarget {
    fn from(target: Target) -> Self {
        match target {
            Target::Stdout => Self::Stdout,
            Target::Stderr => Self::Stderr,
        }
    }
}

impl Default for WritableTarget {
    fn default() -> Self {
        Self::from(Target::default())
    }
}

impl fmt::Debug for WritableTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Stdout => "stdout",
                Self::Stderr => "stderr",
            }
        )
    }
}

/// A terminal target with color awareness.
pub(crate) struct Writer {
    inner: BufferWriter,
}

impl Writer {

    pub(super) fn buffer(&self) -> Buffer {
        self.inner.buffer()
    }

    pub(super) fn print(&self, buf: &Buffer) -> io::Result<()> {
        self.inner.print(buf)
    }
}

/// A builder for a terminal writer.
///
/// The target and style choice can be configured before building.
#[derive(Debug)]
pub(crate) struct Builder {
    target: WritableTarget,
    built: bool,
}

impl Builder {
    /// Initialize the writer builder with defaults.
    pub(crate) fn new() -> Self {
        Builder {
            target: Default::default(),
            built: false,
        }
    }

    /// Set the target to write to.
    pub(crate) fn target(&mut self, target: Target) -> &mut Self {
        self.target = target.into();
        self
    }

    /// Build a terminal writer.
    pub(crate) fn build(&mut self) -> Writer {
        assert!(!self.built, "attempt to re-use consumed builder");
        self.built = true;

        let writer = match mem::take(&mut self.target) {
            WritableTarget::Stderr => BufferWriter::stderr(),
            WritableTarget::Stdout => BufferWriter::stdout(),
        };

        Writer {
            inner: writer,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Writer").finish()
    }
}
