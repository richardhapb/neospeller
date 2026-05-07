use neospeller::firestore_logger::flush_pending;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (succeeded, failed) = flush_pending().await?;
    eprintln!("flushed {}, {} failed", succeeded, failed);
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
