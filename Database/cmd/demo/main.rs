use database::prelude::*;
use database::txn::TxnOptions;

fn main() -> Result<()> {
    let db = Database::open(Config::default())?;
    db.put(b"users:1", br#"{"name":"Alice"}"#)?;
    let v = db.get(b"users:1")?;
    if let Some(val) = v { println!("{}", String::from_utf8_lossy(&val)); }
    let mut tx = db.begin_txn(TxnOptions::read_write());
    tx.put(b"balance:alice", b"100")?;
    tx.put(b"balance:bob", b"50")?;
    tx.commit()?;
    Ok(())
}