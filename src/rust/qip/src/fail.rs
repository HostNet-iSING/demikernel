use custom_error::custom_error;
use std::{io::Error, rc::Rc};

custom_error! {#[derive(Clone)] pub Fail
    IoError{source: Rc<Error>} = "I/O failure",
    Misdelivered{} = "misdelivered packet",
    Unsupported{} = "unsupported",
    Ignored{} = "operation has no effect",
    TryAgain{} = "try again later",
}

impl From<Error> for Fail {
    fn from(e: Error) -> Self {
        Fail::IoError { source: Rc::new(e) }
    }
}
