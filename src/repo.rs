use crate::{
    errors::RepositoryError,
    transaction::{Transaction, TransactionDataAmount},
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Client {
    available: f64,
    held: f64,
}

impl Client {
    pub fn new(available: f64, held: f64) -> Self {
        Self { available, held }
    }
}

#[derive(Debug, Clone)]
pub struct Repository {
    clients: HashMap<u16, Client>,
    deposits: HashMap<u32, TransactionDataAmount>,
    withdrawals: HashMap<u32, TransactionDataAmount>,
}

impl Repository {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            deposits: HashMap::new(),
            withdrawals: HashMap::new(),
        }
    }

    pub fn register_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), RepositoryError> {
        match transaction {
            Transaction::Deposit(data) => {
                self.clients
                    .entry(*data.client())
                    .and_modify(|c| c.available += data.amount())
                    .or_insert_with(|| Client::new(*data.amount(), 0.0));
                self.deposits.insert(*data.tx(), data);
            }
            Transaction::Withdrawal(data) => {
                let c = self.clients.get_mut(data.client()).ok_or_else(|| {
                    RepositoryError::WithdrawalWouldResultInNegativeAmount(*data.client())
                })?;
                if c.available < *data.amount() {
                    return Err(RepositoryError::WithdrawalWouldResultInNegativeAmount(
                        *data.client(),
                    ));
                }

                c.available -= data.amount();
                self.withdrawals.insert(*data.tx(), data);
            }
            Transaction::Dispute(_) => todo!(),
            Transaction::Resolve(_) => todo!(),
            Transaction::Chargeback(_) => todo!(),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Repository;
    use crate::transaction::{Transaction, TransactionDataAmount};

    #[test]
    fn withdrawal_on_non_existing_client_results_in_error() {
        let tr = Transaction::Withdrawal(TransactionDataAmount::new(1, 1, 1.0));
        let mut repo = Repository::new();

        let result = repo.register_transaction(tr);

        match result {
            Ok(_) => panic!("did not return error"),
            Err(e) => match e {
                crate::errors::RepositoryError::WithdrawalWouldResultInNegativeAmount(_) => {}
                _ => panic!("wrong error returned"),
            },
        }
    }

    #[test]
    fn withdrawal_on_insufficient_funds_results_in_error() {
        let tr1 = Transaction::Deposit(TransactionDataAmount::new(1, 1, 1.0));
        let tr2 = Transaction::Withdrawal(TransactionDataAmount::new(1, 1, 2.0));
        let mut repo = Repository::new();

        repo.register_transaction(tr1).expect("deposit failed");
        let result = repo.register_transaction(tr2);

        match result {
            Ok(_) => panic!("did not return error"),
            Err(e) => match e {
                crate::errors::RepositoryError::WithdrawalWouldResultInNegativeAmount(_) => {}
                _ => panic!("wrong error returned"),
            },
        }
    }
}
