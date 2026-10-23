fn main() -> anyhow::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("--serve")=>coqui_engine::helper::serve(),
        Some("--health")=>{println!("{}",coqui_engine::helper::request(&serde_json::json!({"protocol":1,"operation":"health"}))?);Ok(())},
        _=>anyhow::bail!("Use --serve for the installed daemon or --health for the authenticated installation gate"),
    }
}
