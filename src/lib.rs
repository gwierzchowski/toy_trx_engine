use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use anyhow::{Result, Context};
use argh::FromArgs;
use csv::ReaderBuilder;

/// Type to store client ID.
pub type TClientId = u16;

/// Type to store transaction ID.
pub type TTrxID = u32;

/// Type to store Money.
pub type TMoney = f64; // TODO: Search if there are some crates dedicated to this (aka Java Money type)

pub mod accounts;
mod processor;
mod transactions;

use accounts::AccountState;

#[derive(FromArgs)]
/// Toy Transaction Engine.
pub struct Args {
    /// use when transactions file has no headers
    #[argh(switch)]
    no_header: bool,
    
    /// use to skip lines that begin with '#' in transaction file
    #[argh(switch)]
    comments: bool,

    /// path to transactions file CSV file with columns (type,client,tx,amount)
    #[argh(positional)]
    trx_file: PathBuf,
}

/// Performs transaction processing based on parameters passed in `Arg` argument, updates passed accounts object.
/// Function separated from `main()` to feature integration tests.
/// See Integration tests in `tests` folder for example usage.
pub fn process(arg:&Args, accounts: &mut HashMap::<TClientId,AccountState>) -> Result<u128> {
    let trx_file = File::open(&arg.trx_file)
        .with_context(|| format!("opening transactions file: {}", arg.trx_file.display()))?;
    let rdr = ReaderBuilder::new()
        .has_headers(!arg.no_header)
        .comment(if arg.comments {Some(b'#')} else {None})
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(trx_file);
    let processed = processor::processing_loop(rdr, accounts)?;
    Ok(processed)
}
