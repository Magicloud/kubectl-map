#![feature(try_blocks)]

mod cli;

use std::sync::Arc;

use clap::Parser;
use eyre::{Result, eyre};
use jsonpath_rust::query::js_path_vals;
use kube::{
    Api, Client, Discovery,
    api::{DynamicObject, ListParams},
    config::Kubeconfig,
    discovery::Scope,
};

// This usage seems break analyzer.
// #[macro_rules_attribute::apply(smol_macros::main!)]
fn main() -> Result<()> {
    smol::block_on(async_compat::Compat::new(real_main()))
}

async fn real_main() -> Result<()> {
    color_eyre::install()?;

    let cli = cli::Cli::parse();
    let kubeconfig = Kubeconfig::read()?;
    let client = Client::try_from(kubeconfig)?;
    let discovery = Discovery::new(client.clone()).run().await?;
    let list_p = ListParams::default();

    let (res, caps) = discovery
        .groups()
        .flat_map(|group| {
            group
                .resources_by_stability()
                .into_iter()
                .map(move |res| (group, res))
        })
        .filter(|(_, (res, _))| {
            cli.resource_name.eq_ignore_ascii_case(&res.kind)
                || cli.resource_name.eq_ignore_ascii_case(&res.plural)
        })
        .min_by_key(|(group, _)| group.name())
        .map(|(_, res)| res)
        .ok_or(eyre!("No such resource {} found", cli.resource_name))?;
    let api: Api<DynamicObject> = if caps.scope == Scope::Cluster || cli.all_namespaces {
        Api::all_with(client, &res)
    } else {
        Api::namespaced_with(client, &cli.namespace, &res)
    };
    let (ok, err): (Vec<_>, Vec<_>) = api
        .list(&list_p)
        .await?
        .items
        .into_iter()
        .map(|item| {
            let item = Arc::new(item);
            let x: Result<Vec<Result<String>>> = try {
                let json = serde_json::to_value(&*item).map_err(|e| eyre!("{e:?}"))?;
                let values = js_path_vals(&cli.json_path, &json).map_err(|e| eyre!("{e:?}"))?;
                values
                    .into_iter()
                    .map(|v| serde_json::to_string(v).map_err(|e| eyre!("{e:?}")))
                    .collect()
            };
            match x {
                Err(e) => vec![Err((item, e))],
                Ok(s) => s
                    .into_iter()
                    .map(|r| match r {
                        Ok(s) => Ok((item.clone(), s)),
                        Err(e) => Err((item.clone(), e)),
                    })
                    .collect(),
            }
        })
        .flatten()
        .partition(|x| x.is_ok());
    let ok = ok.into_iter().map(|x| x.unwrap());
    let err = err.into_iter().map(|x| x.unwrap_err());

    ok.map(|(item, value)| println!("{}\t{}", format_name(&item, cli.all_namespaces), value))
        .for_each(drop);
    err.map(|(item, err)| eprintln!("{}\t{}", format_name(&item, cli.all_namespaces), err))
        .for_each(drop);

    Ok(())
}

fn format_name(item: &DynamicObject, all_namespaces: bool) -> String {
    if all_namespaces {
        format!(
            "{}/{}",
            item.metadata
                .namespace
                .as_ref()
                .unwrap_or(&"default".to_owned()),
            item.metadata.name.as_ref().unwrap_or(&"default".to_owned())
        )
    } else {
        item.metadata.name.clone().unwrap_or_default()
    }
}
