//! Repository & Client - internal system state

use getset::Getters;

use crate::{errors::RepositoryError, transaction::Transaction};
use std::collections::{HashMap, HashSet};

/// Represents internal state of the client in the engine
#[derive(Debug, Clone, Getters)]
pub struct Client {
    /// Client's unique id
    #[get = "pub"]
    id: u16,

    /// Current available funds
    #[get = "pub"]
    available: f64,

    /// Current held (disputed) funds
    #[get = "pub"]
    held: f64,

    /// Whether the client is locked (chargeback occured)
    #[get = "pub"]
    locked: bool,

    /// Deposit and withdrawal log. On real system this should be backed by some kind of DB, as this will grow indefinitely.
    #[get = "pub"]
    transactions: HashMap<u32, Transaction>,

    /// Set of disputed transactions's IDs
    #[get = "pub"]
    disputed: HashSet<u32>,
}

impl Client {
    /// Creates new client with given `id`
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

    /// Registers the transaction for this client
    pub fn register_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), RepositoryError> {
        let tx;
        match transaction {
            Transaction::Deposit(data) => {
                if self.locked {
                    return Err(RepositoryError::ClientLocked(self.id));
                }

                tx = data.tx().to_owned();
                if self.transactions.keys().any(|&k| k == tx) {
                    return Err(RepositoryError::DuplicateTransactionId(tx));
                }

                self.available += data.amount();
                self.transactions.insert(tx, transaction);
            }
            Transaction::Withdrawal(data) => {
                if self.locked {
                    return Err(RepositoryError::ClientLocked(self.id));
                }

                tx = data.tx().to_owned();
                if self.transactions.keys().any(|&k| k == tx) {
                    return Err(RepositoryError::DuplicateTransactionId(tx));
                }
                if self.available < *data.amount() {
                    return Err(RepositoryError::InsufficientFunds(*data.client()));
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
            Transaction::Chargeback(data) => {
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
                    self.held -= data.amount();
                    self.locked = true;
                    self.disputed.remove(&tx); // not checking for result, b/c we have just checked that the set contains the id
                } else {
                    return Err(RepositoryError::WrongReferenceTransactionType);
                }
            }
        }

        Ok(())
    }
}

/// Repository of all clients handled by this engine.
#[derive(Debug, Clone)]
pub struct Repository {
    // even though `Client` struct holds its id, we use HashMap here
    // instead of Vector for performance reasons
    clients: HashMap<u16, Client>,
}

impl Repository {
    /// Returns new empty `Repository`
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Registers the transaction and modifies internal state
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

    /// Returns an iterator over clients existing in the system
    pub fn iter_clients(&self) -> impl Iterator<Item = &Client> {
        self.clients.values()
    }
}

#[cfg(test)]
mod tests {
    use super::Repository;
    use crate::{
        repo::Client,
        transaction::{Transaction, TransactionData, TransactionDataAmount},
    };
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use std::collections::{HashMap, HashSet};

    macro_rules! valid_amount {
        ($amount:expr) => {
            $amount.is_normal() && $amount.is_sign_positive()
        };
    }

    #[test]
    fn withdrawal_on_non_existing_client_results_in_error() {
        let tr = Transaction::Withdrawal(TransactionDataAmount::new(1, 1, 1.0).unwrap());
        let mut repo = Repository::new();

        let result = repo.register_transaction(tr);

        match result {
            Ok(_) => panic!("did not return error"),
            Err(e) => match e {
                crate::errors::RepositoryError::InsufficientFunds(_) => {}
                _ => panic!("wrong error returned"),
            },
        }
    }

    #[test]
    fn withdrawal_on_insufficient_funds_results_in_error() {
        let tr1 = Transaction::Deposit(TransactionDataAmount::new(1, 1, 1.0).unwrap());
        let tr2 = Transaction::Withdrawal(TransactionDataAmount::new(1, 2, 2.0).unwrap());
        let mut repo = Repository::new();

        repo.register_transaction(tr1).expect("deposit failed");
        let result = repo.register_transaction(tr2);

        match result {
            Ok(_) => panic!("did not return error"),
            Err(e) => match e {
                crate::errors::RepositoryError::InsufficientFunds(_) => {}
                _ => panic!("wrong error returned: `{:?}`", e),
            },
        }
    }

    #[test]
    fn locked_client_rejects_deposit_and_withdrawal() {
        macro_rules! test {
            ($tr:path) => {
                let mut c = Client {
                    id: 1,
                    available: 0.0,
                    held: 0.0,
                    locked: true,
                    transactions: HashMap::new(),
                    disputed: HashSet::new(),
                };
                let tr = $tr(TransactionDataAmount::new(1, 1, 1.0).unwrap());

                let result = c.register_transaction(tr);

                assert!(match result {
                    Ok(_) => false,
                    Err(e) => {
                        matches!(e, crate::errors::RepositoryError::ClientLocked(_))
                    }
                });
            };
        }

        test!(Transaction::Deposit);
        test!(Transaction::Withdrawal);
    }

    #[test]
    fn locked_client_accepts_dispute() {
        let mut log = HashMap::new();
        log.insert(
            1u32,
            Transaction::Deposit(TransactionDataAmount::new(1, 1, 1.0).unwrap()),
        );
        let mut c = Client {
            id: 1,
            available: 0.0,
            held: 0.0,
            locked: true,
            transactions: log,
            disputed: HashSet::new(),
        };
        let tr = Transaction::Dispute(TransactionData::new(1, 1));

        let result = c.register_transaction(tr);

        assert!(result.is_ok());
    }

    #[quickcheck]
    fn deposit_and_withdrawal_for_same_amount_equals_to_zero(x: f64) -> TestResult {
        if !valid_amount!(x) {
            return TestResult::discard();
        }

        let mut client = Client::new(1);
        let dep = Transaction::Deposit(TransactionDataAmount::new(1, 1, x).unwrap());
        let wit = Transaction::Withdrawal(TransactionDataAmount::new(1, 2, x).unwrap());

        client.register_transaction(dep).expect("Deposit failed");
        client.register_transaction(wit).expect("Withdrawal failed");

        TestResult::from_bool(client.available == 0.0 && client.held == 0.0)
    }

    #[quickcheck]
    fn deposit_and_dispute_result_in_held_funds(x: f64) -> TestResult {
        if !valid_amount!(x) {
            return TestResult::discard();
        }

        let mut client = Client::new(1);
        let dep = Transaction::Deposit(TransactionDataAmount::new(1, 1, x).unwrap());
        let dis = Transaction::Dispute(TransactionData::new(1, 1));

        client.register_transaction(dep).expect("Deposit failed");
        client.register_transaction(dis).expect("Dispute failed");

        TestResult::from_bool(client.available == 0.0 && client.held == x)
    }

    #[quickcheck]
    fn deposit_dispute_and_resolve_result_in_available_funds(x: f64) -> TestResult {
        if !valid_amount!(x) {
            return TestResult::discard();
        }

        let mut client = Client::new(1);
        let dep = Transaction::Deposit(TransactionDataAmount::new(1, 1, x).unwrap());
        let dis = Transaction::Dispute(TransactionData::new(1, 1));
        let res = Transaction::Resolve(TransactionData::new(1, 1));

        client.register_transaction(dep).expect("Deposit failed");
        client.register_transaction(dis).expect("Dispute failed");
        client.register_transaction(res).expect("Resolve failed");

        TestResult::from_bool(client.available == x && client.held == 0.0)
    }

    #[quickcheck]
    fn deposit_dispute_and_chargeback_result_in_no_funds_and_locked_client(x: f64) -> TestResult {
        if !valid_amount!(x) {
            return TestResult::discard();
        }

        let mut client = Client::new(1);
        let dep = Transaction::Deposit(TransactionDataAmount::new(1, 1, x).unwrap());
        let dis = Transaction::Dispute(TransactionData::new(1, 1));
        let cha = Transaction::Chargeback(TransactionData::new(1, 1));

        client.register_transaction(dep).expect("Deposit failed");
        client.register_transaction(dis).expect("Dispute failed");
        client.register_transaction(cha).expect("Chargeback failed");

        TestResult::from_bool(client.available == 0.0 && client.held == 0.0 && client.locked)
    }
}
