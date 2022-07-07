pub mod mutex;

pub enum TryLockError {
    WouldBlock
}

pub type TryLockResult<Guard> = Result<Guard, TryLockError>;
