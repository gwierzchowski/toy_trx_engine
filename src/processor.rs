use std::collections::HashMap;
use std::io::Read;

use anyhow::{Result, bail};
use csv::Reader;

use crate::{
    TClientId,
    accounts::AccountState,
    transactions::{TransactionValid, TransactionRec, Transaction, TransactionInt},
};

/// Main transaction processing loop.
/// Reads transactions from passed `data` and updates `accounts`.
pub fn processing_loop<'r, R>(
    mut data: Reader<R>, 
    accounts: &mut HashMap::<TClientId,AccountState>
)   -> Result<u128>
    where R: Read + 'r 
{
    let mut rec_no = 0u128;
    let mut processed_sucessfully = 0u128;
    for record in data.deserialize() {
        rec_no += 1;
        if let Err(err) = record {
            if rec_no > 1 {
                eprintln!("Record# {} - parsing failed: {}", rec_no, err);
                continue;
            } else {
                bail!(err);
            }
        }
            
        let transaction_rec:TransactionRec = record?;
        let transaction = transaction_rec.try_into();
        if let Err(err) = transaction {
            eprintln!("Record# {} - invalid (will be skipped): {}", rec_no, err);
            continue;
        }

        let transaction:Transaction = transaction?;
        match transaction.validate() {
            TransactionValid::Ok => {},
            TransactionValid::Warn(msg) => {
                eprintln!("Record# {}, Transaction ID = {} - validation warning: {}", rec_no, transaction.id(), msg);
            },
            TransactionValid::Invalid(msg) => {
                eprintln!("Record# {}, Transaction ID = {} - invalid (will be skipped): {}", rec_no, transaction.id(), msg);
                continue;
            }
        }
        
        if let Err(e) = transaction.commit(accounts) {
            eprintln!("Record# {}, Transaction ID = {} - failed: {}", rec_no, transaction.id(), e);
            continue;
        }
        processed_sucessfully += 1;
    }
    Ok(processed_sucessfully)
}