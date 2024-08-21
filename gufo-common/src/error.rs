/// Type that allows to return data with the error
///
/// This is useful since commands like `new()` will take ownership of the data.
/// Using this as error type allows to continue using the data afterward.
pub struct ErrorWithData<E: std::error::Error> {
    err: E,
    data: Vec<u8>,
}

impl<E: std::error::Error> ErrorWithData<E> {
    pub fn new(err: E, data: Vec<u8>) -> Self {
        Self { err, data }
    }

    pub fn err(&self) -> &E {
        &self.err
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn map_err<F: std::error::Error>(self, op: impl FnOnce(E) -> F) -> ErrorWithData<F> {
        let err = op(self.err);
        ErrorWithData {
            err,
            data: self.data,
        }
    }
}

impl<E: std::error::Error> std::fmt::Debug for ErrorWithData<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorWithData")
            .field("err", &self.err)
            .field("data", &format!("{} bytes", self.data.len()))
            .finish()
    }
}

impl<E: std::error::Error> std::fmt::Display for ErrorWithData<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl<E: std::error::Error> std::error::Error for ErrorWithData<E> {}
