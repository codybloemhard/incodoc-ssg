use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::process::ExitCode;
use std::path::PathBuf;

use simpleio::{ read_lines, read_file_into_string };

use incodoc::PropVal;
use incodoc::Doc;
use incodoc::actions::toc::TableOfContentsItemType;
use incodoc::actions::toc::TableOfContentsFilterType;

use md_to_incodoc::parse_md_to_incodoc;

use incodoc_to_html::doc_to_html_string;
use incodoc_to_html::link_to_html;
use incodoc_to_html::config::*;

use clap::{
    Parser,
    Subcommand,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    path: String,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(short_flag = 'l', about = "List all entries.")]
    List,
    #[clap(short_flag = 'a', about = "Add entry.")]
    Add {
        path: String,
    },
    #[clap(short_flag = 'r', about = "Remove entry.")]
    Remove {
        path: String,
    },
    #[clap(short_flag = 'e', about = "Enable a disabled entry.")]
    Enable {
        path: String,
    },
    #[clap(short_flag = 'd', about = "Disable an enabled entry.")]
    Disable {
        path: String,
    },
    #[clap(short_flag = 'u', about = "Update an entry by either source or destination.")]
    Update {
        path: String,
        #[clap(help = "How to bump version: major, minor, patch, keep (version the same).")]
        version_bump: String,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();
    let lines = read_lines(&args.path);
    let mut entries = Entries::from_lines(lines);
    match args.command {
        Commands::List => {
            print!("{}", entries.list());
        },
        Commands::Add { path } => {
            if entries.add(path) {
                println!("New entry added successfully.");
            } else {
                eprintln!("Could not add entry: entry already exists!");
            }
        },
        Commands::Remove { path } => {
            if let Some(entry) = entries.remove(&path) {
                println!("Removed this entry: ");
                println!("{}", entry.pretty_print());
            }
        },
        Commands::Enable { path } => {
            if entries.set_enabled(&path, true) {
                println!("Enabled successfully.");
            } else {
                println!("Could not find entry.");
            }
        },
        Commands::Disable { path } => {
            if entries.set_enabled(&path, false) {
                println!("Disabled successfully.");
            } else {
                println!("Could not find entry.");
            }
        },
        Commands::Update { path, version_bump } => {
            if let Some(index) = entries.get_index(&path) {
                let entry = &mut entries.entries[index];
                let mut src_path = PathBuf::from("");
                let mut dst_path = PathBuf::from("testdir/");
                src_path.push(&entry.path);
                dst_path.push(&entry.path);
                dst_path.set_extension("html");
                let src = match read_file_into_string(&src_path) {
                    Ok(src) => src,
                    Err(err) => {
                        eprintln!("Could not open file {}: {}", src_path.display(), err);
                        return ExitCode::FAILURE;
                    },
                };
                let mut doc = parse_md_to_incodoc(&src);
                doc.props.insert("version".to_string(), PropVal::String(entry.print_version()));
                let header = build_header(&doc);
                let footer = build_footer(entry);
                let conf = Config {
                    include: Include::Augmented(header, footer),
                    nav: NavConfig {
                        include: false,
                        close_top: true,
                        closed_depth: 1000,
                        position: Position::Bottom,
                    },
                    table_of_contents: TableOfContentsConfig {
                        closed: false,
                        include: TableOfContentsInclusion::IfSuggested,
                        position: Position::BeforeFirstSubSection,
                        filter: Some((
                            HashSet::from([
                                TableOfContentsItemType::Document,
                                TableOfContentsItemType::Section,
                                TableOfContentsItemType::FootnoteDefinition,
                            ]),
                            TableOfContentsFilterType::IncludeWithChildren
                        )),
                    },
                };
                let html = doc_to_html_string(&mut doc, &conf);
                if let Some(dir) = dst_path.parent()
                    && let Err(error) = std::fs::create_dir_all(dir) {
                    eprintln!("Could not create dir {}: {}.", dir.display(), error);
                };
                let mut file = if let Ok(file) = File::create(&dst_path) { file }
                else {
                    eprintln!("Could not open file {} for writing", dst_path.display());
                    return ExitCode::FAILURE;
                };
                if file.write_all(&html.into_bytes()).is_err() {
                    eprintln!("Could not write to file {}!", dst_path.display());
                    return ExitCode::FAILURE;
                }
                println!("Output written successfully to {}.", dst_path.display());
                match entry.bump_version(&version_bump) {
                    Some(version) => println!("New version: {}", print_version(&version)),
                    None => println!("Version was not bumped up!"),
                }
            } else {
                eprintln!("Could not find entry.");
            }
        }
    }
    let mut file = if let Ok(file) = File::create(&args.path) { file }
    else {
        eprintln!("Could not open file {} for writing", args.path);
        return ExitCode::FAILURE;
    };
    if file.write_all(&entries.to_lines().into_bytes()).is_err() {
        eprintln!("Could not write to file {}!", args.path);
    };

    ExitCode::SUCCESS
}

fn build_header(doc: &Doc) -> String {
    let mut header = String::new();
    header += "<header>";
    if let Some(nav) = doc.navs.first() && let Some(link) = nav.links.first() {
        link_to_html(link, &mut header);
    }
    header += "</header>";
    header
}

fn build_footer(entry: &Entry) -> String {
    let mut footer = String::new();
    footer += "<footer>";
    footer += "<strong>© 2026 Cody Bloemhard</strong> | ";
    footer += "version: ";
    footer += "<strong>";
    footer += &entry.print_version();
    footer += "</strong>";
    footer += "</footer>";
    footer
}

type Version = (usize, usize, usize);

fn print_version(version: &Version) -> String {
    let mut res = String::new();
    res.push_str(&version.0.to_string());
    res.push('.');
    res.push_str(&version.1.to_string());
    res.push('.');
    res.push_str(&version.2.to_string());
    res
}

#[derive(Clone, Default, Hash, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Entry {
    path: String,
    version: Version,
    enabled: bool,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
struct Entries {
    entries: Vec<Entry>,
    index: HashMap<String, usize>,
}

impl Entry {
    fn print_version(&self) -> String {
        print_version(&self.version)
    }

    fn pretty_print(&self) -> String {
        let mut res = String::new();
        res.push_str("   path: ");
        res.push_str(&self.path);
        res.push('\n');
        res.push_str("version: ");
        res.push_str(&self.print_version());
        res.push('\n');
        res.push_str("enabled: ");
        if self.enabled {
            res.push_str("yes");
        } else {
            res.push_str("no");
        }
        res
    }

    fn bump_version(&mut self, bump: &str) -> Option<Version> {
        let new_version = match bump {
            "major" => Some((self.version.0 + 1, 0, 0)),
            "minor" => Some((self.version.0, self.version.1 + 1, 0)),
            "patch" => Some((self.version.0, self.version.1, self.version.2 + 1)),
            _ => None,
        };
        if let Some(new_version) = new_version {
            self.version = new_version;
        }
        new_version
    }
}

impl Entries {
    fn from_lines(lines: Vec<String>) -> Self {
        let mut res = Self::default();
        for (i, line) in lines.into_iter().enumerate() {
            let mut split = line.split(',');
            let path = if let Some(path) = split.next() {
                let path = path.trim();
                path.to_string()
            } else {
                eprintln!("Could not get path on line {i}!");
                continue;
            };
            let version = if let Some(version_raw) = split.next() {
                let mut split = version_raw.split('.');
                let major = if let Some(major_raw) = split.next() {
                    let major_raw = major_raw.trim();
                    if let Ok(major) = major_raw.parse::<usize>() { major }
                    else {
                        eprintln!("Could not parse version major on line {i}!");
                        continue;
                    }
                } else {
                    eprintln!("Could not get version major on line {i}!");
                    continue;
                };
                let minor = if let Some(minor_raw) = split.next() {
                    if let Ok(minor) = minor_raw.parse::<usize>() { minor }
                    else {
                        eprintln!("Could not parse version minor on line {i}!");
                        continue;
                    }
                } else {
                    eprintln!("Could not get version minor on line {i}!");
                    continue;
                };
                let patch = if let Some(patch_raw) = split.next() {
                    if let Ok(patch) = patch_raw.parse::<usize>() { patch }
                    else {
                        eprintln!("Could not parse version patch on line {i}!");
                        continue;
                    }
                } else {
                    eprintln!("Could not get version patch on line {i}!");
                    continue;
                };
                (major, minor, patch)
            } else {
                eprintln!("Could not get version on line {i}!");
                continue;
            };
            let enabled = if let Some(enabled_raw) = split.next() {
                let enabled_raw = enabled_raw.trim();
                if enabled_raw == "enabled" {
                    true
                } else if enabled_raw == "disabled" {
                    false
                } else {
                    eprintln!("Could not parse enabled on line {i}!");
                    continue;
                }
            } else {
                eprintln!("Could not get enabled on line {i}!");
                continue;
            };
            res.entries.push(Entry {
                path: path.clone(),
                version,
                enabled,
            });
            let index = res.entries.len() - 1;
            res.index.insert(path, index);
        }
        res
    }

    fn list(&self) -> String {
        let mut res = String::new();
        for entry in &self.entries {
            res.push_str(&entry.pretty_print());
            res.push_str("\n\n");
        }
        res.pop();
        res
    }

    fn to_lines(&self) -> String {
        let mut res = String::new();
        for Entry { path, version, enabled } in &self.entries {
            // actual deleting of entries happens here
            if path.is_empty() {
                continue;
            }
            res.push_str(path);
            res.push_str(", ");
            res.push_str(&version.0.to_string());
            res.push('.');
            res.push_str(&version.1.to_string());
            res.push('.');
            res.push_str(&version.2.to_string());
            res.push_str(", ");
            if *enabled {
                res.push_str("enabled");
            } else {
                res.push_str("disabled");
            }
            res.push('\n');
        }
        res
    }

    // returns true if added, false if already exists
    fn add(&mut self, path: String) -> bool {
        if self.index.contains_key(&path) {
             false
        } else {
            self.entries.push(Entry {
                path: path.clone(),
                version: (0, 1, 0),
                enabled: true,
            });
            let len = self.entries.len() - 1;
            self.index.insert(path, len);
            true
        }
    }

    fn get_index(&self, path: &str) -> Option<usize> {
        self.index.get(path).copied()
    }

    fn remove(&mut self, path: &str) -> Option<Entry> {
        if let Some(index) = self.get_index(path) {
            self.index.remove(path);
            let path = std::mem::take(&mut self.entries[index].path);
            Some(Entry {
                path,
                version: self.entries[index].version,
                enabled: self.entries[index].enabled,
            })
        } else {
            None
        }
    }

    fn set_enabled(&mut self, path: &str, enabled: bool) -> bool {
        if let Some(index) = self.get_index(path) {
            self.entries[index].enabled = enabled;
            true
        } else {
            false
        }
    }
}

