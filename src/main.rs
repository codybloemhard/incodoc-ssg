mod tests;

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

// TODO: read entries from lines, run map on call, write file
fn main() {
    let args = Args::parse();
    let lines = read_lines(args.path);
    match args.command {
        Commands::List => {
            println!("list");
        },
        Commands::Map { src, dst } => {
            println!("map: src {src} to dst: {dst}");
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

