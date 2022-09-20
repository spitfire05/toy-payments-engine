use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DeserializationError {
    #[error("`{0}` is not a known transaction type")]
    UnknownTransactionType(String),

    #[error("Transaction type `{0}` needs amount value")]
    AmountMissing(String),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RepositoryError {
    #[error("Withdrawal operation on client `{0}` would result in a negative amount")]
    WithdrawalWouldResultInNegativeAmount(u16),

    #[error("Transaction id `{0}` already exists")]
    DuplicateTransactionId(u32),

    #[error("REferenced transaction ID `{0}` does not exist under client `{1}`")]
    TransactionDoesNotExist(u32, u16),

    #[error("Wrong reference transaction type")]
    WrongReferenceTransactionType,

    #[error("Transaction ID `{0}` is already disputed")]
    TransactionAlreadyDisputed(u32),

    #[error("Transaction ID `{0}` is not disputed")]
    TransactionNotDisputed(u32),
}
