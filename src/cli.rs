use clap::*;

#[derive(Parser)]
pub struct Cli {
    /// The resource kind to map the query path
    pub resource_name: String,
    /// The query path. JSON Path format
    pub json_path: String,
    /// Map over objects over all namespaces
    #[arg(short('A'), long, global = true, default_value = "false")]
    pub all_namespaces: bool,
    /// Only map over objects in certain namespace
    #[arg(short('n'), long, global = true, default_value = "default")]
    pub namespace: String,
}
