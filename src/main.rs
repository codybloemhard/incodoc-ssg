use std::fs::File;
use std::fs::remove_file;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;
use std::collections::HashSet;
use std::process::ExitCode;
use std::path::Path;
use std::path::PathBuf;
use std::fmt::Display;

use simpleio::{ read_lines, read_file_into_string, file_exists };

use incodoc::PropVal;
use incodoc::Doc;
use incodoc::output::doc_out;
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

use chrono::{ Local, Datelike };

use rss::ChannelBuilder;
use rss::Channel;
use rss::ItemBuilder;
use rss::Item;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    path: String,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(short_flag = 'i', about = "Initialise a config file.")]
    Init {
        #[clap(long, help = "Set source directory.")]
        src: Option<String>,
        #[clap(long, help = "Set destination directory.")]
        dst: Option<String>,
        #[clap(long, help = "Set CSS path (within destination directory!).")]
        css: Option<String>,
        #[clap(long, help = "Set website url (for RSS).")]
        link: Option<String>,
    },
    #[clap(about = "Initialise an RSS feed file.")]
    Rss {
        #[clap(long, help = "Allow the overwriting of an existing RSS feed file.")]
        force: bool,
    },
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

    let (config, entries, init, write) = match args.command {
        Commands::Init { src, dst, css, link } => {
            let mut config = Config::default();
            if let Some(src) = src {
                config.src = src;
            }
            if let Some(dst) = dst {
                config.dst = dst;
            }
            if let Some(css) = css {
                config.css = normalise_path(css, &config);
            }
            if let Some(link) = link {
                config.link = link;
            }
            (config, Entries::default(), true, true)
        },
        x => {
            let lines = read_lines(&args.path);
            let (config, mut entries) = if let Some(res) = parse(lines) { res }
            else { return ExitCode::FAILURE };
            let mut write = true;

            match x {
                Commands::List => {
                    print!("{}", entries.list());
                    write = false;
                },
                Commands::Add { path } => {
                    let path = normalise_path(path, &config);
                    if entries.add(path.clone()) {
                        println!("New entry added successfully.");
                        update_entry(path, "keep", &mut entries, &config);
                    } else {
                        eprintln!("Could not add entry: entry already exists!");
                    }
                },
                Commands::Remove { path } => {
                    let path = normalise_path(path, &config);
                    if let Some(entry) = entries.remove(&path) {
                        println!("Removed this entry: ");
                        println!("{}", entry.pretty_print());
                        delete_files(&entry.path, &config.dst);
                    }
                },
                Commands::Enable { path } => {
                    let path = normalise_path(path, &config);
                    if entries.set_enabled(&path, true) {
                        println!("Enabled successfully.");
                        update_entry(path, "keep", &mut entries, &config);
                    } else {
                        println!("Could not find entry.");
                    }
                },
                Commands::Disable { path } => {
                    let path = normalise_path(path, &config);
                    if entries.set_enabled(&path, false) {
                        println!("Disabled successfully.");
                        delete_files(&path, &config.dst);
                    } else {
                        println!("Could not find entry.");
                    }
                },
                Commands::Update { path, version_bump } => {
                    let path = normalise_path(path, &config);
                    update_entry(path, &version_bump, &mut entries, &config);
                },
                Commands::Rss { force } => {
                    let date = Local::now();
                    let date_2822 = date.to_rfc2822();
                    let channel = ChannelBuilder::default()
                        .title(&config.title)
                        .link(&config.link)
                        .description(&config.description)
                        .language(Some(config.lang.clone()))
                        .copyright(Some(config.author.clone()))
                        .pub_date(Some(date_2822))
                        .build().to_string();
                    let mut feed_path = PathBuf::from(&config.dst);
                    let mut incodoc_feed_path = PathBuf::from(&config.dst);
                    feed_path.push("feed.xml");
                    incodoc_feed_path.push("incodoc-feed.xml");
                    if (!file_exists(&feed_path) && !file_exists(&incodoc_feed_path)) | force {
                        if !write_file(&feed_path, feed_path.display(), channel.clone()) {
                            return ExitCode::FAILURE;
                        }
                        if !write_file(&incodoc_feed_path, incodoc_feed_path.display(), channel) {
                            return ExitCode::FAILURE;
                        }
                    } else {
                        eprintln!("Could not overwrite RSS files without force.");
                        return ExitCode::FAILURE;
                    }
                    write = false;
                },
                Commands::Init { .. } => { },
            }

            (config, entries, false, write)
        },
    };

    if write {
        let mut output = config.unparse();
        entries.unparse(&mut output);
        if !write_file(&args.path, &args.path, output) {
            return ExitCode::FAILURE;
        }
    }

    if init {
        println!("Make sure to finish by editing the just generated config file!");
        println!("After that you can run the rss command to generate a feed.");
    }

    ExitCode::SUCCESS
}

// returns true if ok, false if need to return with FAILURE
fn update_entry(
    path: String, version_bump: &str, entries: &mut Entries, config: &Config
) -> bool {
    if let Some(index) = entries.get_index(&path) {
        let entry = &mut entries.entries[index];
        let mut src_path = PathBuf::from(&config.src);
        let mut dst_path = PathBuf::from(&config.dst);
        src_path.push(&entry.path);
        dst_path.push(&entry.path);
        dst_path.set_extension("html");
        let mut inc_path = dst_path.clone();
        inc_path.set_extension("incodoc");
        let base_path = if let Some(file_parent) = dst_path.parent()
            && let Some(bp) = pathdiff::diff_paths(&config.dst, file_parent) {
            bp
        } else {
            eprintln!("Could not compute base path!");
            return false;
        };
        let mut css_path = PathBuf::from(&base_path);
        css_path.push(&config.css);
        let inc_rel_path = inc_path
            .file_name()
            .map(|os| os.to_str().unwrap_or(""))
            .unwrap_or("");
        let src = match read_file_into_string(&src_path) {
            Ok(src) => src,
            Err(err) => {
                eprintln!("Could not open file {}: {}", src_path.display(), err);
                return false;
            },
        };
        let mut feed_path = PathBuf::from(&config.dst);
        feed_path.push("feed.xml");
        let mut incodoc_feed_path = PathBuf::from(&config.dst);
        incodoc_feed_path.push("incodoc-feed.xml");
        let mut doc = parse_md_to_incodoc(&src);
        let bump_result = entry.bump_version(version_bump);
        let date = Local::now();
        let date_2822 = date.to_rfc2822();
        let date_unix = date.timestamp();
        let date_footer = date.format("%Y-%m-%d %a");
        let date_year = date.year();
        doc.props.insert("version".to_string(), PropVal::String(entry.print_version()));
        doc.props.insert("lang".to_string(), PropVal::String(config.lang.clone()));
        doc.props.insert("author".to_string(), PropVal::String(config.author.clone()));
        doc.props.insert("date-rfc2822".to_string(), PropVal::String(date_2822.clone()));
        doc.props.insert("date-unix".to_string(), PropVal::String(date_unix.to_string()));
        doc.props.insert(
            "initial-version-date".to_string(), PropVal::String(entry.first_date.clone())
        );
        let header = build_header(&doc, inc_rel_path);
        let footer = build_footer(
            entry, &config.author, &date_footer.to_string(), date_year, entry.first_year
        );
        let mut header_links = vec![
            HeaderLink::Css{ href: css_path.display().to_string() },
            HeaderLink::General {
                rel: "alternate".to_string(),
                ltype: "text/incodoc".to_string(),
                href: inc_rel_path.to_string(),
            },
        ];
        if file_exists(&feed_path) {
            header_links.push(HeaderLink::General {
                rel: "alternate".to_string(),
                ltype: "application/rss+xml".to_string(),
                href: feed_path.display().to_string(),
            });
        }
        if file_exists(&incodoc_feed_path) {
            header_links.push(HeaderLink::General {
                rel: "alternate".to_string(),
                ltype: "incodoc/rss+xml".to_string(),
                href: incodoc_feed_path.display().to_string(),
            });
        }
        let conf = incodoc_to_html::config::Config {
            include: Include::Augmented(header, footer),
            header_links,
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
        if let Some(dir) = dst_path.parent()
            && let Err(error) = std::fs::create_dir_all(dir) {
            eprintln!("Could not create dir {}: {}.", dir.display(), error);
        };
        let html = doc_to_html_string(&mut doc, &conf);
        if !write_file(&dst_path, dst_path.display(), html) {
            return false;
        }
        let mut incodoc = String::new();
        doc_out(&doc, &mut incodoc);
        if !write_file(&inc_path, inc_path.display(), incodoc) {
            return false;
        }
        match bump_result {
            Some(version) => println!("New version: {}", print_version(&version)),
            None => println!("Version was not bumped up!"),
        }
        if version_bump == "major" || version_bump == "minor" {
            let mut link = config.link.clone();
            link.push_str(&path);
            link = link.strip_suffix(".md").unwrap_or(&link).to_string();
            let mut incodoc_link = link.clone();
            link.push_str(".html");
            incodoc_link.push_str(".incodoc");
            let item = ItemBuilder::default()
                // .title(Some("".to_string()))
                .link(Some(link))
                .author(Some(config.author.clone()))
                .pub_date(Some(date_2822))
                .build();
            let mut incodoc_item = item.clone();
            incodoc_item.link = Some(incodoc_link);
            let mut feed_path = PathBuf::from(&config.dst);
            let mut incodoc_feed_path = PathBuf::from(&config.dst);
            feed_path.push("feed.xml");
            incodoc_feed_path.push("incodoc-feed.xml");
            let write_channel = |fp: PathBuf, item: Item| {
                if let Ok(feed_file) = File::open(&fp) {
                    if let Ok(mut channel) = Channel::read_from(BufReader::new(feed_file)) {
                        channel.items.push(item);
                        write_file(&fp, fp.display(), channel.to_string());
                    } else {
                        eprintln!("Could not read RSS channel from file: {}", fp.display());
                    }
                } else {
                    eprintln!("Could not open RSS feed file: {}", fp.display());
                }
            };
            write_channel(feed_path, item);
            write_channel(incodoc_feed_path, incodoc_item);
        }
    } else {
        eprintln!("Could not find entry.");
    }
    true
}

fn normalise_path(path: String, config: &Config) -> String {
    let path = if let Some(path) = path.strip_prefix(&config.src) {
        path.to_string()
    } else if let Some(path) = path.strip_prefix(&config.dst) {
        path.to_string()
    } else {
        path
    };
    if let Some(path) = path.strip_suffix(".html") {
        let mut path = path.to_string();
        path.push_str(".md");
        path
    } else if let Some(path) = path.strip_suffix(".incodoc") {
        let mut path = path.to_string();
        path.push_str(".md");
        path
    } else {
        path
    }
}

// returns false if failed
fn write_file<P: AsRef<Path>, D: Display>(file_path: P, display: D, contents: String) -> bool {
    let mut file = if let Ok(file) = File::create(&file_path) { file }
    else {
        eprintln!("Could not open file {} for writing", display);
        return false;
    };
    if file.write_all(&contents.into_bytes()).is_err() {
        eprintln!("Could not write to file {}!", display);
        return false;
    }
    println!("Output written successfully to {}.", display);
    true
}

fn delete_files(path: &str, base: &str) {
    let mut dst_path = PathBuf::from(base);
    dst_path.push(path);
    dst_path.set_extension("html");
    let mut inc_path = dst_path.clone();
    inc_path.set_extension("incodoc");
    delete_file(&dst_path.display().to_string());
    delete_file(&inc_path.display().to_string());
}

fn delete_file<P: AsRef<Path> + Display + Copy>(path: P) {
    match remove_file(path) {
        Ok(_) => println!("Deleted file: {}", path),
        Err(error) => eprintln!("Could not delete file {}: {}", path, error),
    }
}

fn parse(lines: Vec<String>) -> Option<(Config, Entries)> {
    let mut res = Entries::default();
    let mut iter = lines.into_iter().enumerate();
    let src = parse_kv(iter.next(), "src", true)?;
    let dst = parse_kv(iter.next(), "dst", true)?;
    let css = parse_kv(iter.next(), "css", false)?;
    let link = parse_kv(iter.next(), "link", true)?;
    let lang = parse_kv(iter.next(), "lang", false)?;
    let author = parse_kv(iter.next(), "author", false)?;
    let title = parse_kv(iter.next(), "title", false)?;
    let description = parse_kv(iter.next(), "description", false)?;
    for (i, line) in iter {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(entry) = Entry::from_line(line, i) {
            let path = entry.path.clone();
            res.entries.push(entry);
            let index = res.entries.len() - 1;
            res.index.insert(path, index);
        }
    }
    Some((
        Config {
            src,
            dst,
            css,
            link,
            lang,
            author,
            title,
            description,
        },
        res,
    ))
}

fn parse_kv(line: Option<(usize, String)>, key: &str, ensure_slash: bool) -> Option<String> {
    let (line_nr, line) = if let Some(l) = line { l }
    else {
        eprintln!("Expected line containing source.");
        return None;
    };
    if let Some((k, v)) = line.split_once(':') {
        let k = k.trim();
        if k != key {
            eprintln!("Expected key {key} but found key {k} on line number {line_nr}.");
            return None;
        }
        let mut value = v.trim().to_string();
        if ensure_slash && !value.ends_with('/') {
            value.push('/');
        }
        Some(value)
    } else {
        None
    }
}

struct Config {
    src: String,
    dst: String,
    css: String,
    link: String,
    lang: String,
    author: String,
    title: String,
    description: String,
}

impl Config {
    fn unparse(self) -> String {
        fn unparse_field(res: &mut String, name: &str, value: &str) {
            res.push_str(name);
            res.push_str(": ");
            res.push_str(value);
            res.push('\n');
        }
        let mut res = String::new();
        unparse_field(&mut res, "src", &self.src);
        unparse_field(&mut res, "dst", &self.dst);
        unparse_field(&mut res, "css", &self.css);
        unparse_field(&mut res, "link", &self.link);
        unparse_field(&mut res, "lang", &self.lang);
        unparse_field(&mut res, "author", &self.author);
        unparse_field(&mut res, "title", &self.title);
        unparse_field(&mut res, "description", &self.description);
        res.push('\n');
        res
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            src: "/some/dir".to_string(),
            dst: "/another/dir".to_string(),
            css: "/another/dir/style.css".to_string(),
            link: "https://website.com".to_string(),
            lang: "en".to_string(),
            author: "Firstname Lastname".to_string(),
            title: "Website of Firstname Lastname.".to_string(),
            description: "Very interesting stuff.".to_string(),
        }
    }
}

fn build_header(doc: &Doc, incodoc_url: &str) -> String {
    let mut header = String::new();
    header += "<header>";
    if !incodoc_url.is_empty() {
        header += "<a href=\"./";
        header += incodoc_url;
        header += "\"> incodoc version</a>";
    }
    if let Some(nav) = doc.navs.first() && let Some(link) = nav.links.first() {
        header += " | ";
        link_to_html(link, &mut header);
    }
    header += "</header>";
    header
}

fn build_footer(entry: &Entry, author: &str, date: &str, year: i32, first_year: i32) -> String {
    let mut footer = String::new();
    footer += "<footer>";
    footer += "<strong>© ";
    if year == first_year {
        footer += &year.to_string();
    } else {
        footer += &first_year.to_string();
        footer += " - ";
        footer += &year.to_string();
    }
    footer += " ";
    footer += author;
    footer += "</strong> | ";
    footer += "version: ";
    footer += "<strong>";
    footer += &entry.print_version();
    footer += "</strong>";
    footer += " | date: <strong>";
    footer += date;
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
    first_date: String,
    first_year: i32,
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
        res.push('\n');
        res.push_str("first added on: ");
        res.push_str(&self.first_date);
        res.push('\n');
        res
    }

    fn from_line(line: &str, line_nr: usize) -> Option<Self> {
        let mut split = line.split(',');
        let path = if let Some(path) = split.next() {
            let path = path.trim();
            path.to_string()
        } else {
            eprintln!("Could not get path on line {line_nr}!");
            return None;
        };
        let version = if let Some(version_raw) = split.next() {
            let mut split = version_raw.split('.');
            let major = if let Some(major_raw) = split.next() {
                let major_raw = major_raw.trim();
                if let Ok(major) = major_raw.parse::<usize>() { major }
                else {
                    eprintln!("Could not parse version major on line {line_nr}!");
                    return None;
                }
            } else {
                eprintln!("Could not get version major on line {line_nr}!");
                return None;
            };
            let minor = if let Some(minor_raw) = split.next() {
                if let Ok(minor) = minor_raw.parse::<usize>() { minor }
                else {
                    eprintln!("Could not parse version minor on line {line_nr}!");
                    return None;
                }
            } else {
                eprintln!("Could not get version minor on line {line_nr}!");
                return None;
            };
            let patch = if let Some(patch_raw) = split.next() {
                if let Ok(patch) = patch_raw.parse::<usize>() { patch }
                else {
                    eprintln!("Could not parse version patch on line {line_nr}!");
                    return None;
                }
            } else {
                eprintln!("Could not get version patch on line {line_nr}!");
                return None;
            };
            (major, minor, patch)
        } else {
            eprintln!("Could not get version on line {line_nr}!");
            return None;
        };
        let enabled = if let Some(enabled_raw) = split.next() {
            let enabled_raw = enabled_raw.trim();
            if enabled_raw == "enabled" {
                true
            } else if enabled_raw == "disabled" {
                false
            } else {
                eprintln!("Could not parse enabled on line {line_nr}!");
                return None;
            }
        } else {
            eprintln!("Could not get enabled on line {line_nr}!");
            return None;
        };
        let first_date = if let Some(first_date_raw) = split.next() {
            first_date_raw.trim().to_string()
        } else {
            eprintln!("Could not get first date on line {line_nr}!");
            return None;
        };
        let first_year = if let Ok(first_year) = first_date[0..4].parse::<i32>() {
            first_year
        } else {
            eprintln!("Could not parse first year on line {line_nr}!");
            return None;
        };
        Some(Entry {
            path,
            version,
            enabled,
            first_date,
            first_year,
        })
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
    fn list(&self) -> String {
        let mut res = String::new();
        for entry in &self.entries {
            res.push_str(&entry.pretty_print());
            res.push_str("\n\n");
        }
        res.pop();
        res
    }

    fn unparse(&self, res: &mut String) {
        for Entry { path, version, enabled, first_date, .. } in &self.entries {
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
            res.push_str(", ");
            res.push_str(first_date);
            res.push('\n');
        }
    }

    // returns true if added, false if already exists
    fn add(&mut self, path: String) -> bool {
        if self.index.contains_key(&path) {
             false
        } else {
            let date = Local::now();
            let date_first = date.format("%Y-%m-%d %a");
            let date_year = date.year();
            self.entries.push(Entry {
                path: path.clone(),
                version: (0, 1, 0),
                enabled: true,
                first_date: date_first.to_string(),
                first_year: date_year,
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
            let first_date = std::mem::take(&mut self.entries[index].first_date);
            Some(Entry {
                path,
                version: self.entries[index].version,
                enabled: self.entries[index].enabled,
                first_date,
                first_year: self.entries[index].first_year,
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

