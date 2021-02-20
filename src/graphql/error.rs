use std::io;
use std::fmt;
use std::error;


#[derive(Debug)]
pub enum GraphQLError {
    Ureq(ureq::Error),
    SerdeJson(serde_json::Error),
    Io(io::Error),
}

// Implement Display so GraphQLError can expose underlying errors via Display
impl fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Underlying errors already implement 'Display', so we use their implementations
            GraphQLError::Ureq(ref err) => write!(f, "Ureq Error: {}", err),
            GraphQLError::SerdeJson(ref err) => write!(f, "SerdeJson Error: {}", err),
            GraphQLError::Io(ref err) => write!(f, "Io Error: {}", err),
        }
    }
}

// Implement Error Trait 
impl error::Error for GraphQLError {

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            GraphQLError::Ureq(ref err) => Some(err),
            GraphQLError::SerdeJson(ref err) => Some(err),
            GraphQLError::Io(ref err) => Some(err),
        }
    }


    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            // N.B. Both of these implicitly cast `err` from their concrete
            // types (either `&io::Error` or `&num::ParseIntError`)
            // to a trait object `&Error`. This works because both error types
            // implement `Error`.

            GraphQLError::Ureq(ref err) => Some(err),
            GraphQLError::SerdeJson(ref err) => Some(err),
            GraphQLError::Io(ref err) => Some(err),
        }
    }
}

impl From<ureq::Error> for GraphQLError {
    fn from(err: ureq::Error) -> GraphQLError {
        GraphQLError::Ureq(err)
    }
}

impl From<serde_json::Error> for GraphQLError {
    fn from(err: serde_json::Error) -> GraphQLError {
        GraphQLError::SerdeJson(err)
    }
}

impl From<io::Error> for GraphQLError {
    fn from(err: io::Error) -> GraphQLError {
        GraphQLError::Io(err)
    }
}
