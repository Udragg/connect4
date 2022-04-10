/// All the possible recoverable errors produced by the game.
#[derive(Debug)]
pub enum Error {
    InvalidColumn,
    InvalidType,
    ColumnFull,
    InvalidInput,
    InvalidDim,
    NoPlayer,
    NoUndos,
    Display(c4_display::Error),
}

/// Result type making use of custom errors.
pub(crate) type GameResult<T> = Result<T, Error>;

impl From<c4_display::Error> for Error {
    fn from(e: c4_display::Error) -> Self {
        Self::Display(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {}
