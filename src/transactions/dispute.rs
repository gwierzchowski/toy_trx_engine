use anyhow::{Result, bail};

use crate::{TClientId, TTrxID};
use super::*;

/// Represents Dispute transaction.
pub struct Dispute {
    client: TClientId,
    tx: TTrxID,
}

impl TryFrom<TransactionRec> for Dispute {
    type Error = anyhow::Error;
    fn try_from(value: TransactionRec) -> Result<Self, Self::Error> {
        if value.ttype != TransactionRecType::Dispute {
            bail!("Transaction ID {} - Incompatible type expected Dispute", value.tx)
        } else {
            Ok(Self {client: value.client, tx: value.tx})
        }
    }
}


impl TransactionInt for Dispute {
    fn id(&self) -> TTrxID {self.tx}

    fn validate(&self) -> TransactionValid {
        TransactionValid::Ok
    }

    /// Performs Dispute transaction.
    /// - if account is not registered - reject.
    /// - if account is locked - reject.
    /// - if referenced transaction is not registered for given client - reject.
    /// - if referenced transaction is already in 'on dispute' state logs warning but not reject transaction.
    /// - otherwise puts referenced transaction in 'on dispute' state 
    ///   and decreases account `available` property of given `amount`.
    fn commit(&self, accounts:&mut HashMap::<TClientId,AccountState>) -> Result<()> {
        match accounts.get_mut(&self.client) {
            Some(acct) => {
                if acct.locked {
                    bail!("Dispute transaction failed - account locked")
                }
                match acct.transactions.get_mut(&self.tx) {
                    Some((dispute, amount)) if !*dispute => {
                        acct.available -= *amount;
                        acct.held += *amount;
                        *dispute = true;
                        Ok(())
                    },
                    Some(_) => {
                        eprintln!("Transaction ID = {}: warning - repeated Dispute", self.tx);
                        Ok(())
                    },
                    None => bail!("Dispute transaction failed - reference transaction ID not found for given client")
                }
            }
            None => bail!("Dispute transaction failed - client unknown")
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use crate::{
        TClientId,
        transactions::TransactionInt,
    };
    use super::*;
    use super::super::tests::create_accounts;
    
    impl Dispute {
        #[allow(dead_code)]
        pub fn test(client:TClientId, tx:TTrxID) -> Self {
            Self {client, tx}
        }
    }

    #[test]
    fn on_locked() {
        let mut accounts = create_accounts(&[dec!(0.0)]);
        accounts.get_mut(&1).expect("client 1 in test accounts").locked = true;
        let trx = Dispute {client: 1, tx: 1};
        assert!(trx.commit(&mut accounts).is_err());
    }
    
    #[test]
    fn on_normal_deposit() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = deposit::Deposit::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx2 = Dispute {client, tx};
        assert!(trx2.commit(&mut accounts).is_ok());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance - amount, new_balance);
        assert_eq!(old_total, new_total);
    }
    
    #[test]
    fn on_normal_withdrawal() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = withdrawal::Withdrawal::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx2 = Dispute {client, tx};
        assert!(trx2.commit(&mut accounts).is_ok());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance + amount, new_balance);
        assert_eq!(old_total, new_total);
    }
    
    #[test]
    fn unknown_client() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx = Dispute {client: 10, tx: 1};
        assert!(accounts.get(&trx.client).is_none());
        assert!(trx.commit(&mut accounts).is_err());
    }
    
    #[test]
    fn unknown_transaction() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = deposit::Deposit::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx2 = Dispute {client, tx: tx + 1};
        assert!(trx2.commit(&mut accounts).is_err());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance, new_balance);
        assert_eq!(old_total, new_total);
    }
    
    #[test]
    fn second_dispute() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = deposit::Deposit::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx2 = Dispute {client, tx};
        assert!(trx2.commit(&mut accounts).is_ok());
        let trx3 = Dispute {client, tx};
        assert!(trx3.commit(&mut accounts).is_ok());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance - amount, new_balance); // only first dispute affected balance
        assert_eq!(old_total, new_total);
    }
    
    #[test]
    fn on_failed() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let trx1 = withdrawal::Withdrawal::test(client, 1, dec!(1.0));
        assert!(trx1.commit(&mut accounts).is_ok());
        let balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let trx2 = withdrawal::Withdrawal::test(client, 2, balance + dec!(1.0));
        assert!(trx2.commit(&mut accounts).is_err()); // over balance
        let trx3 = Dispute {client, tx: 2};
        assert!(trx3.commit(&mut accounts).is_err()); // dispute to failed transaction
    }
}
