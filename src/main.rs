use clap::Parser;
use protobuf::{EnumOrUnknown, MessageField};
use scip::{
    self,
    types::{Document, Index, Metadata, Occurrence, SymbolInformation, ToolInfo},
};
use serde::{Deserialize, Serialize, __private::doc};
use serde_json::Result;
use std::{
    collections::HashMap,
    env::args,
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
    /// TODO: Handle _type != tag
    name: String,
    path: PathBuf,
    pattern: String,
    language: String,
    line: i32,
    /// TODO: Use enum
    kind: String,
    /// TODO: Use enum
    scope: Option<String>,
    #[serde(rename = "scopeKind")]
    scope_kind: Option<String>,
    roles: String,
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
                        // TODO: Map this to SCIP language names
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

        let symbol = String::from("bruh");

        println!("{}", symbol);

        document.symbols.push(SymbolInformation {
            symbol: symbol.clone(),
            ..Default::default()
        });

        // TODO: Run regex on line to obtain character position
        // TODO: match syntax kind
        document.occurrences.push(Occurrence {
            range: vec![entry.line, 0, 0],
            symbol: symbol.clone(),
            symbol_roles: 0,
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
