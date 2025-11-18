use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom, BufReader};
use std::path::Path;

pub struct Wal {
    append: File,
    path: String,
}

impl Wal {
    pub fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let append = OpenOptions::new().create(true).append(true).read(true).open(path)?;
        Ok(Self { append, path: path.to_string_lossy().to_string() })
    }

    pub fn replay(&mut self, map: &mut HashMap<Vec<u8>, Vec<u8>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.append.flush()?;
        let mut f = OpenOptions::new().read(true).open(&self.path)?;
        f.seek(SeekFrom::Start(0))?;
        let mut r = BufReader::new(f);
        loop {
            let mut op = [0u8;1];
            if r.read_exact(&mut op).is_err() { break; }
            let kl = read_u32(&mut r)? as usize;
            let vl = read_u32(&mut r)? as usize;
            let mut k = vec![0u8; kl];
            r.read_exact(&mut k)?;
            if op[0] == 1 {
                let mut v = vec![0u8; vl];
                r.read_exact(&mut v)?;
                map.insert(k, v);
            } else if op[0] == 2 {
                map.remove(&k);
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn append_put(&mut self, key: &[u8], value: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.append.write_all(&[1u8])?;
        write_u32(&mut self.append, key.len() as u32)?;
        write_u32(&mut self.append, value.len() as u32)?;
        self.append.write_all(key)?;
        self.append.write_all(value)?;
        self.append.flush()?;
        Ok(())
    }

    pub fn append_delete(&mut self, key: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.append.write_all(&[2u8])?;
        write_u32(&mut self.append, key.len() as u32)?;
        write_u32(&mut self.append, 0u32)?;
        self.append.write_all(key)?;
        self.append.flush()?;
        Ok(())
    }
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let mut b = [0u8;4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn write_u32<W: Write>(w: &mut W, v: u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    w.write_all(&v.to_le_bytes())?;
    Ok(())
}