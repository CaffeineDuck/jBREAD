#[derive(Debug, Clone)]
pub struct Error {
    line: u32,
    message: String,
    where_: String,
}

impl Error {
    pub fn new(line: u32, where_: String, message: String) -> Self {
        Self {
            line,
            message,
            where_,
        }
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        format!(
            "\"{}\" at line: {} in {}",
            self.message, self.line, self.where_
        )
    }
}

#[derive(Debug, Clone)]
pub enum JBreadErrors {
    ParseError(Error),
    RunTimeException(Error),
}

impl ToString for JBreadErrors {
    fn to_string(&self) -> String {
        match self {
            JBreadErrors::ParseError(error) => error.to_string(),
            JBreadErrors::RunTimeException(error) => error.to_string(),
        }
    }
}

impl JBreadErrors {
    pub fn report(&self) {
        eprintln!("{:?}\n{}", self, self.to_string());
    }
}

pub type JBreadResult<T> = Result<T, JBreadErrors>;
