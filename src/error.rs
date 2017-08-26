use std::{fmt, error};
use std::str;
use std::error::Error;
use rocket_sync::SyncError;

pub enum ToolError {
    Sync(SyncError),
}

impl fmt::Display for ToolError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl fmt::Debug for ToolError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        let kind: &'static str = match *self {
            ToolError::Sync(ref e) => match *e {
                SyncError::TrackDoesntExist => "Sync Track does't exist",
            },
        };

        write!(fmt, "{}:\n{}", kind, self.description())
    }
}

impl error::Error for ToolError {
    fn description(&self) -> &str {
        match *self {
            ToolError::Sync(_) => "",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
