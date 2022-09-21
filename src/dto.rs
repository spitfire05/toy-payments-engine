//! CSV Data Transfer Objects

use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::repo::Client;

#[derive(Debug, Deserialize, Clone, Getters)]
pub struct InputRecord {
    #[get = "pub"]
    r#type: String,

    #[get = "pub"]
    client: u16,

    #[get = "pub"]
    tx: u32,

    #[get = "pub"]
    amount: Option<f64>,
}

impl InputRecord {
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

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct OutputRecord {
    client: u16,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

impl OutputRecord {
    pub fn new(client: u16, available: f64, held: f64, total: f64, locked: bool) -> Self {
        Self {
            client,
            available: format!("{:.4}", available),
            held: format!("{:.4}", held),
            total: format!("{:.4}", total),
            locked,
        }
    }
}

impl From<&Client> for OutputRecord {
    fn from(c: &Client) -> Self {
        let available = *c.available();
        let held = *c.held();
        let total = available + held;
        Self::new(*c.id(), available, held, total, *c.locked())
    }
}

impl From<Client> for OutputRecord {
    fn from(c: Client) -> Self {
        OutputRecord::from(&c)
    }
}
