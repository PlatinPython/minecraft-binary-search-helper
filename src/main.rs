// extern crate core;

use std::{fs, process};
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use petgraph::dot::{Config, Dot};
use petgraph::graphmap::DiGraphMap;
use serde_derive::Deserialize;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct ModsToml {
    // modLoader: String,
    // loaderVersion: String,
    // license: String,
    // showAsResourcePack: Option<bool>,
    // properties: Option<Table>,
    // issueTrackerURL: Option<String>,
    mods: Vec<Mods>,
    dependencies: Option<HashMap<String, Vec<Dependency>>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Mods {
    modId: String,
    // namespace: Option<String>,
    // version: Option<String>,
    // displayName: Option<String>,
    // description: Option<String>,
    // logoFile: Option<String>,
    // logoBlur: Option<bool>,
    // updateJSONURL: Option<String>,
    // modproperties: Option<Table>,
    // credits: Option<String>,
    // authors: Option<String>,
    // displayURL: Option<String>,
    // displayTest: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Dependency {
    modId: String,
    mandatory: bool,
    // versionRange: Option<String>,
    // ordering: Option<String>,
    // side: Option<String>,
}

#[allow(unused)]
fn test_mods_toml() -> ModsToml {
    toml::from_str(
        r#"
        modLoader="javafml"
        # Forge for 1.19 is version 41
        loaderVersion="[41,)"
        license="All rights reserved"
        issueTrackerURL="github.com/MinecraftForge/MinecraftForge/issues"
        showAsResourcePack=false

        [[mods]]
            modId="examplemod"
            version="1.0.0.0"
            displayName="Example Mod"
            updateJSONURL="minecraftforge.net/versions.json"
            displayURL="minecraftforge.net"
            logoFile="logo.png"
            credits="I'd like to thank my mother and father."
            authors="Author"
            description='''
        Lets you craft dirt into diamonds. This is a traditional mod that has existed for eons. It is ancient. The holy Notch created it. Jeb rainbowfied it. Dinnerbone made it upside down. Etc.
            '''
            displayTest="MATCH_VERSION"

        [[dependencies.examplemod]]
            modId="forge"
            mandatory=true
            versionRange="[41,)"
            ordering="NONE"
            side="BOTH"

        [[dependencies.examplemod]]
            modId="minecraft"
            mandatory=true
            versionRange="[1.19,1.20)"
            ordering="NONE"
            side="BOTH"
    "#,
    ).unwrap()
}

fn jar_to_mods_toml<P: AsRef<Path>>(path: P) -> Result<ModsToml, Box<dyn Error>> {
    let mut archive = zip::ZipArchive::new(fs::File::open(path)?)?;
    let mut mods_toml = archive.by_name("META-INF/mods.toml")?;
    let mut contents = String::new();
    mods_toml.read_to_string(&mut contents)?;
    Ok(toml::from_str(&contents)?)
}

#[derive(Debug)]
struct Mod {
    file_name: PathBuf,
    mod_id: String,
    dependencies: Vec<(String, bool)>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let start = std::time::SystemTime::now();
    let mods_dir = Path::new("mods");
    if !mods_dir.exists() || !mods_dir.is_dir() {
        eprintln!("Dir doesn't exist");
        process::exit(1);
    }
    let mut mods = vec![];
    fs::read_dir(mods_dir)?.for_each(|s| {
        let s = s.unwrap();
        let file_name = s.path();
        let Ok(mods_toml) = jar_to_mods_toml(s.path()) else {
            eprintln!("{} has a broken mods.toml file", file_name.as_os_str().to_str().unwrap());
            mods.push(Mod {
                file_name,
                mod_id: String::new(),
                dependencies: Vec::new(),
            });
            return;
        };
        let map = mods_toml.dependencies.unwrap_or_default();
        mods_toml.mods.iter().for_each(|m| {
            let dependencies = map
                .get(&m.modId)
                .unwrap_or(&Vec::new())
                .iter()
                .map(|d| (d.modId.to_owned(), d.mandatory))
                .collect();
            mods.push(Mod {
                file_name: file_name.clone(),
                mod_id: m.modId.to_owned(),
                dependencies,
            });
        });
        // let mod_ids: Vec<String> = mods_toml.mods.iter().map(|s| s.modId.to_owned()).collect();
        // let dependencies: Vec<&Vec<Dependency>> = mod_ids
        //     .iter()
        //     .filter(|&s| map.contains_key(s))
        //     .map(|s| map.get(s).unwrap())
        //     .collect();
        // let dependencies: Vec<String> = dependencies
        //     .iter()
        //     .flat_map(|s| s.iter().filter(|s| s.mandatory).map(|s| s.modId.to_owned()))
        //     .collect();
        // mods.push(Mod {
        //     file_name,
        //     mod_id,
        //     dependencies,
        // });
    });
    // println!("{:#?}", mods);
    let mut graph = DiGraphMap::new();
    let mut i = 0;
    mods.iter().for_each(|m| {
        graph.add_node(&m.mod_id);
    });
    mods.iter().for_each(|m| {
        for (modId, mandatory) in &m.dependencies {
            if modId == "forge" || modId == "minecraft" {
                continue;
            }
            graph.add_edge(&m.mod_id, modId, *mandatory);
            i += 1;
        }
    });
    println!(
        "nodes: {}, edges: {}",
        graph.node_count(),
        graph.edge_count()
    );

    let nodes: Vec<&String> = graph.nodes().collect();
    for node in &nodes {
        graph.remove_edge(node, node);
    }

    for a in &nodes {
        for b in &nodes {
            if graph.contains_edge(a, b) {
                for c in &nodes {
                    if graph.contains_edge(b, c) {
                        graph.remove_edge(a, c);
                    }
                }
            }
        }
    }

    // let edges: Vec<(&String, &String, bool)> =
    //     graph.all_edges().map(|e| (e.0, e.1, *e.2)).collect();
    // for edge in edges {
    //     if !edge.2 {
    //         graph.remove_edge(edge.0, edge.1);
    //     }
    // }

    println!(
        "nodes: {}, edges: {}",
        graph.node_count(),
        graph.edge_count()
    );

    println!(
        "vec: {}, i: {}, graph: {}",
        mods.len(),
        i,
        graph.node_count()
    );
    // println!("{:#?}", graph);
    let m: Vec<_> = mods.iter().map(|m| &m.mod_id).collect();
    println!("{}", m.contains(&&"appliedenergistics2".to_owned()));
    println!(
        "{}",
        graph
            .nodes()
            .any(|s| s == &"appliedenergistics2".to_owned())
    );
    // m.push("minecraft");
    // m.push("forge");
    println!("{}", graph.nodes().all(|s| m.iter().any(|m| *m == s)));
    let mut f = fs::File::create("graph")?;
    write!(
        f,
        "{}",
        Dot::with_attr_getters(
            &graph,
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_, e| if !*e.2 {
                String::from("color = gray36")
            } else {
                String::new()
            },
            &|_, n| format!("label = \"{}\" ", n.0)
        )
    )?;
    // f.write_fmt(format!(
    //     "{}",
    //     Dot::with_attr_getters(
    //         &graph,
    //         &[Config::EdgeNoLabel, Config::NodeNoLabel],
    //         &|_, _| String::new(),
    //         &|_, n| format!("label = \"{}\" ", n.0)
    //     )
    // ))?;
    // println!(
    //     "{}",
    //     Dot::with_attr_getters(
    //         &graph,
    //         &[Config::EdgeNoLabel, Config::NodeNoLabel],
    //         &|_, _| String::new(),
    //         &|_, n| format!("label = \"{}\" ", n.0)
    //     )
    // );
    // println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    println!("Took {} ms", start.elapsed().unwrap().as_millis());
    Ok(())
}
