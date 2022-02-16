use std::collections::HashMap;

use anyhow::Result;

use toy_trx_engine:: {
    Args,
    process,
    accounts::AccountState,
};

fn main() -> Result<()> {
    let arg: Args = argh::from_env();
    let mut accounts = HashMap::new();

    let _ = process(&arg, &mut accounts)?;

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
