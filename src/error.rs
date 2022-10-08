use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error(transparent)]
    IOError(#[from] io::Error),
}
