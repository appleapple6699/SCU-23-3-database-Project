use database::prelude::*;
use database::txn::TxnOptions;

fn unique_path(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    p.push(format!("db-{}-{}", name, nanos));
    p
}

#[test]
fn put_get_roundtrip() {
    let cfg = Config { data_dir: unique_path("putget") };
    let db = Database::open(cfg).unwrap();
    db.put(b"k", b"v").unwrap();
    let v = db.get(b"k").unwrap();
    assert_eq!(v, Some(b"v".to_vec()));
}

#[test]
fn txn_commit_visibility() {
    let cfg = Config { data_dir: unique_path("txn") };
    let db = Database::open(cfg).unwrap();
    let mut tx = db.begin_txn(TxnOptions::read_write());
    tx.put(b"a", b"1").unwrap();
    tx.put(b"b", b"2").unwrap();
    assert_eq!(tx.get(b"a").unwrap(), Some(b"1".to_vec()));
    tx.commit().unwrap();
    assert_eq!(db.get(b"a").unwrap(), Some(b"1".to_vec()));
}