use std::error::Error;
use std::fmt::{Display, Error as fmtError, Formatter};

pub(crate) type Result<T> = std::result::Result<T, NError>;

pub const ERROR_MESSAGE_NONE: i32 = 0;
pub const ERROR_PARSE: i32 = 1;
pub const ERROR_MESSAGE_SIZE_TOO_LARGE: i32 = 2;
pub const ERROR_INVALID_SUBJECT: i32 = 3;
pub const ERROR_SUBSCRIBTION_NOT_FOUND: i32 = 4;
pub const ERROR_CONNECTION_CLOSED: i32 = 5;

//pub const ERROR_UNKOWN_ERROR: i32 = 1000;

#[derive(Debug)]
pub struct NError {
    err_code: i32,
}

impl NError {
    pub fn new(err_code: i32) -> Self {
        NError { err_code }
    }

    fn desc_error_message(&self) -> &'static str {
        match self.err_code {
            ERROR_PARSE => "Parse error",
            _ => "other error",
        }
    }
}

impl Error for NError {}

impl Display for NError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), fmtError> {
        write!(f, "NError[{}{}]", self.err_code, self.desc_error_message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        println!("{}", NError::new(ERROR_PARSE));
    }
}
