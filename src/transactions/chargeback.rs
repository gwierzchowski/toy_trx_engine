use anyhow::{Result, bail};

use crate::{TClientId, TTrxID};
use super::*;

/// Represents Chargeback transaction.
pub struct Chargeback {
    client: TClientId,
    tx: TTrxID,
}

impl TryFrom<TransactionRec> for Chargeback {
    type Error = anyhow::Error;
    fn try_from(value: TransactionRec) -> Result<Self, Self::Error> {
        if value.ttype != TransactionRecType::Chargeback {
            bail!("Transaction ID {} - Incompatible type expected Chargeback", value.tx)
        } else {
            Ok(Self {client: value.client, tx: value.tx})
        }
    }
}

#[async_trait]
impl TransactionInt for Chargeback {
    fn id(&self) -> TTrxID {self.tx}

    fn validate(&self) -> TransactionValid {
        TransactionValid::Ok
    }

    /// Performs Chargeback transaction.
    /// - if account is not registered - reject.
    /// - if account is locked - reject.
    /// - if referenced transaction is not registered for given client - reject.
    /// - if referenced transaction is not 'on dispute' state - reject.
    /// - otherwise releases referenced transaction from 'on dispute' state, 
    ///   releases money reserved on case of Resolve transaction and locks account.
    async fn commit(&self, accounts:&mut HashMap::<TClientId,AccountState>) -> Result<()> {
        match accounts.get_mut(&self.client) {
            Some(acct) => {
                if acct.locked {
                    bail!("Chargeback transaction failed - account locked")
                }
                match acct.transactions.get_mut(&self.tx) {
                    Some((dispute, amount)) if *dispute => {
                        acct.held -= *amount;
                        *dispute = false;
                        acct.locked = true;
                        Ok(())
                    },
                    Some(_) => bail!("Chargeback transaction failed - not disputed transaction"),
                    None => bail!("Chargeback transaction failed - reference transaction ID not found for given client")
                }
            }
            None => bail!("Chargeback transaction failed - client unknown")
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use crate::transactions::TransactionInt;
    use super::*;
    use super::super::tests::create_accounts;

    #[async_std::test]
    async fn on_locked() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        accounts.get_mut(&1).expect("client 1 in test accounts").locked = true;
        let trx = Chargeback {client: 1, tx: 1};
        assert!(trx.commit(&mut accounts).await.is_err());
    }
    
    #[async_std::test]
    async fn on_normal_deposit() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = deposit::Deposit::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).await.is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx2 = dispute::Dispute::try_from(
            TransactionRec {
                ttype:TransactionRecType::Dispute,
                client,
                tx,
                amount: None
            }).expect("Dispute transaction from transaction record");
        assert!(trx2.commit(&mut accounts).await.is_ok());
        let trx3 = Chargeback {client, tx};
        assert!(trx3.commit(&mut accounts).await.is_ok());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance - amount, new_balance);
        assert_eq!(old_total - amount, new_total);
        assert!(accounts.get(&client).expect("client 1 in test accounts").locked);
    }
    
    #[async_std::test]
    async fn on_normal_withdrawal() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = withdrawal::Withdrawal::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).await.is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx2 = dispute::Dispute::try_from(
            TransactionRec {
                ttype:TransactionRecType::Dispute,
                client,
                tx,
                amount: None
            }).expect("Dispute transaction from transaction record");
        assert!(trx2.commit(&mut accounts).await.is_ok());
        let trx3 = Chargeback {client, tx};
        assert!(trx3.commit(&mut accounts).await.is_ok());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance + amount, new_balance);
        assert_eq!(old_total + amount, new_total);
        assert!(accounts.get(&client).expect("client 1 in test accounts").locked);
    }
    
    #[async_std::test]
    async fn unknown_client() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx = Chargeback {client: 10, tx: 1};
        assert!(accounts.get(&trx.client).is_none());
        assert!(trx.commit(&mut accounts).await.is_err());
    }
    
    #[async_std::test]
    async fn unknown_transaction() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = deposit::Deposit::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).await.is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx2 = Chargeback {client, tx: tx + 1};
        assert!(trx2.commit(&mut accounts).await.is_err());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance, new_balance);
        assert_eq!(old_total, new_total);
        assert!(! accounts.get(&client).expect("client 1 in test accounts").locked);
    }
    
    #[async_std::test]
    async fn chargeback_without_dispute() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = withdrawal::Withdrawal::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).await.is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx3 = Chargeback {client, tx};
        assert!(trx3.commit(&mut accounts).await.is_err());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance, new_balance);
        assert_eq!(old_total, new_total);
        assert!(! accounts.get(&client).expect("client 1 in test accounts").locked);
    }
}
