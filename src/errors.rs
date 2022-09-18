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
}
