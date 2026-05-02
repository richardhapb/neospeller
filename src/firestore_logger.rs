use std::fs::OpenOptions;
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

pub fn log(original: String, corrected: String) {
    let entry = SpellcheckLog {
        original,
        corrected,
        created_at: Utc::now(),
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();

    let pushed = match runtime {
        Ok(rt) => rt.block_on(push(&entry)).is_ok(),
        Err(err) => {
            eprintln!("firestore: failed to start runtime: {}", err);
            false
        }
    };

    if !pushed {
        if let Err(err) = append_to_file(&entry) {
            eprintln!("firestore: failed to write fallback log: {}", err);
        }
    }
}
