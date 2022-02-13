use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::{TClientId, TMoney, TTrxID};

pub trait Transaction: From<TransactionData> {
    fn validate(&self) -> TransactionValid;
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
}

#[derive(Deserialize)]
pub struct TransactionData {
    #[serde(rename = "type")]
    pub ttype: TransactionType,
    pub client: TClientId,
    pub tx: TTrxID,
    #[serde(default)]
    pub amount: Option<TMoney>,
}

#[derive(PartialEq)]
pub enum TransactionValid {
    Ok,
    Warn(&'static str),
    Invalid(&'static str)
}

impl TransactionData {
    pub fn validate(&self) -> TransactionValid {
        match self.ttype {
            TransactionType::Deposit => 
                if let Some(amt) = self.amount {
                    if amt > 0.0 {TransactionValid::Ok}
                    else if amt == 0.0 {TransactionValid::Warn("Amount == 0 in Deposit transaction")}
                    else {TransactionValid::Invalid("Amount < 0 in Deposit transaction")}
                } else {
                    TransactionValid::Invalid("Amount not present in Deposit transaction")
                },
            TransactionType::Withdrawal => 
                if let Some(amt) = self.amount {
                    if amt > 0.0 {TransactionValid::Ok}
                    else if amt == 0.0 {TransactionValid::Warn("Amount == 0 in Withdrawal transaction")}
                    else {TransactionValid::Invalid("Amount < 0 in Withdrawal transaction")}
                } else {
                    TransactionValid::Invalid("Amount not present in Withdrawal transaction")
                },
            TransactionType::Dispute => 
                if self.amount.is_none() {
                    TransactionValid::Ok
                } else {
                    TransactionValid::Warn("Amount is present in Dispute transaction")
                },
        }
    }
}

pub fn store_transaction(store: &mut HashSet<TTrxID>, transaction: &TransactionData) -> Result<()> {
    assert!(! matches!(transaction.validate(), TransactionValid::Invalid(_)));
    if store.insert(transaction.tx) {
        Ok(())
    } else {
        bail!("Duplicated ID")
    }
}
