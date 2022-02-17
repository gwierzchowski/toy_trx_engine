use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Result, Context};
use argh::FromArgs;
use async_std::fs::File;
use csv_async::AsyncReaderBuilder;

/// Type to store client ID.
pub type TClientId = u16;

/// Type to store transaction ID.
pub type TTrxID = u32;

/// Type to store Money.
pub type TMoney = rust_decimal::Decimal;

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
    
    /// number of workers to process transactions (default: CPU cores)
    #[argh(option, default = "num_cpus::get()")]
    wrk_num: usize,
    
    /// buffer size x1000 for worker queue until it blocks (default: 10)
    #[argh(option, default = "10")]
    wrk_buff: usize,

    /// path to transactions CSV file with columns (type,client,tx,amount)
    #[argh(positional)]
    trx_file: PathBuf,
}

/// Performs transaction processing based on parameters passed in `Arg` argument, updates passed accounts object.
/// Function separated from `main()` to feature integration tests.
/// See Integration tests in `tests` folder for example usage.
pub async fn process(arg:&Args) -> Result<(u128, HashMap::<TClientId,AccountState>)> {
    let trx_file = File::open(&arg.trx_file).await
        .with_context(|| format!("opening transactions file: {}", arg.trx_file.display()))?;
    let rdr = AsyncReaderBuilder::new()
        .has_headers(!arg.no_header)
        .comment(if arg.comments {Some(b'#')} else {None})
        .trim(csv_async::Trim::All)
        .flexible(true)
        .create_deserializer(trx_file);

    processor::processing_loop(rdr, arg.wrk_num, arg.wrk_buff).await
}
