// CSV DTOs

use getset::Getters;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Getters)]
pub struct Record {
    #[get = "pub"]
    r#type: String,

    #[get = "pub"]
    client: u16,

    #[get = "pub"]
    tx: u32,

    #[get = "pub"]
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
