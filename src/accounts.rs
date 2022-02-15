use std::collections::HashMap;

use crate::{TMoney, TTrxID};

/// Represents state of Client Account
pub struct AccountState {
    
    /// Funds available for withdraw
    pub available: TMoney, 
    
    /// Founds Locked on account
    pub held: TMoney, 
    
    /// Account is locked
    pub locked: bool,
    
    /// List of transactions
    /// Value stands for pair (trx is under dispute, trx amount (negative if withdrawal))
    pub transactions: HashMap<TTrxID, (bool,TMoney)>,
}

// Implemented manually for better clarity
impl Default for AccountState {
    fn default() -> Self {
        Self { available: 0.0, held: 0.0, locked: false, transactions: HashMap::new() }
    }
}

impl AccountState {
    /// Creates Account object giving its `available` property a value.
    pub fn with_balance(balance: TMoney) -> Self {
        Self {available: balance, ..Default::default()}
    }

    /// Print a header line for the data to `stdout` - don't add newline.
    pub fn print_headers_to_stdout() {
        print!("available,held,total,locked");
    }

    /// Print a record of data to `stdout` - don't add newline.
    pub fn print_as_csv_to_stdout(&self) {
        print!("{},{},{},{}", self.available, self.held, self.total(), self.locked);
    }

    /// Returns total balance of account (sum of available and locked amounts).
    pub fn total(&self) -> TMoney {self.available + self.held}

    // pub fn deposit(&mut self, amount: TMoney) -> Result<()> {
    //     self.available += amount;
    //     Ok(())
    // }

    // pub fn withdraw(&mut self, amount: TMoney) -> Result<()> {
    //     if self.available < amount {bail!("Not enough funds")}
    //     self.available -= amount;
    //     Ok(())
    // }

}
