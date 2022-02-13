use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Result, Context};
use argh::FromArgs;
use async_std::fs::File;
use csv_async::{AsyncReaderBuilder};

pub type TClientId = u16;
pub type TTrxID = u32;
pub type TMoney = f64;

mod accounts;
mod processor;
mod transactions;

use accounts::AccountState;

#[derive(FromArgs)]
/// Toy Transaction Engine.
struct Args {
    /// use when transactions file has no headers
    #[argh(switch)]
    no_header: bool,

    /// path to transactions file CSV file with columns (type,client,tx,amount)
    #[argh(positional)]
    trx_file: PathBuf,
}

#[async_std::main]
async fn main() -> Result<()> {
    let arg: Args = argh::from_env();
    let trx_file = File::open(arg.trx_file).await.context("opening transactions file")?;
    let rdr = AsyncReaderBuilder::new()
        .has_headers(!arg.no_header)
        .trim(csv_async::Trim::All)
        .flexible(true)
        .create_deserializer(trx_file);

    let mut accounts = HashMap::new();

    let _ = processor::processing_loop(
        rdr, 
        &mut accounts
    ).await?;

    print!("client,");
    AccountState::print_headers_to_stdout();
    println!();
    for (client, account) in accounts {
        print!("{},", client);
        account.print_as_csv_to_stdout();
        println!();
    }
    Ok(())
}
