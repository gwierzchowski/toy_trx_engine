use std::collections::{HashMap, HashSet};

use anyhow::{Result, Context, bail};
use async_std::stream::StreamExt;
use csv_async::AsyncDeserializer;
use futures::io::AsyncRead;

use crate::{
    TClientId,
    accounts::AccountState,
    transactions::{TransactionValid, TransactionType, TransactionData, store_transaction},
};

pub async fn processing_loop<'r, R>(
    mut data: AsyncDeserializer<R>, 
    accounts: &mut HashMap::<TClientId,AccountState>
)   -> Result<u128>
    where R: AsyncRead + Unpin + Send + 'r 
{
    let mut transactions = HashSet::new();
    let mut records = data.deserialize::<TransactionData>();
    let mut rec_no = 0u128;
    while let Some(record) = records.next().await {
        rec_no += 1;
        if let Err(err) = record {
            if rec_no > 1 {
                eprintln!("Record# {} - parsing failed: {}", rec_no, err);
                continue;
            } else {
                bail!(err);
            }
        }
            
        let transaction = record?;
        match transaction.validate() {
            TransactionValid::Ok => {},
            TransactionValid::Warn(msg) => {
                eprintln!("Record# {}, Transaction ID = {} - validation warning: {}", rec_no, transaction.tx, msg);
            },
            TransactionValid::Invalid(msg) => {
                eprintln!("Record# {}, Transaction ID = {} - invalid (will be skipped): {}", rec_no, transaction.tx, msg);
                continue;
            }
        }
        
        if let Err(e) = store_transaction(&mut transactions, &transaction) {
            eprintln!("Record# {}, Transaction ID = {} - discarded: {}", rec_no, transaction.tx, e);
            continue;
        }

        match transaction.ttype {
            TransactionType::Deposit => {
                let trx_amount = transaction.amount.unwrap();
                accounts.entry(transaction.client)
                    .and_modify(|acct| (*acct).deposit(trx_amount).context("Deposit transaction")
                        .unwrap_or_else(|e| eprintln!("Transaction ID = {} - failed: {:#}", transaction.tx, e))
                    )
                    .or_insert(AccountState::with_balance(trx_amount));},
            TransactionType::Withdrawal => {
                let trx_amount = transaction.amount.unwrap();
                accounts.entry(transaction.client)
                    .and_modify(|acct| (*acct).withdraw(trx_amount).context("Withdraw transaction")
                    .unwrap_or_else(|e| eprintln!("Transaction ID = {} - failed: {:#}", transaction.tx, e))
                )
                    .or_insert(AccountState::with_balance(-trx_amount));},
            TransactionType::Dispute => {}
        };
    }
    Ok(rec_no)
}