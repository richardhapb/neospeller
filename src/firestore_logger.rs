use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use firestore::*;
use serde::{Deserialize, Serialize};

const PROJECT_ID: &str = "neospeller";
const COLLECTION: &str = "spellcheck_logs";
const FALLBACK_FILE: &str = ".neospeller";

#[derive(Debug, Serialize, Deserialize)]
pub struct SpellcheckLog {
    pub original: String,
    pub corrected: String,
    pub created_at: DateTime<Utc>,
}

pub async fn fetch_all() -> Result<Vec<SpellcheckLog>, Box<dyn std::error::Error>> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let db = FirestoreDb::new(PROJECT_ID).await?;

    let docs: Vec<SpellcheckLog> = db
        .fluent()
        .select()
        .from(COLLECTION)
        .obj()
        .query()
        .await?;

    Ok(docs)
}

pub fn read_fallback_file() -> Vec<SpellcheckLog> {
    let Some(path) = fallback_path() else {
        return Vec::new();
    };

    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<SpellcheckLog>(l).ok())
        .collect()
}

async fn push(entry: &SpellcheckLog) -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let db = FirestoreDb::new(PROJECT_ID).await?;

    db.fluent()
        .insert()
        .into(COLLECTION)
        .generate_document_id()
        .object(entry)
        .execute::<SpellcheckLog>()
        .await?;

    Ok(())
}

fn fallback_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(FALLBACK_FILE))
}

fn append_to_file(entry: &SpellcheckLog) -> std::io::Result<()> {
    let path = fallback_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;
    let line = serde_json::to_string(entry)?;
    let mut file = OpenOptions::new().append(true).create(true).open(&path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

/// Append a correction to the local JSONL spool. Never touches the network;
/// a separate `flush` binary drains the spool into Firestore.
pub fn spool(original: String, corrected: String) {
    let entry = SpellcheckLog {
        original,
        corrected,
        created_at: Utc::now(),
    };

    if let Err(err) = append_to_file(&entry) {
        eprintln!("neospeller: failed to spool log: {}", err);
    }
}

/// Push every entry in `~/.neospeller` to Firestore. Successfully pushed
/// entries are removed; failures stay in the file for the next flush.
/// Returns `(succeeded, failed)`.
pub async fn flush_pending() -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let entries = read_fallback_file();
    if entries.is_empty() {
        return Ok((0, 0));
    }

    let mut pending: Vec<SpellcheckLog> = Vec::new();
    let mut succeeded = 0usize;

    for entry in entries {
        match push(&entry).await {
            Ok(()) => succeeded += 1,
            Err(err) => {
                eprintln!("flush: push failed, keeping entry: {}", err);
                pending.push(entry);
            }
        }
    }

    let failed = pending.len();
    rewrite_fallback(&pending)?;
    Ok((succeeded, failed))
}

fn rewrite_fallback(entries: &[SpellcheckLog]) -> std::io::Result<()> {
    let path = fallback_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;

    if entries.is_empty() {
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    } else {
        let tmp = path.with_extension("tmp");
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&tmp)?;
            for entry in entries {
                let line = serde_json::to_string(entry)?;
                writeln!(file, "{}", line)?;
            }
            file.sync_all()?;
        }
        fs::rename(&tmp, &path)
    }
}
