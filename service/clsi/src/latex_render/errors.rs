use std::sync;

pub enum Error {
    FormatError(sync::Arc<str>),
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        match err {
            Error::FormatError(msg) => msg.to_string(),
        }
    }
}
