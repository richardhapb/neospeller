use chrono::{DateTime, Utc};
use firestore::*;
use serde::{Deserialize, Serialize};

const PROJECT_ID: &str = "neospeller";
const COLLECTION: &str = "spellcheck_logs";

#[derive(Debug, Serialize, Deserialize)]
struct SpellcheckLog {
    original: String,
    corrected: String,
    created_at: DateTime<Utc>,
}

async fn append(original: String, corrected: String) -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let db = FirestoreDb::new(PROJECT_ID).await?;

    let log = SpellcheckLog {
        original,
        corrected,
        created_at: Utc::now(),
    };

    db.fluent()
        .insert()
        .into(COLLECTION)
        .generate_document_id()
        .object(&log)
        .execute::<SpellcheckLog>()
        .await?;

    Ok(())
}

pub fn log(original: String, corrected: String) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("firestore: failed to start runtime: {}", err);
            return;
        }
    };

    if let Err(err) = runtime.block_on(append(original, corrected)) {
        eprintln!("firestore: failed to append log: {}", err);
    }
}
