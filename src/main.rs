use anyhow::Result;

use toy_trx_engine:: {
    Args,
    process,
    accounts::AccountState,
};

#[async_std::main]
async fn main() -> Result<()> {
    let arg: Args = argh::from_env();

    let (_, accounts) = process(&arg).await?;

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
