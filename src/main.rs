use dotenv::dotenv;
use sp_application_crypto::Ss58Codec;
use std::{ffi::OsString, iter::Product};
use std::path::PathBuf;

use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};

mod api;
mod error;
mod rpc;
mod types;
mod db;
mod key_store;

use api::*;
use rpc::*;
use key_store::*;
use db::*;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        println!("Error: {}", error);
    }
    // Continued program logic goes here...
}

fn cli() -> Command {
    Command::new("wallet-cli")
        .about("A wallet cli, to view balances, transfer tokens and administer token ")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("add-account")
                .about("Adds an account to the wallet")
                .arg(
                    Arg::new("mnemonic")
                        .action(ArgAction::Set)
                        .required(true)
                        .help("mnemonic of the account"),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("get-accounts")
                .about("Views accounts stored in wallet")
        )
        .subcommand(
            Command::new("get-admin-account")
                .about("View admin account")
        )
        .subcommand(
            Command::new("get-balance")
                .about("Get the balance of an account, if no account is specified the balance for the default account is returned")
                .arg(
                    Arg::new("account")
                        .action(ArgAction::Set)
                        .required(false)
                        .help("The address of the account to view the balance for"),
                ),
        )
        .subcommand(
            Command::new("get-default-account")
                .about("View current default account")
        )
        .subcommand(
            Command::new("get-min-fee").about("Gets the current minimum fee for transactions"),
        )
        .subcommand(Command::new("get-total-supply").about("Gets the current total supply"))
        .subcommand(
            Command::new("mint")
                .about("Mints tokens for the specified account, has to be executed as admin")
                .arg(
                    Arg::new("account")
                        .action(ArgAction::Set)
                        .required(true)
                        .help("The address of the account to mint tokens for"),
                )
                .arg(
                    Arg::new("amount")
                        .value_parser(value_parser!(u128))
                        .action(ArgAction::Set)
                        .required(true)
                        .help("The amount to mint"),
                )
                .arg(get_tx_fee_arg())
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("transfer")
                .about("Transfers tokens to the specified account")
                .arg(
                    Arg::new("account")
                        .action(ArgAction::Set)
                        .required(true)
                        .help("The address of the account to transfer tokens to"),
                )
                .arg(
                    Arg::new("amount")
                        .value_parser(value_parser!(u128))
                        .action(ArgAction::Set)
                        .required(true)
                        .help("The amount to transfer"),
                )
                .arg(get_tx_fee_arg())
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("set-min-fee")
                .about("Sets the min fee for transactions, has to be executed as admin")
                .arg(
                    Arg::new("min-fee")
                        .value_parser(value_parser!(u32))
                        .action(ArgAction::Set)
                        .required(true)
                        .help("The amount to set the min fee to"),
                )
                .arg(get_tx_fee_arg())
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("set-default-account")
                .about("Sets the default account to operate with, the account must have been added beforehand")
                .arg(
                    Arg::new("account")
                        .action(ArgAction::Set)
                        .required(true)
                        .help("The address of the account to set as default"),
                )
                .arg_required_else_help(true),
        )
}

async fn run() -> Result<(), error::Error> {
    dotenv()?;
    let matches = cli().get_matches();
    let addr = std::env::var("ADDR")?;
    let db_file = std::env::var("DATABASE_FILE")?;
    let db = DB::new(&db_file)?;
    let api = API::new(RPC::new(&addr).await?, KeyStore::new());

    match matches.subcommand() {
        Some(("add-account", sub_matches)) => {
            let mnemonic = sub_matches
                .get_one::<String>("mnemonic")
                .expect("mnemonic is required");
                let key = api.keystore.add(mnemonic)?;
            println!("Adding account: {}", key.to_ss58check());
            db.add_account(&key.to_ss58check(), mnemonic)?;

        }
        Some(("get-min-fee", _)) => {
            let fee = api.get_min_fee().await?;
            println!("Current min fee is: {}", fee);
        }
        Some(("get-total-supply", _)) => {
            let supply = api.get_supply().await?;
            println!("Current total supply is: {}", supply);
        }
        Some(("get-accounts", _)) => {
            println!("Accounts:");
            for account in db.get_accounts()? {
                println!("{}", account);
            }
        }
        Some(("get-default-account", _)) => {
            println!("Default Account: {}", db.get_default_account()?)
        }
        Some(("get-admin-account", _)) => {
            println!("Admin Account: {}", api.keystore.add(&std::env::var("ADMIN_SEED")?)?.to_ss58check());
        }
        Some(("get-balance", sub_matches)) => {
            let account = get_operating_account(sub_matches, &db)?;
            println!(
                "Balance for account: {} is: {}",
                account, api.get_balance(&account).await?
            );
        }
        Some(("mint", sub_matches)) => {
            let account = sub_matches
                .get_one::<String>("account")
                .expect("account is required");
            let amount = *sub_matches
                .get_one::<u128>("amount")
                .expect("amount is required");
            println!(
                "Minting: {} tokens for account: {} ...",
                amount, account
            );
            api.mint(account, amount).await?;
        }
        Some(("transfer", sub_matches)) => {
            let account = sub_matches
                .get_one::<String>("account")
                .expect("account is required");
            let amount = *sub_matches
                .get_one::<u128>("amount")
                .expect("amount is required");
            let tx_fee = get_tx_fee(sub_matches, &api).await;
            let (default_account, mnemonic) = db.get_default_mnemonic()?;
            println!(
                "Transfering from {}  to {} amount: {} ...",
                default_account, account, amount
            );
            api.transfer(&mnemonic, account, amount, tx_fee).await?;
        }
        Some(("set-default-account", sub_matches)) => {
            let account = sub_matches
                .get_one::<String>("account")
                .expect("account is required");
            println!(
                "Setting account: {} as default ...",
                account
            );
            db.set_default_account(account)?;
        }
        Some(("set-min-fee", sub_matches)) => {
            let min_fee = *sub_matches
                .get_one::<u32>("min-fee")
                .expect("min-fee is required");
            let tx_fee = get_tx_fee(sub_matches, &api).await;
            println!(
                "Setting minimum fee to: {}, tx fee: {} ...",
                min_fee, tx_fee
            );
            api.set_min_fee(min_fee, tx_fee).await?;
        }
        Some(("diff", sub_matches)) => {
            let color = sub_matches
                .get_one::<String>("color")
                .map(|s| s.as_str())
                .expect("defaulted in clap");

            let mut base = sub_matches.get_one::<String>("base").map(|s| s.as_str());
            let mut head = sub_matches.get_one::<String>("head").map(|s| s.as_str());
            let mut path = sub_matches.get_one::<String>("path").map(|s| s.as_str());
            if path.is_none() {
                path = head;
                head = None;
                if path.is_none() {
                    path = base;
                    base = None;
                }
            }
            let base = base.unwrap_or("stage");
            let head = head.unwrap_or("worktree");
            let path = path.unwrap_or("");
            println!("Diffing {}..{} {} (color={})", base, head, path, color);
        }
        Some(("push", sub_matches)) => {
            println!(
                "Pushing to {}",
                sub_matches.get_one::<String>("REMOTE").expect("required")
            );
        }
        Some(("add", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            println!("Adding {:?}", paths);
        }
        Some(("stash", sub_matches)) => {
            let stash_command = sub_matches.subcommand().unwrap_or(("push", sub_matches));
            match stash_command {
                ("apply", sub_matches) => {
                    let stash = sub_matches.get_one::<String>("STASH");
                    println!("Applying {:?}", stash);
                }
                ("pop", sub_matches) => {
                    let stash = sub_matches.get_one::<String>("STASH");
                    println!("Popping {:?}", stash);
                }
                ("push", sub_matches) => {
                    let message = sub_matches.get_one::<String>("message");
                    println!("Pushing {:?}", message);
                }
                (name, _) => {
                    unreachable!("Unsupported subcommand `{}`", name)
                }
            }
        }
        Some((ext, sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            println!("Calling out to {:?} with {:?}", ext, args);
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
    Ok(())
    // Continued program logic goes here...
}

fn get_operating_account(sub_matches: &ArgMatches, db: &DB) ->Result<String, error::Error> {
    if let Some(account) = sub_matches
    .get_one::<String>("account") {
        return Ok(account.to_owned());
    }
    let account = db.get_default_account()?;
    Ok(account)
}
fn get_tx_fee_arg() -> Arg {
    arg!(--txfee <txfee> "Specify desired tx fee, if not specified the minimum is used")
        .value_parser(value_parser!(u32))
}

async fn get_tx_fee(sub_matches: &ArgMatches, api: &API) -> u32 {
    if let Some(tx_fee) = sub_matches.get_one::<u32>("txfee") {
        return *tx_fee;
    }
    api.get_min_fee().await.unwrap_or_default()
}

fn push_args() -> Vec<clap::Arg> {
    vec![arg!(-m --message <MESSAGE>)]
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn print_public_key_address() {
		let key = sp_core::sr25519::Public([212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125]);
        println!("address: {}", key.to_ss58check());
	}
}
