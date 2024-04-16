#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    IOError,
    KeyNotFound,
    KeyExists,
}
