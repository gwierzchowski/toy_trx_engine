use std::collections::HashMap;

use anyhow::Result;
use enum_dispatch::enum_dispatch;
use serde::Deserialize;

use crate::{
    TClientId, TTrxID,
    accounts::AccountState
};

mod deposit;
mod withdrawal;
mod dispute;
mod resolve;
mod chargeback;

use deposit::Deposit;
use withdrawal::Withdrawal;
use dispute::Dispute;
use resolve::Resolve;
use chargeback::Chargeback;

/// Transaction Interface. Every transaction must implement it.
/// `TryFrom` implementation should initialization of transaction from input record, 
/// it may fail if input does not have all necessary data - in such case transaction will be discarded.
#[enum_dispatch]
pub trait TransactionInt: TryFrom<TransactionRec> {
    /// Returns transaction id
    fn id(&self) -> TTrxID;

    /// Performs additional validation of transaction consistency with possibility to raise a warning.
    /// In case of `Ok`, and `Warn` transaction is being processed, `Invalid` result cause transaction to be rejected.
    fn validate(&self) -> TransactionValid;

    /// Actually performs transaction making necessary changes in passed accounts.
    fn commit(&self, accounts:&mut HashMap::<TClientId,AccountState>) -> Result<()>;
}

/// Transaction object.
#[enum_dispatch(TransactionInt)]
pub enum Transaction {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Result of transaction validation
#[derive(PartialEq)]
pub enum TransactionValid {
    /// Transaction is valid and may be processed.
    Ok,
    /// Transaction is valid and may be processed, but passed message should be logged.
    Warn(&'static str),
    /// Transaction is invalid and should be rejected. Passed message should be logged.
    Invalid(&'static str)
}

impl TryFrom<TransactionRec> for Transaction {
    type Error = anyhow::Error;
    fn try_from(td: TransactionRec) -> Result<Self, Self::Error> {
        Ok(
            match &td.ttype {
                TransactionRecType::Deposit => Transaction::from(Deposit::try_from(td)?),
                TransactionRecType::Withdrawal => Transaction::from(Withdrawal::try_from(td)?),
                TransactionRecType::Dispute => Transaction::from(Dispute::try_from(td)?),
                TransactionRecType::Resolve => Transaction::from(Resolve::try_from(td)?),
                TransactionRecType::Chargeback => Transaction::from(Chargeback::try_from(td)?),
            }
        )
    }
}

/// Transaction type as may occur in input file as small caps word (first column).
#[derive(PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionRecType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Represents transaction record as read from input file.
#[derive(Deserialize)]
pub struct TransactionRec {
    #[serde(rename = "type")]
    pub ttype: TransactionRecType,
    pub client: TClientId,
    pub tx: TTrxID,
    #[serde(default)]
    pub amount: Option<f64>, // Option<TMoney> is possible here, but maybe f64 will be faster
}

mod tests {
    use std::collections::HashMap;
    
    use crate::{
        TClientId, TMoney,
        accounts::AccountState,
    };

    #[allow(dead_code)]
    pub fn create_accounts(balance: &[TMoney]) -> HashMap::<TClientId,AccountState> {
        let mut accounts = HashMap::<TClientId,AccountState>::new();
        for (id, bal) in balance.iter().enumerate() {
            accounts.entry(id as TClientId + 1).or_default().available = *bal;
        }
        accounts
    }
}
