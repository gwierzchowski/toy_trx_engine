use std::path::PathBuf;

use anyhow::{Result, Context};
use argh::FromArgs;
use async_std::fs::File;
use async_std::stream::StreamExt;
//use csv_async::{AsyncReaderBuilder, AsyncDeserializer};
// use csv::{ReaderBuilder, Deserializer};
use csv::{ReaderBuilder};
use serde::Deserialize;

#[derive(FromArgs)]
/// Toy Transaction Engine.
struct Args {
    #[argh(positional)]
    trx_file: PathBuf,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
}
#[derive(Deserialize, Debug)]
struct Transaction {
    #[serde(rename = "type")]
    ttype: TransactionType,
    client: u16,
    tx: u32,
    #[serde(default)]
    amount: f64,
    //amount: Option<f64>,
}

#[async_std::main]
async fn main() -> Result<()> {
    let arg: Args = argh::from_env();
    // let trx_file = File::open(arg.trx_file).await.context("opening transactions file")?;
    let trx_file = std::fs::File::open(arg.trx_file).context("opening transactions file")?;
    //let mut rdr = AsyncDeserializer::from_reader(trx_file);
    // let mut rdr = AsyncReaderBuilder::new()
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(trx_file);

//    let mut balances = std::collections::HashMap<u16, f64>
    let mut balances = std::collections::HashMap::new();
    // let mut records = rdr.deserialize::<Transaction>();
    // while let Some(record) = records.next().await {
    //     let record = record?;
    //     dbg!(record);
    // }
    for result in rdr.deserialize() {
        let record: Transaction = result?;
        //dbg!(record);
        match record.ttype {
            TransactionType::Deposit => 
                balances.entry(record.client)
                    .and_modify(|amt| *amt += record.amount)
                    .or_insert(record.amount),
            TransactionType::Withdrawal => 
                balances.entry(record.client)
                    .and_modify(|amt| *amt -= record.amount)
                    .or_insert(-record.amount),
            };
    }
    for (client, amount) in balances {
        println!("Client: {} - balance: {}", client, amount);
    }
    Ok(())
}
