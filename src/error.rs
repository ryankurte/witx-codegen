use std::fmt;

#[derive(Debug)]
pub enum Error {
    Witnext(witnext::WitxError),
    Witx0_9(witx0_9::WitxError),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<witnext::WitxError> for Error {
    fn from(e: witnext::WitxError) -> Self {
        Self::Witnext(e)
    }
}

impl From<witx0_9::WitxError> for Error {
    fn from(e: witx0_9::WitxError) -> Self {
        Self::Witx0_9(e)
    }
}
