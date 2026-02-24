mod tests;

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

use simpleio::read_lines;

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
        #[clap(short = 'd', default_value_t = false, help = "Show all.")]
        dry: bool,
    },
}

fn main() {
    let args = Args::parse();
    let lines = read_lines(&args.path);
    let mut entries = Entries::from_lines(lines);
    match args.command {
        Commands::List => {
            print!("{}", entries.list());
        },
        Commands::Map { src, dst, force, carry } => {
            match entries.map(src, dst, force, carry) {
                MapResult::Noop => println!("  Nothing need to be done."),
                MapResult::NewDst => println!("  New destination set."),
                MapResult::NewSrc => println!("  New source set."),
                MapResult::NewEntry => println!("  New source and destination set."),
                MapResult::NewDstBlocked => println!("  Could not set new destination (use force)."),
                MapResult::NewSrcBlocked => println!("  Could not set new source (use force)."),
            }
        },
        Commands::Remove { src_or_dst } => {
            println!("remove: {src_or_dst}");
        },
        Commands::Enable { src_or_dst } => {
            println!("remove: {src_or_dst}");
        },
        Commands::Disable { src_or_dst } => {
            println!("remove: {src_or_dst}");
        },
        Commands::Update { src_or_dst, dry } => {
            println!("remove: {src_or_dst}, dry: {dry}");
        },
    }
    let mut file = if let Ok(file) = File::create(&args.path) { file }
    else {
        eprintln!("Could not open file {} for writing", args.path);
        return;
    };
    if file.write_all(&entries.to_lines().into_bytes()).is_err() {
        eprintln!("Could not write to file {}!", args.path);
    };
}

#[derive(Clone, Default, Hash, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Entry {
    src: String,
    dst: String,
    version: (usize, usize, usize),
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
                let fix = if let Some(fix_raw) = split.next() {
                    if let Ok(fix) = fix_raw.parse::<usize>() { fix }
                    else {
                        eprintln!("Could not parse version fix on line {i}!");
                        continue;
                    }
                } else {
                    eprintln!("Could not get version fix on line {i}!");
                    continue;
                };
                (major, minor, fix)
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
        for Entry { src, dst, version, enabled } in &self.entries {
            res.push_str("     source: ");
            res.push_str(src);
            res.push('\n');
            res.push_str("destination: ");
            res.push_str(dst);
            res.push('\n');
            res.push_str("    version: ");
            res.push_str(&version.0.to_string());
            res.push('.');
            res.push_str(&version.1.to_string());
            res.push('.');
            res.push_str(&version.2.to_string());
            res.push('\n');
            res.push_str("    enabled: ");
            if *enabled {
                res.push_str("yes");
            } else {
                res.push_str("no");
            }
            res.push_str("\n\n");
        }
        res.pop();
        res
    }

    fn to_lines(&self) -> String {
        let mut res = String::new();
        for Entry { src, dst, version, enabled } in &self.entries {
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
}

