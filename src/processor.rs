use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use anyhow::{Result, bail};
use async_std::stream::StreamExt;
//use async_std::{stream::StreamExt, channel::Receiver};
use std::collections::hash_map::Entry;

use csv_async::AsyncDeserializer;
use futures::io::AsyncRead;

use crate::{
    TClientId,
    accounts::AccountState,
    transactions::{TransactionValid, TransactionRec, Transaction, TransactionInt, TheEnd},
};

/// Main transaction processing loop.
/// Reads transactions from passed `data` and updates `accounts`.
pub async fn processing_loop<'r, R>(
    mut data: AsyncDeserializer<R>, 
    num_workers: usize,
    wrk_buffer_size: usize,
)   -> Result<(u128, HashMap::<TClientId,AccountState>)>
    where R: AsyncRead + Unpin + Send + 'r 
{
    assert!(num_workers > 0);
    
    let mut accounts = HashMap::new();      // this will store accumulated accounts data
    let mut workers = Vec::new();           // this will store pairs (sender channel of worker to send transactions, worker handle)
    let mut cli_to_worker = HashMap::new(); // maps clientID to index in workers vector

    let mut records = data.deserialize::<TransactionRec>();
    let mut rec_no = 0u128;
    let mut wrk_idx = 0usize;
    while let Some(record) = records.next().await {
        rec_no = rec_no.wrapping_add(1);
        if rec_no == 0 {rec_no = 1;} // rec_no 0 means end-of stream by convention.
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
        
        match cli_to_worker.entry(transaction.client_id()) {
            Entry::Vacant(ent) => {
                let wrk_len = workers.len();
                if wrk_len < num_workers {
                    let (tx, rx) = mpsc::sync_channel(wrk_buffer_size * 1000);
                    let jh = thread::spawn(|| process_transactions(rx));
                    workers.push((tx, jh));
                    ent.insert(wrk_len);
                    if let Err(e) = workers[wrk_len].0.send((rec_no, transaction)) {
                        eprintln!("Record# {}, Transaction ID = {} - skipped - internal error in send(): {}", rec_no, e.0.1.id(), e);
                    }
                } else {
                    ent.insert(wrk_idx);
                    if let Err(e) = workers[wrk_idx].0.send((rec_no, transaction)) {
                        eprintln!("Record# {}, Transaction ID = {} - skipped - internal error in send(): {}", rec_no, e.0.1.id(), e);
                    }
                    wrk_idx = (wrk_idx + 1) % num_workers;
                }
            },
            Entry::Occupied(ent) => {
                if let Err(e) = workers[*ent.get()].0.send((rec_no, transaction)) {
                    eprintln!("Record# {}, Transaction ID = {} - skipped - internal error in send(): {}", rec_no, e.0.1.id(), e);
                }
            }
        }
    }

    for i in 0..workers.len() {
        if let Err(_) = workers[i].0.send((0, Transaction::TheEnd(TheEnd{}))) {
            workers.remove(i);
        }
    }

    let mut processed_sucessfully = 0u128;
    for (_, jh) in workers {
        match jh.join() {
            Ok((ps, acct)) => {
                accounts.extend(acct);
                processed_sucessfully = processed_sucessfully.wrapping_add(ps);
            },
            Err(_) => {
                eprintln!("Worker crashed");
            }
        }
    }

    Ok((processed_sucessfully, accounts))
}

fn process_transactions(rx: mpsc::Receiver<(u128, Transaction)>) -> (u128, HashMap::<TClientId,AccountState>) {
    let mut processed_sucessfully = 0u128;
    let mut accounts = HashMap::new();
    loop {
        if let Ok((rec_no, transaction)) = rx.recv() {
            if rec_no == 0 {
                assert!(matches!(transaction, Transaction::TheEnd(_))); 
                break;
            }
            if let Err(e) = transaction.commit(&mut accounts) {
                eprintln!("Record# {}, Transaction ID = {} - failed: {}", rec_no, transaction.id(), e);
                continue;
            }
        } else {
            eprint!("Internal error in process_transactions() - channel broken");
        }
        processed_sucessfully = processed_sucessfully.wrapping_add(1);
    }
    (processed_sucessfully, accounts)
}