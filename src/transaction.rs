use crate::errors::*;
use getset::Getters;
use serde::Deserialize;
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

impl TryFrom<&Record> for Transaction {
    type Error = DeserializationError;

    fn try_from(value: &Record) -> Result<Self, Self::Error> {
        match value.r#type.as_str() {
            "deposit" => {
                let data = TransactionDataAmount::new(
                    value.client,
                    value.tx,
                    value.amount.ok_or_else(|| {
                        DeserializationError::AmountMissing(value.r#type.to_owned())
                    })?,
                );

                Ok(Transaction::Deposit(data))
            }
            "withdrawal" => {
                let data = TransactionDataAmount::new(
                    value.client,
                    value.tx,
                    value.amount.ok_or_else(|| {
                        DeserializationError::AmountMissing(value.r#type.to_owned())
                    })?,
                );

                Ok(Transaction::Withdrawal(data))
            }
            "dispute" => {
                let data = TransactionData::new(value.client, value.tx);

                Ok(Transaction::Dispute(data))
            }
            "resolve" => {
                let data = TransactionData::new(value.client, value.tx);

                Ok(Transaction::Resolve(data))
            }
            "chargeback" => {
                let data = TransactionData::new(value.client, value.tx);

                Ok(Transaction::Chargeback(data))
            }
            _ => Err(DeserializationError::UnknownTransactionType(
                value.r#type.to_owned(),
            )),
        }
    }
}

impl TryFrom<Record> for Transaction {
    type Error = DeserializationError;

    fn try_from(value: Record) -> Result<Self, Self::Error> {
        Transaction::try_from(&value)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Record {
    r#type: String,
    client: u16,
    tx: u32,
    amount: Option<f64>,
}

impl Record {
    #[cfg(test)]
    pub fn new<T: Into<String>>(r#type: T, client: u16, tx: u32, amount: Option<f64>) -> Self {
        let owned_type = r#type.into();

        Self {
            r#type: owned_type,
            client,
            tx,
            amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Record, Transaction};
    use crate::errors::DeserializationError;
    use paste::paste;
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;
    use std::convert::TryInto;

    impl Arbitrary for Record {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut amount;
            loop {
                amount = f64::arbitrary(g);
                if amount.is_sign_positive() && amount.is_finite() {
                    break;
                }
            }

            // TODO: this can be changed to lazy global static value, but this is just a unit test..
            let transaction_types =
                vec!["deposit", "withdrawal", "dispute", "resolve", "chargeback"];

            let chosen_type = *g.choose(&transaction_types).unwrap();
            Record {
                r#type: chosen_type.to_owned(),
                client: u16::arbitrary(g),
                tx: u32::arbitrary(g),
                amount: Some(amount),
            }
        }
    }

    #[quickcheck]
    fn transaction_from_record(r: Record) -> bool {
        let tr = Transaction::try_from(&r).unwrap();

        macro_rules! record_equals_data {
            ($record:expr, $data:expr) => {
                // TODO: also check amount?
                $data.client == $record.client && $data.tx == $record.tx
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
                    let record = Record::new($type, 1, 1, None);
                    let result: Result<Transaction, _> = (&record).try_into();
                    assert!(result.is_err());
                    assert_eq!(
                        result.unwrap_err(),
                        DeserializationError::AmountMissing(record.r#type)
                    );
                }
            }
        };
    }

    test_needs_amount!("deposit");
    test_needs_amount!("withdrawal");
}
