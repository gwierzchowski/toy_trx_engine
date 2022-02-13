use anyhow::{Result, bail};

use crate::TMoney;

//#[derive(Debug)]
pub struct AccountState {
    available: TMoney, 
    held: TMoney, 
    locked: bool
}

// Implemented manually for better clarity
impl Default for AccountState {
    fn default() -> Self {
        Self { available: 0.0, held: 0.0, locked: false }
    }
}

impl AccountState {
    pub fn with_balance(balance: TMoney) -> Self {
        Self {available: balance, ..Default::default()}
    }

    pub fn print_headers_to_stdout() {
        print!("available,held,total,locked");
    }

    pub fn print_as_csv_to_stdout(&self) {
        print!("{},{},{},{}", self.available, self.held, self.total(), self.locked);
    }

    fn total(&self) -> TMoney {self.available + self.held}

    pub fn deposit(&mut self, amount: TMoney) -> Result<()> {
        self.available += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: TMoney) -> Result<()> {
        if self.available < amount {bail!("Not enough funds")}
        self.available -= amount;
        Ok(())
    }

}
