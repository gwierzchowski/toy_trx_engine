use anyhow::{Result, bail};

use crate::{TClientId, TTrxID};
use super::*;

/// Represents Resolve transaction.
pub struct Resolve {
    client: TClientId,
    tx: TTrxID,
}

impl TryFrom<TransactionRec> for Resolve {
    type Error = anyhow::Error;
    fn try_from(value: TransactionRec) -> Result<Self, Self::Error> {
        if value.ttype != TransactionRecType::Resolve {
            bail!("Transaction ID {} - Incompatible type expected Resolve", value.tx)
        } else {
            Ok(Self {client: value.client, tx: value.tx})
        }
    }
}


impl TransactionInt for Resolve {
    fn id(&self) -> TTrxID {self.tx}

    fn validate(&self) -> TransactionValid {
        TransactionValid::Ok
    }

    /// Performs Resolve transaction.
    /// - if account is not registered - reject.
    /// - if account is locked - reject.
    /// - if referenced transaction is not registered for given client - reject.
    /// - if referenced transaction is not 'on dispute' state - reject.
    /// - otherwise releases referenced transaction from 'on dispute' state and re-apply transaction.
    fn commit(&self, accounts:&mut HashMap::<TClientId,AccountState>) -> Result<()> {
        match accounts.get_mut(&self.client) {
            Some(acct) => {
                if acct.locked {
                    bail!("Resolve transaction failed - account locked")
                }
                match acct.transactions.get_mut(&self.tx) {
                    Some((dispute, amount)) if *dispute => {
                        acct.available += *amount;
                        acct.held -= *amount;
                        *dispute = false;
                        Ok(())
                    },
                    Some(_) => bail!("Resolve transaction failed - not disputed transaction"),
                    None => bail!("Resolve transaction failed - reference transaction ID not found for given client")
                }
            }
            None => bail!("Resolve transaction failed - client unknown")
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
        let mut accounts = create_accounts(&[dec!(0.0)]);
        accounts.get_mut(&1).expect("client 1 in test accounts").locked = true;
        let trx = Resolve {client: 1, tx: 1};
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
        // Below raises strange error:
        // error[E0223]: ambiguous associated type
        // help: use fully-qualified syntax: `<transactions::dispute::Dispute as Trait>::test`
        // But last also do not work:
        // error[E0658]: usage of qualified paths in this context is experimental
        // note: see issue #86935 <https://github.com/rust-lang/rust/issues/86935> for more information
        // Rust dec!(1.5)8.1
        // let trx2:dispute::Dispute = dispute::Dispute::test{client, tx};
        let trx2 = dispute::Dispute::try_from(
            TransactionRec {
                ttype:TransactionRecType::Dispute,
                client,
                tx,
                amount: None
            }).expect("Dispute transaction from transaction record");
        assert!(trx2.commit(&mut accounts).is_ok());
        let trx3 = Resolve {client, tx};
        assert!(trx3.commit(&mut accounts).is_ok());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance, new_balance);
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
        let trx2 = dispute::Dispute::try_from(
            TransactionRec {
                ttype:TransactionRecType::Dispute,
                client,
                tx,
                amount: None
            }).expect("Dispute transaction from transaction record");
        assert!(trx2.commit(&mut accounts).is_ok());
        let trx3 = Resolve {client, tx};
        assert!(trx3.commit(&mut accounts).is_ok());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance, new_balance);
        assert_eq!(old_total, new_total);
    }
    
    #[test]
    fn unknown_client() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let trx = Resolve {client: 10, tx: 1};
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
        let trx2 = Resolve {client, tx: tx + 1};
        assert!(trx2.commit(&mut accounts).is_err());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance, new_balance);
        assert_eq!(old_total, new_total);
    }
    
    #[test]
    fn second_resolve() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = deposit::Deposit::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx2 = dispute::Dispute::try_from(
            TransactionRec {
                ttype:TransactionRecType::Dispute,
                client,
                tx,
                amount: None
            }).expect("Dispute transaction from transaction record");
        assert!(trx2.commit(&mut accounts).is_ok());
        let trx3 = Resolve {client, tx};
        assert!(trx3.commit(&mut accounts).is_ok());
        let trx4 = Resolve {client, tx};
        assert!(trx4.commit(&mut accounts).is_err());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance, new_balance);
        assert_eq!(old_total, new_total);
    }
    
    #[test]
    fn resolve_without_dispute() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1;
        let tx = 1;
        let amount = dec!(1.5);
        let trx1 = withdrawal::Withdrawal::test(client, tx, amount);
        assert!(trx1.commit(&mut accounts).is_ok());
        
        let old_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let old_total = accounts.get(&client).expect("client 1 in test accounts").total();
        let trx3 = Resolve {client, tx};
        assert!(trx3.commit(&mut accounts).is_err());
        let new_balance = accounts.get(&client).expect("client 1 in test accounts").available;
        let new_total = accounts.get(&client).expect("client 1 in test accounts").total();
        assert_eq!(old_balance, new_balance);
        assert_eq!(old_total, new_total);
    }
    
    #[test]
    fn several_disputes() {
        let mut accounts = create_accounts(&[dec!(2.0)]);
        let client = 1; // initial balance dec!(2.0)
        let trx = deposit::Deposit::test(client, 1, dec!(10.1));
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(12.1), accounts.get(&client).expect("client 1 in test accounts").total());
        let trx = deposit::Deposit::test(client, 2, dec!(10.2));
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(22.3), accounts.get(&client).expect("client 1 in test accounts").total());
        
        let trx = dispute::Dispute::try_from(
            TransactionRec {
                ttype:TransactionRecType::Dispute,
                client,
                tx: 1,
                amount: None
            }).expect("Dispute transaction from transaction record");
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(22.3), accounts.get(&client).expect("client 1 in test accounts").total());
        let trx = dispute::Dispute::try_from(
            TransactionRec {
                ttype:TransactionRecType::Dispute,
                client,
                tx: 2,
                amount: None
            }).expect("Dispute transaction from transaction record");
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(22.3), accounts.get(&client).expect("client 1 in test accounts").total());
        let trx = Resolve {client, tx: 2};
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(22.3), accounts.get(&client).expect("client 1 in test accounts").total());
        let trx = Resolve {client, tx: 1};
        assert!(trx.commit(&mut accounts).is_ok());
        assert_eq!(dec!(22.3), accounts.get(&client).expect("client 1 in test accounts").total());
    }
}
