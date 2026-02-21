use clap::{
    Parser,
    Subcommand,
};

// src, dst, version, enabled

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args{
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

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::List => {
            println!("list");
        },
        Commands::Add { src, dst } => {
            println!("add src: {src}, dst: {dst}");
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
