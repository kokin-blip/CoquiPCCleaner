fn main() -> anyhow::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("--fixture-report") => coqui_engine::helper_validation::report(),
        Some("--validate-fixtures") => coqui_engine::helper_validation::validate(),
        Some("--serve")=>coqui_engine::helper::serve(),
        Some("--user-trash-directory")=>coqui_engine::privileged::trash_directory_worker(),
        Some("--health") if coqui_engine::helper::protocol()==2 => {println!("{}",coqui_engine::helper::production(coqui_engine::privileged::Operation::Health,false)?);Ok(())},
        Some("--health")=>{println!("{}",coqui_engine::helper::request(&serde_json::json!({"protocol":1,"operation":"health"}))?);Ok(())},
        _=>anyhow::bail!("Use --serve for the installed daemon or --health for the authenticated installation gate"),
    }
}
