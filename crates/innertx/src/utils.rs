use alloc::format;
use core::fmt::Debug;

use alloy_primitives::{BlockHash, TxHash};
use alloy_rlp::{decode_exact, encode, Encodable};
use eyre::Report;
use once_cell::sync::OnceCell;

use crate::{
    innertx_inspector::InternalTransaction,
    structs::{BlockTable, DBTables, TxTable},
};
use reth_db::{
    create_db,
    mdbx::{Database, DatabaseArguments, Transaction, WriteFlags, RW},
    table::Table,
    DatabaseEnv,
};

static XLAYERDB: OnceCell<DatabaseEnv> = OnceCell::new();

pub fn initialize(db_path: &str) -> Result<(), Report> {
    let db_dir = format!("{}/{}", db_path, "xlayerdb");
    let db_create_result = create_db(&db_dir, DatabaseArguments::default());
    if let Err(e) = db_create_result {
        return Err(e.wrap_err(format!("xlayerdb creation failed at path {}", db_dir)));
    }

    let mut db = db_create_result.unwrap();

    let tables_create_result = db.create_and_track_tables_for::<DBTables>();
    if let Err(err) = tables_create_result {
        return Err(Into::<Report>::into(err).wrap_err("xlayerdb tables creation failed"));
    }

    let db_set_result = XLAYERDB.set(db);
    if db_set_result.is_err() {
        return Err(Report::msg("xlayerdb was initialized more than once"));
    }

    Ok(())
}
