use std::collections::hash_map::Entry;

use anyhow::{Result, Context, bail};

use crate::{TClientId, TMoney, TTrxID};
use super::*;

/// Represents Deposit transaction.
pub struct Deposit {
    client: TClientId,
    tx: TTrxID,
    amount: TMoney,
}

impl TryFrom<TransactionRec> for Deposit {
    type Error = anyhow::Error;
    fn try_from(value: TransactionRec) -> std::result::Result<Self, Self::Error> {
        if value.ttype != TransactionRecType::Deposit {
            bail!("Transaction ID {} - Incompatible type expected Deposit", value.tx)
        } 
        else if let Some(amount) = value.amount {
            Ok(Self {
                client: value.client, 
                tx: value.tx, 
                amount: amount.try_into()
                    .with_context(|| format!("Transaction ID {} - Parsing float value: {}", value.tx, amount))?
            })
        } else {
            bail!("Transaction ID {} - Amount is missing in Deposit transaction", value.tx)
        }
    }
}

#[async_trait]
impl TransactionInt for Deposit {
    fn id(&self) -> TTrxID {self.tx}

    fn validate(&self) -> TransactionValid {
        if self.amount.is_sign_positive() {
            TransactionValid::Ok
        } else if self.amount.is_zero() {
            TransactionValid::Warn("Amount == 0 in Deposit transaction")
        } else {
            TransactionValid::Invalid("Amount < 0 in Deposit transaction")
        }
    }

    /// Performs Deposit transaction.
    /// - if account is not registered - register it with passed initial balance (`available` property).
    /// - if account is locked - reject.
    /// - if there is already registered transaction with the same ID - reject.
    /// - otherwise increase account `available` property of given `amount` and stores transaction amount.
    async fn commit(&self, accounts:&mut HashMap::<TClientId,AccountState>) -> Result<()> {
        match accounts.get_mut(&self.client) {
            Some(acct) => {
                if acct.locked {
                    bail!("Deposit transaction failed - account locked")
                } else if let Entry::Vacant(ent) = acct.transactions.entry(self.tx) {
                    acct.available += self.amount;
                    ent.insert((false, self.amount));
                    Ok(())
                } else {
                    bail!("Deposit transaction failed - duplicated transaction ID")
                }
            }
            None => {
                accounts.entry(self.client)
                    .or_insert_with(|| AccountState::with_balance(self.amount))
                    .transactions.insert(self.tx, (false, self.amount));
                Ok(())
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use crate::transactions::TransactionInt;
    use super::*;
    use super::super::tests::create_accounts;

    impl Deposit {
        pub fn test(client:TClientId, tx:TTrxID, amount:TMoney) -> Self {
            Self {client, tx, amount}
        }
    }

    #[async_std::test]
    async fn on_locked() {
        let mut accounts = create_accounts(&[dec!(0.0)]);
        accounts.get_mut(&1).expect("client 1 in test accounts").locked = true;
        let trx = Deposit {client: 1, tx: 1, amount: dec!(1.0)};
        assert!(trx.commit(&mut accounts).await.is_err());
    }
    
    #[async_std::test]
    async fn on_normal() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx = Deposit {client: 1, tx: 1, amount: dec!(1.0)};
        let old_balance = accounts.get(&trx.client).expect("client 1 in test accounts").available;
        assert!(trx.commit(&mut accounts).await.is_ok());
        let new_balance = accounts.get(&trx.client).expect("client 1 in test accounts").available;
        assert_eq!(old_balance + trx.amount, new_balance);
    }
    
    #[async_std::test]
    async fn new_client() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx = Deposit {client: 10, tx: 1, amount: dec!(1.0)};
        assert!(accounts.get(&trx.client).is_none());
        assert!(trx.commit(&mut accounts).await.is_ok());
        let new_balance = accounts.get(&trx.client).expect("new client in test accounts").available;
        assert_eq!(trx.amount, new_balance);
    }
    
    #[async_std::test]
    async fn duplicated_tx_id() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx1 = Deposit {client: 1, tx: 1, amount: dec!(0.1)};
        assert!(trx1.commit(&mut accounts).await.is_ok());
        let trx2 = Deposit {client: 1, tx: 1, amount: dec!(0.1)};
        assert!(trx2.commit(&mut accounts).await.is_err()); // duplicated id
        let trx3 = Deposit {client: 1, tx: 2, amount: dec!(0.1)};
        assert!(trx3.commit(&mut accounts).await.is_ok());
    }
}
