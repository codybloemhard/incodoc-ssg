mod tests;

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::process::ExitCode;

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
    #[clap(short_flag = 'a', about = "Map a source to a destination.")]
    Map {
        src: String,
        dst: String,
        #[clap(short = 'f', default_value_t = false, help = "Force: may overwrite.")]
        force: bool,
        #[clap(short = 'c', default_value_t = false, help = "Carry over version and enabled when overwriting.")]
        carry: bool,
    },
    #[clap(short_flag = 'r', about = "Remove entry by either source or destination.")]
    Remove {
        src_or_dst: String,
    },
    #[clap(short_flag = 'e', about = "Enable a disabled entry by either source or destination.")]
    Enable {
        src_or_dst: String,
    },
    #[clap(short_flag = 'd', about = "Disable an enabled entry by either source or destination.")]
    Disable {
        src_or_dst: String,
    },
    #[clap(short_flag = 'u', about = "Update an entry by either source or destination.")]
    Update {
        src_or_dst: String,
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
        Commands::Map { src, dst, force, carry } => {
            match entries.map(src, dst, force, carry) {
                MapResult::Noop => println!("Nothing need to be done."),
                MapResult::NewDst => println!("New destination set."),
                MapResult::NewSrc => println!("New source set."),
                MapResult::NewEntry => println!("New source and destination set."),
                MapResult::NewDstBlocked => println!("Could not set new destination (use force)."),
                MapResult::NewSrcBlocked => println!("Could not set new source (use force)."),
            }
        },
        Commands::Remove { src_or_dst } => {
            if let Some(entry) = entries.remove(&src_or_dst) {
                println!("Removed this entry: ");
                println!("{}", entry.pretty_print());
            }
        },
        Commands::Enable { src_or_dst } => {
            if entries.set_enabled(&src_or_dst, true) {
                println!("Enabled successfully.");
            } else {
                println!("Could not find that source or destination.");
            }
        },
        Commands::Disable { src_or_dst } => {
            if entries.set_enabled(&src_or_dst, false) {
                println!("Disabled successfully.");
            } else {
                println!("Could not find that source or destination.");
            }
        },
        Commands::Update { src_or_dst, version_bump } => {
            if let Some(index) = entries.index_by_src_or_dst(&src_or_dst) {
                let entry = &mut entries.entries[index];
                let src = match read_file_into_string(&entry.src) {
                    Ok(src) => src,
                    Err(err) => {
                        eprintln!("Could not open file {}: {}", entry.src, err);
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
                let mut file = if let Ok(file) = File::create(&entry.dst) { file }
                else {
                    eprintln!("Could not open file {} for writing", entry.dst);
                    return ExitCode::FAILURE;
                };
                if file.write_all(&html.into_bytes()).is_err() {
                    eprintln!("Could not write to file {}!", entry.dst);
                    return ExitCode::FAILURE;
                }
                println!("Output written successfully to {}.", entry.dst);
                match entry.bump_version(&version_bump) {
                    Some(version) => println!("New version: {}", print_version(&version)),
                    None => println!("Version was not bumped up!"),
                }
            } else {
                eprintln!("Could not find source nor destination.");
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
    src: String,
    dst: String,
    version: Version,
    enabled: bool,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
struct Entries {
    entries: Vec<Entry>,
    src_index: HashMap<String, usize>,
    dst_index: HashMap<String, usize>,
}

#[derive(Clone, Copy, Hash, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum MapResult {
    Noop,
    NewDst,
    NewSrc,
    NewEntry,
    NewDstBlocked,
    NewSrcBlocked,
}

impl Entry {
    fn print_version(&self) -> String {
        print_version(&self.version)
    }

    fn pretty_print(&self) -> String {
        let mut res = String::new();
        res.push_str("     source: ");
        res.push_str(&self.src);
        res.push('\n');
        res.push_str("destination: ");
        res.push_str(&self.dst);
        res.push('\n');
        res.push_str("    version: ");
        res.push_str(&self.print_version());
        res.push('\n');
        res.push_str("    enabled: ");
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
            let src = if let Some(src) = split.next() {
                let src = src.trim();
                src.to_string()
            } else {
                eprintln!("Could not get src on line {i}!");
                continue;
            };
            let dst = if let Some(dst) = split.next() {
                let dst = dst.trim();
                dst.to_string()
            } else {
                eprintln!("Could not get dst on line {i}!");
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
                src: src.clone(),
                dst: dst.clone(),
                version,
                enabled,
            });
            let index = res.entries.len() - 1;
            res.src_index.insert(src, index);
            res.dst_index.insert(dst, index);
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
        for Entry { src, dst, version, enabled } in &self.entries {
            // actual deleting of entries happens here
            if src.is_empty() || dst.is_empty() {
                continue;
            }
            res.push_str(src);
            res.push_str(", ");
            res.push_str(dst);
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

    fn map(&mut self, src: String, dst: String, overwrite: bool, carry_over: bool)
        -> MapResult
    {
        if let Some(sindex) = self.src_index.get(&src) {
            if self.entries[*sindex].dst == dst {
                MapResult::Noop
            } else if overwrite {
                let old_dst = self.entries[*sindex].dst.clone();
                self.dst_index.insert(dst.clone(), *sindex); // point new dst to entry
                self.entries[*sindex].dst = dst; // set new dst in entry
                self.dst_index.remove(&old_dst); // make sure the old dst doesn't point to anything
                if !carry_over {
                    self.entries[*sindex].version = (0, 1, 0);
                    self.entries[*sindex].enabled = true;
                }
                MapResult::NewDst
            } else {
                MapResult::NewDstBlocked
            }
        } else if let Some(dindex) = self.dst_index.get(&dst) {
            if self.entries[*dindex].src == src {
                MapResult::Noop
            } else if overwrite {
                let old_src = self.entries[*dindex].src.clone();
                self.src_index.insert(src.clone(), *dindex); // point new dst to entry
                self.entries[*dindex].src = src; // set new src in entry
                self.src_index.remove(&old_src); // make sure the old src doesn't point to anything
                if !carry_over {
                    self.entries[*dindex].version = (0, 1, 0);
                    self.entries[*dindex].enabled = true;
                }
                MapResult::NewSrc
            } else {
                MapResult::NewSrcBlocked
            }
        } else {
            self.entries.push(Entry {
                src: src.clone(),
                dst: dst.clone(),
                version: (0, 1, 0),
                enabled: true,
            });
            let len = self.entries.len() - 1;
            self.src_index.insert(src, len);
            self.dst_index.insert(dst, len);
            MapResult::NewEntry
        }
    }

    fn index_by_src_or_dst(&self, src_or_dst: &str) -> Option<usize> {
        let src_res = self.src_index.get(src_or_dst);
        if src_res.is_some() { return src_res.copied(); }
        let dst_res = self.dst_index.get(src_or_dst);
        if dst_res.is_some() { return dst_res.copied(); }
        None
    }

    fn remove(&mut self, src_or_dst: &str) -> Option<Entry> {
        if let Some(index) = self.index_by_src_or_dst(src_or_dst) {
            self.src_index.remove(&self.entries[index].src);
            self.dst_index.remove(&self.entries[index].dst);
            let src = std::mem::take(&mut self.entries[index].src);
            let dst = std::mem::take(&mut self.entries[index].dst);
            Some(Entry {
                src,
                dst,
                version: self.entries[index].version,
                enabled: self.entries[index].enabled,
            })
        } else {
            None
        }
    }

    fn set_enabled(&mut self, src_or_dst: &str, enabled: bool) -> bool {
        if let Some(index) = self.index_by_src_or_dst(src_or_dst) {
            self.entries[index].enabled = enabled;
            true
        } else {
            false
        }
    }
}

