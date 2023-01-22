use crate::error::Error;
use sqlite::{Connection, State, Statement, Value};
use std::fs;
use std::path::Path;

pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new(db_file: &str) -> Result<DB, Error> {
        let path = Path::new(db_file);
        if !path.exists() {
            fs::create_dir_all(
                path.parent()
                    .ok_or(Error::new("Directory for the db file wasn't specified"))?,
            )?;
        }
        let conn = sqlite::open(path)?;
        let query = "
        CREATE TABLE IF NOT EXISTS config(current_address TEXT,
          CONSTRAINT config_pk PRIMARY KEY (current_address)
        );
        CREATE TABLE IF NOT EXISTS accounts(address TEXT, mnemonic TEXT,
          CONSTRAINT accounts_pk PRIMARY KEY (address)
        );
      ";
        conn.execute(query)?;

        Ok(DB { conn })
    }

    pub fn add_account(&self, address: &str, mnemonic: &str) -> Result<(), Error> {
        let query = "
      INSERT INTO accounts(address, mnemonic) 
      VALUES(:address,:mnemonic)
      ";
        let mut statement = self.conn.prepare(query)?;
        statement.bind::<&[(&str, Value)]>(
            &[(":address", address.into()), (":mnemonic", mnemonic.into())][..],
        )?;
        Self::execute_prepared_statement(&mut statement);
        self.set_default_account(address)?;
        Ok(())
    }

    pub fn execute_prepared_statement(statement: &mut Statement) {
        while let Ok(State::Row) = statement.next() {}
    }

    pub fn get_accounts(&self) -> Result<Vec<String>, Error> {
        let mut accounts = Vec::new();
        let query = "SELECT address from accounts";
        let mut statement = self.conn.prepare(query)?;
        while let Ok(State::Row) = statement.next() {
            accounts.push(statement.read::<String, _>("address")?);
        }
        Ok(accounts)
    }

    pub fn find_account(&self, address: &str) -> Result<Option<(String, String)>, Error> {
        let query = "SELECT address, mnemonic from accounts where address = :address";
        let mut statement = self.conn.prepare(query)?;
        statement.bind((":address", address))?;
        while let Ok(State::Row) = statement.next() {
            return Ok(Some((
                address.to_owned(),
                statement.read::<String, _>("mnemonic")?,
            )));
        }
        Ok(None)
    }

    pub fn get_account(&self, address: &str) -> Result<(String, String), Error> {
        let account = self
            .find_account(address)?
            .ok_or(Error::new("Account not found"))?;
        Ok(account)
    }

    pub fn find_default_account(&self) -> Result<Option<String>, Error> {
        let query = "SELECT current_address from config";
        let mut statement = self.conn.prepare(query)?;
        while let Ok(State::Row) = statement.next() {
            return Ok(Some(statement.read::<String, _>("current_address")?));
        }
        Ok(None)
    }

    pub fn get_default_account(&self) -> Result<String, Error> {
        let account = self
            .find_default_account()?
            .ok_or(Error::new("Default account not set"))?;
        Ok(account)
    }

    pub fn get_default_mnemonic(&self) -> Result<(String, String), Error> {
        let account = self.get_default_account()?;
        let account = self.get_account(&account)?;
        Ok(account)
    }

    pub fn set_default_account(&self, address: &str) -> Result<(), Error> {
        self.get_account(address)?;
        let query = format!(
            "
      DELETE FROM config;
      INSERT INTO config(current_address) 
      VALUES('{}');
      ",
            address
        );
        self.conn.execute(query)?;
        Ok(())
    }
}
