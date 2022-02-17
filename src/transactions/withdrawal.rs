use std::collections::hash_map::Entry;

use anyhow::{Result, Context, bail};

use crate::{TClientId, TMoney, TTrxID};
use super::*;

/// Represents Withdrawal transaction.
pub struct Withdrawal {
    client: TClientId,
    tx: TTrxID,
    amount: TMoney,
}

#[cfg(test)]
impl Withdrawal {
    #[allow(dead_code)]
    pub fn test(client:TClientId, tx:TTrxID, amount:TMoney) -> Self {
        Self {client, tx, amount}
    }
}

impl TryFrom<TransactionRec> for Withdrawal {
    type Error = anyhow::Error;
    fn try_from(value: TransactionRec) -> std::result::Result<Self, Self::Error> {
        if value.ttype != TransactionRecType::Withdrawal {
            bail!("Transaction ID {} - Incompatible type expected Withdrawal", value.tx)
        } 
        else if let Some(amount) = value.amount {
            Ok(Self {
                
                client: value.client, 
                tx: value.tx, 
                amount: amount.try_into()
                    .with_context(|| format!("Transaction ID {} - Parsing float value: {}", value.tx, amount))?
            })
        } else {
            bail!("Transaction ID {} - Amount is missing in Withdrawal transaction", value.tx)
        }
    }
}

impl TransactionInt for Withdrawal {
    fn id(&self) -> TTrxID {self.tx}

    fn client_id(&self) -> TClientId {self.client}

    fn validate(&self) -> TransactionValid {
        if self.amount.is_sign_positive() {
            TransactionValid::Ok
        } else if self.amount.is_zero() {
            TransactionValid::Warn("Amount == 0 in Withdrawal transaction")
        } else {
            TransactionValid::Invalid("Amount < 0 in Withdrawal transaction")
        }
    }

    /// Performs Withdrawal transaction.
    /// - if account is not registered - reject.
    /// - if account is locked - reject.
    /// - if there is already registered transaction with the same ID - reject.
    /// - if account's `available` property is less then `amount` - reject.
    /// - otherwise decrease account `available` property of given `amount` and stores transaction amount (as negative value).
    fn commit(&self, accounts:&mut HashMap::<TClientId,AccountState>) -> Result<()> {
        match accounts.get_mut(&self.client) {
            Some(acct) => {
                if acct.locked {
                    bail!("Withdrawal transaction failed - account locked")
                } else if acct.available >= self.amount {
                    if let Entry::Vacant(ent) = acct.transactions.entry(self.tx) {
                        acct.available -= self.amount;
                        ent.insert((false, -self.amount));
                        Ok(())
                    } else {
                        bail!("Withdrawal transaction failed - duplicated transaction ID")
                    }
                } else {
                    bail!("Withdrawal transaction failed - not enough funds")
                }
            }
            None => bail!("Withdrawal transaction failed - client unknown")
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use crate::transactions::TransactionInt;
    use super::*;
    use super::super::tests::create_accounts;

    #[test]
    fn on_locked() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        accounts.get_mut(&1).expect("client 1 in test accounts").locked = true;
        let trx = Withdrawal {client: 1, tx: 1, amount: dec!(1.0)};
        assert!(trx.commit(&mut accounts).is_err());
    }
    
    #[test]
    fn on_normal() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx = Withdrawal {client: 1, tx: 1, amount: dec!(1.0)};
        let old_balance = accounts.get(&trx.client).expect("client 1 in test accounts").available;
        assert!(trx.commit(&mut accounts).is_ok());
        let new_balance = accounts.get(&trx.client).expect("client 1 in test accounts").available;
        assert_eq!(old_balance - trx.amount, new_balance);
    }
    
    #[test]
    fn unknown_client() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx = Withdrawal {client: 10, tx: 1, amount: dec!(1.0)};
        assert!(accounts.get(&trx.client).is_none());
        assert!(trx.commit(&mut accounts).is_err());
    }
    
    #[test]
    fn over_balance() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let mut trx = Withdrawal {client: 1, tx: 1, amount: dec!(1.0)};
        let old_balance = accounts.get(&trx.client).expect("client 1 in test accounts").available;
        trx.amount = old_balance + dec!(0.1);
        assert!(trx.commit(&mut accounts).is_err());
    }
    
    #[test]
    fn duplicated_tx_id() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx1 = Withdrawal {client: 1, tx: 1, amount: dec!(0.1)};
        assert!(trx1.commit(&mut accounts).is_ok());
        let trx2 = Withdrawal {client: 1, tx: 1, amount: dec!(0.1)};
        assert!(trx2.commit(&mut accounts).is_err()); // duplicated id
        let trx3 = Withdrawal {client: 1, tx: 2, amount: dec!(0.1)};
        assert!(trx3.commit(&mut accounts).is_ok());
    }
    
    #[test]
    fn roundings() {
        let mut accounts = create_accounts(&[dec!(0.0)]);
        let client = 1; 
        let mut id = 1;
        let trx = deposit::Deposit::test(client, id, dec!(10.1)); id += 1;
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(10.1), accounts.get(&client).expect("client 1 in test accounts").total());
        let trx = deposit::Deposit::test(client, id, dec!(10.2)); id += 1;
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(20.3), accounts.get(&client).expect("client 1 in test accounts").total());
        let trx = Withdrawal {client, tx: id, amount: dec!(0.33)};
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(19.97), accounts.get(&client).expect("client 1 in test accounts").total());
    }
}
