use std::vec;
use std::fmt::Write;

use clap::Parser;
use file_system::*;
use pmap_analyzer::PMapCategory;

use crate::pmap::*;

mod pmap;
mod pmap_analyzer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the file containing the output of the `pmap -XX -p <PID>` command
    #[clap(short, long)]
    pmap_output: String,

    /// Path to the folder containing the application (executables and libraries)
    #[clap(short, long, default_value = "/app")]
    application_folder: Option<String>,

    /// Default thread stack size run `ulimit -s` to read for your distribution, can be modified by `COMPlus_DefaultStackSize` env variable for dotnet
    #[clap(short, long, default_value = "8192")]
    thread_stack_size: Option<u64>,

    /// Path to csv file, that contains start and end addresses of coalesces memory pages, that should be broken down
    #[clap(short, long)]
    csv_of_memory_regions: Option<String>,
}

fn main() {
    let args = Args::parse();
    let pmap_output = FileInfo::new(args.pmap_output);
    let memory_pages = get_memory_pages(pmap_output);
    let categories = get_categories_from_memory_pages(memory_pages.clone(), args.application_folder);
    println!("Overview of Categories:");
    println!("{}\n", categories);
    println!("Overview of Memory Pages which are bigger than 10 MiB:");
    println!("{}\n", memory_pages);
    let potential_threads: usize = memory_pages.0.iter().filter(|page| 
        page.size_in_kibibyte == args.thread_stack_size.unwrap()
        && page.mapping_kind == MappingKind::AnonymousPrivate(None)
        && page.permissions.contains(Permissions::Read)
        && page.permissions.contains(Permissions::Write)
        && page.permissions.contains(Permissions::Private)
        && page.virtual_memory_flags.contains(VirtualMemoryFlags::MayRead)
        && page.virtual_memory_flags.contains(VirtualMemoryFlags::MayWrite)
        && page.virtual_memory_flags.contains(VirtualMemoryFlags::MayExecute))
        .count();
    println!("{:~<258}", "");
    println!("Potential Number of Threads Stacks: {} (Total: {} KiB)", potential_threads, potential_threads * 8192);

    if let Some(file_with_memory_regions) = args.csv_of_memory_regions {

        let memory_regions = FileInfo::new(file_with_memory_regions);
        if !memory_regions.is_exist() {
            eprintln!("File with memory regions does not exist");
            return;
        }

        println!("{:~<258}", "");

        let mut memory_pages_in_regions = vec![];

        memory_regions.read_to_string().lines().for_each(
            |line| {
                let line = line.trim();
                if line.is_empty() {
                    return; // skip empty lines
                }
                let memory_region = line.split(',').map(|s| parse_hex(s.trim().to_string())).collect::<Vec<u64>>();
                if memory_region.len() != 2 {
                    eprintln!("Invalid line: {}", line);
                    return;
                }

                let start = memory_region[0];
                let end = memory_region[1];

                println!("Memory Pages in the range: 0x{:x} - 0x{:x}", start, end);

                memory_pages.0.iter().filter(|page| page.address >= start && ( page.address + page.size_in_kibibyte * 1024) <= end).for_each(|page| {
                    if !page.virtual_memory_flags.contains(VirtualMemoryFlags::DoNotIncludeInCoreDump) {
                        print!("{}", page);
                        memory_pages_in_regions.push(page.address);
                    }
                });

            },
        );

        if memory_pages_in_regions.is_empty() {
            println!("No memory pages found in the given memory regions");
        } else {
            let mut res = String::new(); 
            write!(&mut res, "{}", '{').unwrap();
            for page_addr in memory_pages_in_regions.iter() {
                write!(&mut res, " 0x{:x},", page_addr).unwrap();
            }
            write!(&mut res, "{}", " }").unwrap();

            println!("{}", res);
        }
    }

}

fn parse_hex(hex_str: String) -> u64 {
    u64::from_str_radix(hex_str.replace("`", "").as_str(), 16).unwrap_or(0)
}

fn get_memory_pages(input: FileInfo) -> pmap::PMapVec {
    let memory_pages = pmap::PMap::parse_pmap_output(input).expect("Could not parse pmap output");
    memory_pages
}

fn get_categories_from_memory_pages(memory_pages: pmap::PMapVec, application_folder: Option<String>) -> pmap_analyzer::PMapCategoryVec {

    let category_lookup = | mapping: MappingKind | -> String {

        let file_lookup = |full_name: &str | -> String {
            if full_name.starts_with("/usr/share/dotnet") {
                ".NET Libraries".to_string()
            } else if full_name.contains("memfd:doublemapper (deleted)") {
                "JIT Code".to_string()
            } else if let Some(app_folder) = &application_folder {
                if full_name.starts_with(&app_folder.as_str()) {
                    "Application".to_string()
                } else {
                    full_name.to_string()
                }
            } else {
                full_name.to_string()
            }
        };

        match mapping {
            MappingKind::File(file_info) => {
                if ! file_info.full_name().is_empty() {
                    file_lookup(&file_info.full_name())
                } else {
                    "".to_string()
                }
            },
            MappingKind::AnonymousPrivate(file_info) => {
                if let Some(full_name) = file_info {
                    file_lookup(&full_name)
                } else {
                    "Anonymous".to_string()
                }
            },
            MappingKind::AnonymousShared(file_info) => {
                 if let Some(full_name) = file_info {
                        file_lookup(&full_name)
                } else {
                    "Anonymous".to_string()
                }
            },
            _ => "".to_string()
        }
    };
    let categories = PMapCategory::get_categories_from_memory_pages(memory_pages, &category_lookup).expect("Couldn't generate categories from memory pages");
    categories
}

#[cfg(test)]
mod tests {
    use enumflags2::make_bitflags;

    use super::*;
    use crate::pmap::*;
    use std::path::*;

    #[test]
    fn test_pmap_output() {
        let pmap_output = FileInfo::new(std::env::current_dir().unwrap().join("demo_data/pmap_demo").display().to_string());

        let memory_pages = get_memory_pages(pmap_output);
        assert_eq!(memory_pages.0.len(), 4150);

        let some_page = memory_pages.0.get(36).unwrap();
        assert_eq!(some_page.address, 0x7f6e842c5000);
        assert_eq!(some_page.permissions, make_bitflags!(Permissions::{Read | Execute | Private}));
        assert_eq!(some_page.offset, 0x000c5000);
        assert_eq!(some_page.device_major, 08);
        assert_eq!(some_page.device_minor, 01);
        assert_eq!(some_page.inode, 784663);
        assert_eq!(some_page.size_in_kibibyte, 2528);
        assert_eq!(some_page.virtual_memory_flags, make_bitflags!(VirtualMemoryFlags::{Readable | Executable | MayRead | MayWrite | MayExecute | SoftDirty}));
        assert_eq!(some_page.mapping_kind, MappingKind::File(FileInfo::new("libcrypto.so.3".to_string())));
    }

    #[test]
    fn test_pmap_category_mapping_heap() { 
        let memory_pages = vec![
            PMap {
                mapping_kind: MappingKind::Heap,
                size_in_kibibyte: 10,
                ..Default::default()
            },
            PMap {
                mapping_kind: MappingKind::Stack,
                size_in_kibibyte: 20,
                ..Default::default()
            },
            PMap {
                mapping_kind: MappingKind::VirtualSystemCall,
                size_in_kibibyte: 30,
                ..Default::default()
            },
            PMap {
                mapping_kind: MappingKind::Heap,
                size_in_kibibyte: 40,
                ..Default::default()
            },
            PMap {
                mapping_kind: MappingKind::AnonymousPrivate(None),
                size_in_kibibyte: 10,
                ..Default::default()
            }
        ];

        let categories = get_categories_from_memory_pages(PMapVec(memory_pages), None);
        assert_eq!(categories.0.len(), 4);
        assert_eq!(categories.0[0].name, "[heap]");
        assert_eq!(categories.0[1].name, "[stack]");
        assert_eq!(categories.0[2].name, "[vsyscall]");
        assert_eq!(categories.0[0].total_size_in_kibibyte, 50);
        assert_eq!(categories.0[0].pages.len(), 2);
    }
}