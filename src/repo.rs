use crate::{errors::RepositoryError, transaction::Transaction};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Client {
    id: u16,
    available: f64,
    held: f64,
    locked: bool,
    transactions: HashMap<u32, Transaction>,
    disputed: HashSet<u32>,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            available: 0.0,
            held: 0.0,
            locked: false,
            transactions: HashMap::new(),
            disputed: HashSet::new(),
        }
    }

    pub fn register_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), RepositoryError> {
        let tx;
        match transaction {
            Transaction::Deposit(data) => {
                tx = data.tx().to_owned();
                if self.transactions.keys().any(|&k| k == tx) {
                    return Err(RepositoryError::DuplicateTransactionId(tx));
                }

                self.available += data.amount();
                self.transactions.insert(tx, transaction);
            }
            Transaction::Withdrawal(data) => {
                tx = data.tx().to_owned();
                if self.transactions.keys().any(|&k| k == tx) {
                    return Err(RepositoryError::DuplicateTransactionId(tx));
                }
                if self.available < *data.amount() {
                    return Err(RepositoryError::WithdrawalWouldResultInNegativeAmount(
                        *data.client(),
                    ));
                }

                self.available -= data.amount();
                self.transactions.insert(tx, transaction);
            }
            Transaction::Dispute(data) => {
                tx = data.tx().to_owned();
                let org_tx = self
                    .transactions
                    .get(&tx)
                    .ok_or(RepositoryError::TransactionDoesNotExist(tx, self.id))?;

                if self.disputed.contains(&tx) {
                    return Err(RepositoryError::TransactionAlreadyDisputed(tx));
                }

                // I assume dispute can only be done on deposit
                if let Transaction::Deposit(data) = org_tx {
                    self.available -= data.amount();
                    self.held += data.amount();
                    self.disputed.insert(tx);
                } else {
                    return Err(RepositoryError::WrongReferenceTransactionType);
                }
            }
            Transaction::Resolve(data) => {
                tx = data.tx().to_owned();
                let org_tx = self
                    .transactions
                    .get(&tx)
                    .ok_or(RepositoryError::TransactionDoesNotExist(tx, self.id))?;

                if !self.disputed.contains(&tx) {
                    return Err(RepositoryError::TransactionNotDisputed(tx));
                }

                // I assume dispute can only be done on deposit
                if let Transaction::Deposit(data) = org_tx {
                    self.available += data.amount();
                    self.held -= data.amount();
                    self.disputed.remove(&tx); // not checking for result, b/c we have just checked that the set contains the id
                } else {
                    return Err(RepositoryError::WrongReferenceTransactionType);
                }
            }
            Transaction::Chargeback(_) => todo!(),
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Repository {
    clients: HashMap<u16, Client>,
}

impl Repository {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn register_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), RepositoryError> {
        let client_id = match transaction {
            Transaction::Deposit(data) => data.client().to_owned(),
            Transaction::Withdrawal(data) => data.client().to_owned(),
            Transaction::Dispute(data) => data.client().to_owned(),
            Transaction::Resolve(data) => data.client().to_owned(),
            Transaction::Chargeback(data) => data.client().to_owned(),
        };

        let client = self
            .clients
            .entry(client_id)
            .or_insert_with(|| Client::new(client_id));

        client.register_transaction(transaction)?;

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
        let tr2 = Transaction::Withdrawal(TransactionDataAmount::new(1, 2, 2.0));
        let mut repo = Repository::new();

        repo.register_transaction(tr1).expect("deposit failed");
        let result = repo.register_transaction(tr2);

        match result {
            Ok(_) => panic!("did not return error"),
            Err(e) => match e {
                crate::errors::RepositoryError::WithdrawalWouldResultInNegativeAmount(_) => {}
                _ => panic!("wrong error returned: `{:?}`", e),
            },
        }
    }
}
