//! Transaction definitions

use crate::{dto::InputRecord, errors::*};
use getset::Getters;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Getters)]
pub struct TransactionDataAmount {
    #[get = "pub"]
    client: u16,

    #[get = "pub"]
    tx: u32,

    #[get = "pub"]
    amount: f64,
}

impl TransactionDataAmount {
    pub fn new(client: u16, tx: u32, amount: f64) -> Self {
        if !amount.is_normal() || amount.is_sign_negative() {
            panic!("Amount has to be non-zero, positive, finite value");
        }

        Self { client, tx, amount }
    }
}

#[derive(Debug, Clone, Copy, Getters)]
pub struct TransactionData {
    #[get = "pub"]
    client: u16,

    #[get = "pub"]
    tx: u32,
}

impl TransactionData {
    pub fn new(client: u16, tx: u32) -> Self {
        Self { client, tx }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Transaction {
    Deposit(TransactionDataAmount),
    Withdrawal(TransactionDataAmount),
    Dispute(TransactionData),
    Resolve(TransactionData),
    Chargeback(TransactionData),
}

impl TryFrom<&InputRecord> for Transaction {
    type Error = DeserializationError;

    fn try_from(value: &InputRecord) -> Result<Self, Self::Error> {
        match value.r#type().as_str() {
            "deposit" => {
                let data = TransactionDataAmount::new(
                    value.client().to_owned(),
                    value.tx().to_owned(),
                    value.amount().ok_or_else(|| {
                        DeserializationError::AmountMissing(value.r#type().to_owned())
                    })?,
                );

                Ok(Transaction::Deposit(data))
            }
            "withdrawal" => {
                let data = TransactionDataAmount::new(
                    value.client().to_owned(),
                    value.tx().to_owned(),
                    value.amount().ok_or_else(|| {
                        DeserializationError::AmountMissing(value.r#type().to_owned())
                    })?,
                );

                Ok(Transaction::Withdrawal(data))
            }
            "dispute" => {
                let data = TransactionData::new(value.client().to_owned(), value.tx().to_owned());

                Ok(Transaction::Dispute(data))
            }
            "resolve" => {
                let data = TransactionData::new(value.client().to_owned(), value.tx().to_owned());

                Ok(Transaction::Resolve(data))
            }
            "chargeback" => {
                let data = TransactionData::new(value.client().to_owned(), value.tx().to_owned());

                Ok(Transaction::Chargeback(data))
            }
            _ => Err(DeserializationError::UnknownTransactionType(
                value.r#type().to_owned(),
            )),
        }
    }
}

impl TryFrom<InputRecord> for Transaction {
    type Error = DeserializationError;

    fn try_from(value: InputRecord) -> Result<Self, Self::Error> {
        Transaction::try_from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::Transaction;
    use crate::{dto::InputRecord, errors::DeserializationError};
    use paste::paste;
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;
    use std::convert::TryInto;

    impl Arbitrary for InputRecord {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut amount;
            loop {
                amount = f64::arbitrary(g);
                if amount.is_sign_positive() && amount.is_normal() {
                    break;
                }
            }

            // TODO: this can be changed to lazy global static value, but this is just a unit test..
            let transaction_types =
                vec!["deposit", "withdrawal", "dispute", "resolve", "chargeback"];

            let chosen_type = *g.choose(&transaction_types).unwrap();

            InputRecord::new(
                chosen_type.to_owned(),
                u16::arbitrary(g),
                u32::arbitrary(g),
                Some(amount),
            )
        }
    }

    #[quickcheck]
    fn transaction_from_record(r: InputRecord) -> bool {
        let tr = Transaction::try_from(&r).unwrap();

        macro_rules! record_equals_data {
            ($record:expr, $data:expr) => {
                // TODO: also check amount?
                $data.client == *$record.client() && $data.tx == *$record.tx()
            };
        }

        match tr {
            Transaction::Deposit(data) => record_equals_data!(r, data),
            Transaction::Withdrawal(data) => record_equals_data!(r, data),
            Transaction::Dispute(data) => record_equals_data!(r, data),
            Transaction::Resolve(data) => record_equals_data!(r, data),
            Transaction::Chargeback(data) => record_equals_data!(r, data),
        }
    }

    macro_rules! test_needs_amount {
        ($type:expr) => {
            paste! {
                #[test]
                fn [<$type _needs_amount>]() {
                    let record = InputRecord::new($type, 1, 1, None);
                    let result: Result<Transaction, _> = (&record).try_into();
                    assert!(result.is_err());
                    assert_eq!(
                        result.unwrap_err(),
                        DeserializationError::AmountMissing(record.r#type().to_owned())
                    );
                }
            }
        };
    }

    test_needs_amount!("deposit");
    test_needs_amount!("withdrawal");
}
