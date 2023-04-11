use clap::Parser;
use protobuf::{EnumOrUnknown, MessageField};
use scip::{
    self,
    types::{Document, Index, Metadata, Occurrence, SymbolInformation, ToolInfo},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug)]
struct Args {
    /// The project root that'll be used to generate the SCIP index
    #[arg(name = "project-root", long, short = 'r')]
    project_root: PathBuf,

    #[arg(long, short = 'i')]
    input: PathBuf,
    #[arg(long, short = 'o')]
    output: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct CTagsEntry {
    /// TODO(SuperAuguste): Handle _type != tag
    name: String,
    path: PathBuf,
    pattern: String,
    language: String,
    line: i32,
    /// TODO(SuperAuguste): Use enum
    kind: String,
    /// TODO(SuperAuguste): Use enum
    scope: Option<String>,
    #[serde(rename = "scopeKind")]
    scope_kind: Option<String>,
    roles: String,
}

fn contains_bad_chars(string: &str) -> bool {
    for c in string.chars() {
        match c {
            '/' | '#' | '.' | ':' | '!' | '(' | ')' | '[' | ']' => return true,
            _ => {}
        }
    }
    false
}

fn emit_name_maybe_escape(string: &mut String, name: &str) {
    let cbc = contains_bad_chars(name);
    if cbc {
        string.push('`');
    }
    string.push_str(name);
    if cbc {
        string.push('`');
    }
}

fn main() {
    let args = Args::parse();
    let input = match fs::read_to_string(Path::new(&args.input)) {
        Ok(data) => data,
        Err(_) => {
            println!(
                "Error opening and reading file '{}'",
                args.input.to_str().unwrap()
            );
            exit(1);
        }
    };

    let metadata = Metadata {
        version: EnumOrUnknown::from(scip::types::ProtocolVersion::UnspecifiedProtocolVersion),
        tool_info: MessageField::some(ToolInfo {
            name: String::from("ctags-to-scip"),
            version: String::from("0.0.1"),
            arguments: std::env::args().collect(),
            ..Default::default()
        }),
        project_root: args.project_root.into_os_string().into_string().unwrap(),
        text_document_encoding: EnumOrUnknown::from(scip::types::TextEncoding::UTF8),
        ..Default::default()
    };

    let mut documents: HashMap<String, scip::types::Document> = HashMap::new();

    for line in input.lines() {
        let entry: CTagsEntry = serde_json::from_str(line).expect("Invalid ctags file!");
        let relative_path = entry.path.into_os_string().into_string().unwrap();

        let document: &mut Document = match documents.get_mut(&relative_path.clone()) {
            Some(value) => value,
            None => {
                documents.insert(
                    relative_path.clone(),
                    Document {
                        // TODO(SuperAuguste): Map this to SCIP language names
                        language: entry.language,
                        relative_path: relative_path.clone(),
                        ..Default::default()
                    },
                );
                documents.get_mut(&relative_path.clone()).unwrap()
            }
        };

        if entry.roles != "def" {
            panic!("Why is this not a definition? {}", line);
        }

        // TODO(SuperAuguste): Use scope information - everything is advertised as top-level global atm
        let mut symbol = String::from("file no_package_manager no_package no_version ");
        for segment in relative_path.split("/") {
            emit_name_maybe_escape(&mut symbol, segment);
            symbol.push('/');
        }
        emit_name_maybe_escape(&mut symbol, &entry.name);
        // TODO(SuperAuguste): Add more cases
        match entry.kind.as_str() {
            "function" => {
                symbol.push('(');
                symbol.push_str(entry.line.to_string().as_str());
                symbol.push(')');
                symbol.push('.');
            }
            &_ => symbol.push('.'),
        }

        println!("{}", symbol);

        document.symbols.push(SymbolInformation {
            symbol: symbol.clone(),
            ..Default::default()
        });

        // TODO(SuperAuguste): Run regex on line to obtain character position
        // TODO(SuperAuguste): match syntax kind
        document.occurrences.push(Occurrence {
            range: vec![entry.line - 1, 0, 0],
            symbol: symbol.clone(),
            symbol_roles: 1,
            syntax_kind: EnumOrUnknown::from(scip::types::SyntaxKind::UnspecifiedSyntaxKind),
            ..Default::default()
        })
    }

    match scip::write_message_to_file(
        Path::new(Path::new(&args.output)),
        Index {
            metadata: MessageField::some(metadata),
            documents: documents.into_values().collect(),
            ..Default::default()
        },
    ) {
        Ok(()) => {}
        Err(error) => panic!("Problem emitting index: {:?}", error),
    };
}
