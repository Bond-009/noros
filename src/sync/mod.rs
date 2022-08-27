pub mod mutex;

#[derive(Debug)]
pub enum TryLockError {
    WouldBlock
}

pub type TryLockResult<Guard> = Result<Guard, TryLockError>;
