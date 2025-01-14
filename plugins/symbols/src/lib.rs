use std::{collections::HashMap, fs};

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;

include!(concat!(env!("OUT_DIR"), "/unicode.rs"));

#[derive(Clone, Debug)]
struct Symbol {
    chr: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    symbols: HashMap<String, String>,
}

#[init]
fn init(config_dir: RString) -> Vec<Symbol> {
    // Try to load the config file, if it does not exist only use the static unicode characters
    if let Ok(content) = fs::read_to_string(format!("{}/symbols.ron", config_dir)) {
        match ron::from_str::<Config>(&content) {
            Ok(config) => {
                let symbols = UNICODE_CHARS
                    .iter()
                    .map(|(name, chr)| (name.to_string(), chr.to_string()))
                    .chain(config.symbols.into_iter())
                    .map(|(name, chr)| Symbol { chr, name })
                    .collect();
                return symbols;
            }
            Err(why) => {
                println!("Error parsing symbols config file: {}", why);
            }
        }
    }

    UNICODE_CHARS
        .iter()
        .map(|(name, chr)| Symbol {
            chr: chr.to_string(),
            name: name.to_string(),
        })
        .collect()
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Symbols".into(),
        icon: "accessories-character-map".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, symbols: &Vec<Symbol>) -> RVec<Match> {
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().ignore_case();
    let mut symbols = symbols
        .clone()
        .into_iter()
        .filter_map(|symbol| {
            matcher
                .fuzzy_match(&symbol.name, &input)
                .map(|score| (symbol, score))
        })
        .collect::<Vec<_>>();

    // Sort the symbol list according to the score
    symbols.sort_by(|a, b| b.1.cmp(&a.1));

    symbols.truncate(3);

    symbols
        .into_iter()
        .map(|(symbol, _)| Match {
            title: symbol.chr.into(),
            description: ROption::RSome(symbol.name.into()),
            use_pango: false,
            icon: ROption::RNone,
            id: ROption::RNone,
        })
        .collect()
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    HandleResult::Copy(selection.title.into_bytes())
}
