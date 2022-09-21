mod dto;
mod errors;
mod repo;
mod transaction;

use color_eyre::{
    eyre::{bail, Context},
    Result,
};
use csv::Trim;
use repo::Repository;
use std::{convert::TryInto, env, fs::File};
use transaction::Transaction;

use crate::dto::{InputRecord, OutputRecord};

fn print_usage() {
    let bin = env!("CARGO_BIN_NAME");
    eprintln!("USAGE: {} INPUT_PATH", bin);
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        print_usage();
        bail!("Incorrect number of arguments");
    }

    let mut repo = Repository::new();

    let file = File::open(args[1].as_str())
        .wrap_err_with(|| format!("Can not open file `{}`", args[1].as_str()))?;
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_reader(file);
    for result in rdr.deserialize() {
        let record: InputRecord = result?;
        let transaction: Transaction = record.try_into()?;

        let result = repo.register_transaction(transaction);
        if let Err(e) = result {
            eprintln!("ERROR: {}", e)
        }
    }

    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for c in repo.iter_clients() {
        let or: OutputRecord = c.into();
        wtr.serialize(or)?;
    }

    Ok(())
}
