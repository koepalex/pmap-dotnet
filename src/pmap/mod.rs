use enumflags2::{bitflags, BitFlags, BitFlag};
use file_system::*;
use std::fmt::Display;
use std::io::Error as ioError;
use std::{error::Error, str::FromStr};

// Sample output of pmap -XX -p PID
//       Adresse Zugr  Versatz Gerät   Inode      Size KernelPageSize MMUPageSize    Rss    Pss Pss_Dirty Shared_Clean Shared_Dirty Private_Clean Private_Dirty Referenced Anonymous LazyFree AnonHugePages ShmemPmdMapped y Shared_Hugetlb Private_Hugetlb Swap SwapPss Locked THPeligible                 VmFlags Zuordnung
// 7faf68872000 r-xs 02743000  00:01    4128         4              4           4      0      0         0            0            0             0             0          0         0        0             0              0             0              0               0    0       0      0           0 rd ex sh mr mw me ms sd memfd:doublemapper (deleted)
// which is a parser friendly output of the smaps structure, example of smap of debian bookworm:
// 7ffdcd768000-7ffdcd76a000 r-xp 00000000 00:00 0                          [vdso]
// Size:                  8 kB
// KernelPageSize:        4 kB
// MMUPageSize:           4 kB
// Rss:                   4 kB
// Pss:                   0 kB
// Pss_Dirty:             0 kB
// Shared_Clean:          4 kB
// Shared_Dirty:          0 kB
// Private_Clean:         0 kB
// Private_Dirty:         0 kB
// Referenced:            4 kB
// Anonymous:             0 kB
// LazyFree:              0 kB
// AnonHugePages:         0 kB
// ShmemPmdMapped:        0 kB
// FilePmdMapped:         0 kB
// Shared_Hugetlb:        0 kB
// Private_Hugetlb:       0 kB
// Swap:                  0 kB
// SwapPss:               0 kB
// Locked:                0 kB
// THPeligible:    0
// VmFlags: rd ex mr mw me de sd
// as documented under https://www.kernel.org/doc/html/latest/filesystems/proc.html

/// Structure of one line of `pmap -XX -p PID` output describing one memory page of the processor
#[derive(Debug, PartialEq, Clone)]
pub struct PMap {
    // Address - start address of the memory page in the process linier address space
    pub address: u64,
    // Perm - permissions of the memory page
    pub permissions: BitFlags<Permissions>,
    // Offset - offset in the file (in case of file backed mapping)
    pub offset: u64,
    // Device - device id where the file resides (in case of file backed mapping)
    pub device_major: u16,
    pub device_minor: u16,
    // Inode - filesystem inode number of the file (in case of file backed mapping)
    pub inode: u64,
    // Size - size of the mapping in KiB
    pub size_in_kibibyte: u64,
    // KernelPageSize - paging size of the kernel in KiB
    pub kernel_page_size_in_kibibyte: u8,
    // MMUPageSize - memory management unit page size in KiB
    pub mmu_page_size_in_kibibyte: u8,
    // RSS - size of the memory which is currently in RAM (not swapped out) in KiB
    pub resident_set_size_in_kibibyte: u64,
    // PSS - private size + shared size divided by number of mappings
    pub proportional_share_size_in_kibibyte: u64,
    // PSS dirty - size of PSS which was updated by another process
    pub proportional_share_size_dirty_in_kibibyte: u64,
    // Shared_Clean - size of memory that is shared with other processes and not modified in KiB (Note: memory that can be shared but isn't is counted as private)
    pub shared_clean_in_kibibyte: u64,
    // Shared_Dirty - size of memory that is shared with other processes and was modified in KiB
    pub shared_dirty_in_kibibyte: u64,
    // Private_Clean - size of memory that is private to the process and not modified in KiB
    pub private_clean_in_kibibyte: u64,
    // Private_Dirty - size of memory that is private to the process and was modified in KiB
    pub private_dirty_in_kibibyte: u64,
    // Referenced - This is the memory that is currently being accessed or referenced.
    pub referenced_in_kibibyte: u64,
    // Anonymous - size of memory that doesn't belong to a file (Note: even file based mappings may contain anonymous memory in case of copy-on-write)
    pub anonymous_in_kibibyte: u64,
    // LazyFree - indicates the pages flagged as MADV_FREE. These pages can be reclaimed though they may have unwritten changes in them. The MADV_FREE flag is removed from the pages if any changes are made to them after initial flagging. The pages remain unclaimed until the changes are written.
    pub lazy_free_in_kibibyte: u64,
    // AnonHugePages - size of memory pages used for anonymous mappings that is bigger than MMU page size (see: https://www.kernel.org/doc/html/latest/admin-guide/mm/transhuge.html)
    pub anonymous_huge_pages_in_kibibyte: u64,
    // ShmemPmdMapped - size of memory pages used for file mappings that is bigger than MMU page size (see: https://www.kernel.org/doc/html/latest/admin-guide/mm/transhuge.html)
    pub shared_memory_associated_with_huge_pages_in_kibibyte: u64,
    // FilePmdMapped - The “Pmd” in the term stands for Page Middle Directory. It is one of the kernel’s paging schemes, and this value indicates the number of file-backed pages that PMD entries are pointing to.
    pub file_pme_mapped_in_kibibyte: u64,
    // Shared_Hugetlb - size of transition lookaside buffer (TLB) for shared huge memory pages
    pub shared_hugetlb_in_kibibyte: u64,
    // Private_Hugetlb - size of transition lookaside buffer (TLB) for private huge memory pages
    pub private_hugetlb_in_kibibyte: u64,
    // Swap - size of memory that was swapped out in KiB (Note: file based read only memory like code does not need to be swapped out as it can be reloaded from the file)
    pub swap_in_kibibyte: u64,
    // SwapPSS - size of memory that was swapped out and is part of PSS in KiB
    pub swap_pss_in_kibibyte: u64,
    // Locked - size of memory that is locked in RAM and can't be swapped out in KiB
    pub locked_in_kibibyte: u64,
    // THPeligible - indicates if the memory page is eligible for transparent huge pages
    pub transparent_huge_page_eligible: bool,
    // VmFlags - flags of the memory page
    pub virtual_memory_flags: BitFlags<VirtualMemoryFlags>,
    // Mapping - type of mapping (heap, stack, file, anonymous, shared, etc.)
    pub mapping_kind: MappingKind,
}

impl PMap {
    pub fn parse_pmap_output(pmap_output: FileInfo) -> Result<PMapVec, Box<dyn Error>> {
        if !pmap_output.is_exist() {
            return Err(ioError::new(std::io::ErrorKind::NotFound, "File not found").into());
        }

        let mut pmaps = PMapVec(Vec::new());
        pmap_output.read_to_string().lines().skip(1).try_for_each(
            |line| -> Result<(), Box<dyn Error>> {
                let line = line.trim();
                if line.is_empty() {
                    return Ok(()); // skip empty lines
                }
                let pmap = PMap::from_str(line)?;
                pmaps.0.push(pmap);
                Ok(())
            },
        )?;

        Ok(pmaps)
    }
}

impl FromStr for PMap {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let mut parts = s.split_whitespace();

        let address = parts.next().ok_or("Can't parse address")?;
        let address = u64::from_str_radix(address, 16).map_err(|_| "Can't parse address")?;

        let permissions = parts.next().ok_or("Can't parse permissions")?;
        let permissions = BitFlags::<Permissions>::from_str(permissions)?;

        let offset = parts.next().ok_or("Can't parse offset")?;
        let offset = u64::from_str_radix(offset, 16).map_err(|_| "Can't parse offset")?;

        let device = parts.next().ok_or("Can't parse device")?;
        let mut device_parts = device.split(':');
        let device_major = device_parts.next().ok_or("Can't parse device major")?;
        let device_major =
            u16::from_str_radix(device_major, 16).map_err(|_| "Can't parse device major")?;
        let device_minor = device_parts.next().ok_or("Can't parse device minor")?;
        let device_minor =
            u16::from_str_radix(device_minor, 16).map_err(|_| "Can't parse device minor")?;

        let inode = parts.next().ok_or("Can't parse inode")?;
        let inode = u64::from_str_radix(inode, 10).map_err(|_| "Can't parse inode")?;

        let size_in_kibibyte = parts.next().ok_or("Can't parse size")?;
        let size_in_kibibyte =
            u64::from_str_radix(size_in_kibibyte, 10).map_err(|_| "Can't parse size")?;

        let kernel_page_size_in_kibibyte = parts.next().ok_or("Can't parse kernel page size")?;
        let kernel_page_size_in_kibibyte = u8::from_str_radix(kernel_page_size_in_kibibyte, 10)
            .map_err(|_| "Can't parse kernel page size")?;

        let mmu_page_size_in_kibibyte = parts.next().ok_or("Can't parse mmu page size")?;
        let mmu_page_size_in_kibibyte = u8::from_str_radix(mmu_page_size_in_kibibyte, 10)
            .map_err(|_| "Can't parse mmu page size")?;

        let resident_set_size_in_kibibyte = parts.next().ok_or("Can't parse resident set size")?;
        let resident_set_size_in_kibibyte = u64::from_str_radix(resident_set_size_in_kibibyte, 10)
            .map_err(|_| "Can't parse resident set size")?;

        let proportional_share_size_in_kibibyte =
            parts.next().ok_or("Can't parse proportional share size")?;
        let proportional_share_size_in_kibibyte =
            u64::from_str_radix(proportional_share_size_in_kibibyte, 10)
                .map_err(|_| "Can't parse proportional share size")?;

        let proportional_share_size_dirty_in_kibibyte = parts
            .next()
            .ok_or("Can't parse proportional share size dirty")?;
        let proportional_share_size_dirty_in_kibibyte =
            u64::from_str_radix(proportional_share_size_dirty_in_kibibyte, 10)
                .map_err(|_| "Can't parse proportional share size dirty")?;

        let shared_clean_in_kibibyte = parts.next().ok_or("Can't parse shared clean")?;
        let shared_clean_in_kibibyte = u64::from_str_radix(shared_clean_in_kibibyte, 10)
            .map_err(|_| "Can't parse shared clean")?;

        let shared_dirty_in_kibibyte = parts.next().ok_or("Can't parse shared dirty")?;
        let shared_dirty_in_kibibyte = u64::from_str_radix(shared_dirty_in_kibibyte, 10)
            .map_err(|_| "Can't parse shared dirty")?;

        let private_clean_in_kibibyte = parts.next().ok_or("Can't parse private clean")?;
        let private_clean_in_kibibyte = u64::from_str_radix(private_clean_in_kibibyte, 10)
            .map_err(|_| "Can't parse private clean")?;

        let private_dirty_in_kibibyte = parts.next().ok_or("Can't parse private dirty")?;
        let private_dirty_in_kibibyte = u64::from_str_radix(private_dirty_in_kibibyte, 10)
            .map_err(|_| "Can't parse private dirty")?;

        let referenced_in_kibibyte = parts.next().ok_or("Can't parse referenced")?;
        let referenced_in_kibibyte = u64::from_str_radix(referenced_in_kibibyte, 10)
            .map_err(|_| "Can't parse referenced")?;

        let anonymous_in_kibibyte = parts.next().ok_or("Can't parse anonymous")?;
        let anonymous_in_kibibyte =
            u64::from_str_radix(anonymous_in_kibibyte, 10).map_err(|_| "Can't parse anonymous")?;

        let lazy_free_in_kibibyte = parts.next().ok_or("Can't parse lazy free")?;
        let lazy_free_in_kibibyte =
            u64::from_str_radix(lazy_free_in_kibibyte, 10).map_err(|_| "Can't parse lazy free")?;

        let anonymous_huge_pages_in_kibibyte =
            parts.next().ok_or("Can't parse anonymous huge pages")?;
        let anonymous_huge_pages_in_kibibyte =
            u64::from_str_radix(anonymous_huge_pages_in_kibibyte, 10)
                .map_err(|_| "Can't parse anonymous huge pages")?;

        let shared_memory_associated_with_huge_pages_in_kibibyte = parts
            .next()
            .ok_or("Can't parse shared memory associated with huge pages")?;
        let shared_memory_associated_with_huge_pages_in_kibibyte =
            u64::from_str_radix(shared_memory_associated_with_huge_pages_in_kibibyte, 10)
                .map_err(|_| "Can't parse shared memory associated with huge pages")?;

        let file_pme_mapped_in_kibibyte = parts.next().ok_or("Can't parse shared hugetlb")?;
        let file_pme_mapped_in_kibibyte = u64::from_str_radix(file_pme_mapped_in_kibibyte, 10)
            .map_err(|_| "Can't parse file pme mapped")?;

        let shared_hugetlb_in_kibibyte = parts.next().ok_or("Can't parse shared hugetlb")?;
        let shared_hugetlb_in_kibibyte = u64::from_str_radix(shared_hugetlb_in_kibibyte, 10)
            .map_err(|_| "Can't parse shared hugetlb")?;

        let private_hugetlb_in_kibibyte = parts.next().ok_or("Can't parse private hugetlb")?;
        let private_hugetlb_in_kibibyte = u64::from_str_radix(private_hugetlb_in_kibibyte, 10)
            .map_err(|_| "Can't parse private hugetlb")?;

        let swap_in_kibibyte = parts.next().ok_or("Can't parse swap")?;
        let swap_in_kibibyte =
            u64::from_str_radix(swap_in_kibibyte, 10).map_err(|_| "Can't parse swap")?;

        let swap_pss_in_kibibyte = parts.next().ok_or("Can't parse swap pss")?;
        let swap_pss_in_kibibyte =
            u64::from_str_radix(swap_pss_in_kibibyte, 10).map_err(|_| "Can't parse swap pss")?;

        let locked_in_kibibyte = parts.next().ok_or("Can't parse locked")?;
        let locked_in_kibibyte =
            u64::from_str_radix(locked_in_kibibyte, 10).map_err(|_| "Can't parse locked")?;

        let transparent_huge_page_eligible = parts
            .next()
            .ok_or("Can't parse transparent huge page eligible")?;
        let transparent_huge_page_eligible = transparent_huge_page_eligible == "-1";

        let mut virtual_memory_flags = BitFlags::<VirtualMemoryFlags>::empty();

        let mut mapping_kind = "";

        for part in parts {
            match part {
                "rd" => virtual_memory_flags.toggle(VirtualMemoryFlags::Readable),
                "wr" => virtual_memory_flags.toggle(VirtualMemoryFlags::Writeable),
                "ex" => virtual_memory_flags.toggle(VirtualMemoryFlags::Executable),
                "sh" => virtual_memory_flags.toggle(VirtualMemoryFlags::Shared),
                "mr" => virtual_memory_flags.toggle(VirtualMemoryFlags::MayRead),
                "mw" => virtual_memory_flags.toggle(VirtualMemoryFlags::MayWrite),
                "me" => virtual_memory_flags.toggle(VirtualMemoryFlags::MayExecute),
                "ms" => virtual_memory_flags.toggle(VirtualMemoryFlags::MayShare),
                "gd" => virtual_memory_flags.toggle(VirtualMemoryFlags::GrowsDown),
                "pf" => virtual_memory_flags.toggle(VirtualMemoryFlags::PurePFNRange),
                "dw" => virtual_memory_flags.toggle(VirtualMemoryFlags::DisabledWriteToMappedFile),
                "lo" => virtual_memory_flags.toggle(VirtualMemoryFlags::Locked),
                "io" => virtual_memory_flags.toggle(VirtualMemoryFlags::Io),
                "sr" => {
                    virtual_memory_flags.toggle(VirtualMemoryFlags::SequentialReadAdviceProvided)
                }
                "rr" => virtual_memory_flags.toggle(VirtualMemoryFlags::RandomReadAdviceProvided),
                "dc" => virtual_memory_flags.toggle(VirtualMemoryFlags::DoNotCopyOnFork),
                "de" => virtual_memory_flags.toggle(VirtualMemoryFlags::DoNotExpandOnRemapping),
                "ac" => virtual_memory_flags.toggle(VirtualMemoryFlags::AreaIsAccountable),
                "nr" => virtual_memory_flags
                    .toggle(VirtualMemoryFlags::SwapSpaceIsNotReservedForTheArea),
                "ht" => virtual_memory_flags.toggle(VirtualMemoryFlags::AreaUsesHugeTlbPages),
                "sf" => virtual_memory_flags.toggle(VirtualMemoryFlags::SynchronousPageFault),
                "ar" => virtual_memory_flags.toggle(VirtualMemoryFlags::ArchitectureSpecific),
                "wf" => virtual_memory_flags.toggle(VirtualMemoryFlags::WipeOnFork),
                "dd" => virtual_memory_flags.toggle(VirtualMemoryFlags::DoNotIncludeInCoreDump),
                "sd" => virtual_memory_flags.toggle(VirtualMemoryFlags::SoftDirty),
                "mm" => virtual_memory_flags.toggle(VirtualMemoryFlags::MixedMapArea),
                "hg" => virtual_memory_flags.toggle(VirtualMemoryFlags::HugePageAdvise),
                "nh" => virtual_memory_flags.toggle(VirtualMemoryFlags::NoHugePageAdvise),
                "mg" => virtual_memory_flags.toggle(VirtualMemoryFlags::MergeableAdvise),
                "bt" => virtual_memory_flags.toggle(VirtualMemoryFlags::Arm64BTIGuardedPage),
                "mt" => virtual_memory_flags
                    .toggle(VirtualMemoryFlags::Arm64MTEAllocationTagsAreEnabled),
                "um" => virtual_memory_flags.toggle(VirtualMemoryFlags::UserfaultfdMissingTracking),
                "uw" => {
                    virtual_memory_flags.toggle(VirtualMemoryFlags::UserfaultfdWriteProtectTracking)
                }
                "ss" => virtual_memory_flags.toggle(VirtualMemoryFlags::ShadowStackPage),
                _ => {
                    let position = s.to_string().find(part).unwrap_or(s.len());
                    mapping_kind = &s[position..];
                    break;
                }
            }
        }

        let mapping_kind = MappingKind::from_str(mapping_kind)?;

        let result = PMap {
            address,
            permissions,
            offset,
            device_major,
            device_minor,
            inode,
            size_in_kibibyte,
            kernel_page_size_in_kibibyte,
            mmu_page_size_in_kibibyte,
            resident_set_size_in_kibibyte,
            proportional_share_size_in_kibibyte,
            proportional_share_size_dirty_in_kibibyte,
            shared_clean_in_kibibyte,
            shared_dirty_in_kibibyte,
            private_clean_in_kibibyte,
            private_dirty_in_kibibyte,
            referenced_in_kibibyte,
            anonymous_in_kibibyte,
            lazy_free_in_kibibyte,
            anonymous_huge_pages_in_kibibyte,
            shared_memory_associated_with_huge_pages_in_kibibyte,
            file_pme_mapped_in_kibibyte,
            shared_hugetlb_in_kibibyte,
            private_hugetlb_in_kibibyte,
            swap_in_kibibyte,
            swap_pss_in_kibibyte,
            locked_in_kibibyte,
            transparent_huge_page_eligible,
            virtual_memory_flags,
            mapping_kind,
        };

        Ok(result)
    }
}

impl Default for PMap {
    fn default() -> Self {
        Self {
            address: Default::default(),
            permissions: Default::default(),
            offset: Default::default(),
            device_major: Default::default(),
            device_minor: Default::default(),
            inode: Default::default(),
            size_in_kibibyte: Default::default(),
            kernel_page_size_in_kibibyte: Default::default(),
            mmu_page_size_in_kibibyte: Default::default(),
            resident_set_size_in_kibibyte: Default::default(),
            proportional_share_size_in_kibibyte: Default::default(),
            proportional_share_size_dirty_in_kibibyte: Default::default(),
            shared_clean_in_kibibyte: Default::default(),
            shared_dirty_in_kibibyte: Default::default(),
            private_clean_in_kibibyte: Default::default(),
            private_dirty_in_kibibyte: Default::default(),
            referenced_in_kibibyte: Default::default(),
            anonymous_in_kibibyte: Default::default(),
            lazy_free_in_kibibyte: Default::default(),
            anonymous_huge_pages_in_kibibyte: Default::default(),
            shared_memory_associated_with_huge_pages_in_kibibyte: Default::default(),
            file_pme_mapped_in_kibibyte: Default::default(),
            shared_hugetlb_in_kibibyte: Default::default(),
            private_hugetlb_in_kibibyte: Default::default(),
            swap_in_kibibyte: Default::default(),
            swap_pss_in_kibibyte: Default::default(),
            locked_in_kibibyte: Default::default(),
            transparent_huge_page_eligible: Default::default(),
            virtual_memory_flags: Default::default(),
            mapping_kind: MappingKind::AnonymousPrivate(None),
        }
    }
}

impl Display for PMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("| {:12x} | {:10} | {:30} | {:30} | {:150} |\n", self.address, self.size_in_kibibyte, self.mapping_kind, self.permissions.my_display(), self.virtual_memory_flags.my_display()).fmt(f)?;
        Ok(())
    }
}
// Permissions of an memory page
#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Permissions {
    // r - it is allowed to read the memory page
    Read,
    // w - it is allowed to write to the memory page
    Write,
    // x - it is allowed to execute the memory page
    Execute,
    // p - memory page is private (copy-on-write)
    Private,
    // s - memory page is shared
    Shared,
}

impl MyFromStr for BitFlags<Permissions> {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let mut permissions: BitFlags<Permissions> = BitFlags::empty();

        let mut parts = s.chars();

        let read = parts.next();
        if read == Some('r') {
            permissions.toggle(Permissions::Read);
        } else if read != Some('-') {
            return Err(format!("Can't parse permissions: {}", s).into());
        }

        let write = parts.next();
        if write == Some('w') {
            permissions.toggle(Permissions::Write);
        } else if write != Some('-') {
            return Err(format!("Can't parse permissions: {}", s).into());
        }

        let execute = parts.next();
        if execute == Some('x') {
            permissions.toggle(Permissions::Execute);
        } else if execute != Some('-') {
            return Err(format!("Can't parse permissions: {}", s).into());
        }

        let private_or_shared = parts.next();
        if private_or_shared == Some('p') {
            permissions.toggle(Permissions::Private);
        } else if private_or_shared == Some('s') {
            permissions.toggle(Permissions::Shared);
        } else if private_or_shared != Some('-') {
            return Err(format!("Can't parse permissions: {}", s).into());
        }
        if parts.next() != None {
            return Err(format!("Can't parse permissions: {}", s).into());
        }

        Ok(permissions)
    }
}

pub trait MyDisplay {
    fn my_display(&self) -> String;
}

impl MyDisplay for BitFlags<Permissions>{
    fn my_display(&self) -> String {
        let mut parts = Vec::new();

        if self.contains(Permissions::Read) {
            parts.push("Read - ");
        }

        if self.contains(Permissions::Write) {
            parts.push("Write - ");
        }

        if self.contains(Permissions::Execute) {
            parts.push("Execute - ");
        }

        if self.contains(Permissions::Private) {
            parts.push("Private");
        } else if self.contains(Permissions::Shared) {
            parts.push("Share");
        }

        parts.join("")
    }
}

// Flags of an memory page
#[bitflags]
#[repr(u64)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VirtualMemoryFlags {
    // rd
    Readable,
    // wr
    Writeable,
    // ex
    Executable,
    // sh
    Shared,
    // mr
    MayRead,
    // mw
    MayWrite,
    // me
    MayExecute,
    // ms
    MayShare,
    // gd
    GrowsDown,
    // pf
    PurePFNRange,
    // dw
    DisabledWriteToMappedFile,
    // lo
    Locked,
    // io
    Io,
    // sr
    SequentialReadAdviceProvided,
    // rr
    RandomReadAdviceProvided,
    // dc
    DoNotCopyOnFork,
    // de
    DoNotExpandOnRemapping,
    // ac
    AreaIsAccountable,
    // nr
    SwapSpaceIsNotReservedForTheArea,
    // ht
    AreaUsesHugeTlbPages,
    // sf
    SynchronousPageFault,
    // ar
    ArchitectureSpecific,
    // wf
    WipeOnFork,
    // dd
    DoNotIncludeInCoreDump,
    // sd
    SoftDirty,
    // mm
    MixedMapArea,
    // hg
    HugePageAdvise,
    // nh
    NoHugePageAdvise,
    // mg
    MergeableAdvise,
    // bt
    Arm64BTIGuardedPage,
    // mt
    Arm64MTEAllocationTagsAreEnabled,
    // um
    UserfaultfdMissingTracking,
    // uw
    UserfaultfdWriteProtectTracking,
    // ss
    ShadowStackPage,
}

pub trait MyFromStr: Sized {
    type Err;

    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

impl MyFromStr for BitFlags<VirtualMemoryFlags> {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut flags: BitFlags<VirtualMemoryFlags> = BitFlags::empty();
        let parts = s.split_whitespace();

        for part in parts {
            match part {
                "rd" => flags.toggle(VirtualMemoryFlags::Readable),
                "wr" => flags.toggle(VirtualMemoryFlags::Writeable),
                "ex" => flags.toggle(VirtualMemoryFlags::Executable),
                "sh" => flags.toggle(VirtualMemoryFlags::Shared),
                "mr" => flags.toggle(VirtualMemoryFlags::MayRead),
                "mw" => flags.toggle(VirtualMemoryFlags::MayWrite),
                "me" => flags.toggle(VirtualMemoryFlags::MayExecute),
                "ms" => flags.toggle(VirtualMemoryFlags::MayShare),
                "gd" => flags.toggle(VirtualMemoryFlags::GrowsDown),
                "pf" => flags.toggle(VirtualMemoryFlags::PurePFNRange),
                "dw" => flags.toggle(VirtualMemoryFlags::DisabledWriteToMappedFile),
                "lo" => flags.toggle(VirtualMemoryFlags::Locked),
                "io" => flags.toggle(VirtualMemoryFlags::Io),
                "sr" => flags.toggle(VirtualMemoryFlags::SequentialReadAdviceProvided),
                "rr" => flags.toggle(VirtualMemoryFlags::RandomReadAdviceProvided),
                "dc" => flags.toggle(VirtualMemoryFlags::DoNotCopyOnFork),
                "de" => flags.toggle(VirtualMemoryFlags::DoNotExpandOnRemapping),
                "ac" => flags.toggle(VirtualMemoryFlags::AreaIsAccountable),
                "nr" => flags.toggle(VirtualMemoryFlags::SwapSpaceIsNotReservedForTheArea),
                "ht" => flags.toggle(VirtualMemoryFlags::AreaUsesHugeTlbPages),
                "sf" => flags.toggle(VirtualMemoryFlags::SynchronousPageFault),
                "ar" => flags.toggle(VirtualMemoryFlags::ArchitectureSpecific),
                "wf" => flags.toggle(VirtualMemoryFlags::WipeOnFork),
                "dd" => flags.toggle(VirtualMemoryFlags::DoNotIncludeInCoreDump),
                "sd" => flags.toggle(VirtualMemoryFlags::SoftDirty),
                "mm" => flags.toggle(VirtualMemoryFlags::MixedMapArea),
                "hg" => flags.toggle(VirtualMemoryFlags::HugePageAdvise),
                "nh" => flags.toggle(VirtualMemoryFlags::NoHugePageAdvise),
                "mg" => flags.toggle(VirtualMemoryFlags::MergeableAdvise),
                "bt" => flags.toggle(VirtualMemoryFlags::Arm64BTIGuardedPage),
                "mt" => flags.toggle(VirtualMemoryFlags::Arm64MTEAllocationTagsAreEnabled),
                "um" => flags.toggle(VirtualMemoryFlags::UserfaultfdMissingTracking),
                "uw" => flags.toggle(VirtualMemoryFlags::UserfaultfdWriteProtectTracking),
                "ss" => flags.toggle(VirtualMemoryFlags::ShadowStackPage),
                _ => return Err(format!("Can't parse virtual memory flags: {}", s).into()),
            }
        }

        //let flags = VirtualMemoryFlags(flags.bits());
        Ok(flags)
    }
}

impl MyDisplay for BitFlags<VirtualMemoryFlags> {
    fn my_display(&self) -> String {
        let mut parts = Vec::new();

        if self.contains(VirtualMemoryFlags::Readable) {
            parts.push("Readable");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::Writeable) {
            parts.push("Writeable");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::Executable) {
            parts.push("Executable");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::Shared) {
            parts.push("Shared");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::MayRead) {
            parts.push("May Read");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::MayWrite) {
            parts.push("May Write");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::MayExecute) {
            parts.push("May Execute");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::MayShare) {
            parts.push("May Share");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::GrowsDown) {
            parts.push("Grows Down");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::PurePFNRange) {
            parts.push("Pure PFN Range");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::DisabledWriteToMappedFile) {
            parts.push("Disabled Write");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::Locked) {
            parts.push("Locked");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::Io) {
            parts.push("Io");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::SequentialReadAdviceProvided) {
            parts.push("Sequential Read");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::RandomReadAdviceProvided) {
            parts.push("Random Read");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::DoNotCopyOnFork) {
            parts.push("Do Not Copy");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::DoNotExpandOnRemapping) {
            parts.push("Do Not Expand");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::AreaIsAccountable) {
            parts.push("Area Is Accountable");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::SwapSpaceIsNotReservedForTheArea) {
            parts.push("Swap Space");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::AreaUsesHugeTlbPages) {
            parts.push("Huge TLB Pages");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::SynchronousPageFault) {
            parts.push("Synchronous Page Fault");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::ArchitectureSpecific) {
            parts.push("Architecture Specific");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::WipeOnFork) {
            parts.push("Wipe On Fork");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::DoNotIncludeInCoreDump) {
            parts.push("Not Include In Core Dump");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::SoftDirty) {
            parts.push("Soft Dirty");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::MixedMapArea) {
            parts.push("Mixed Map Area");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::HugePageAdvise) {
            parts.push("Huge Page");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::NoHugePageAdvise) {
            parts.push("No Huge Page");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::MergeableAdvise) {
            parts.push("Mergeable");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::Arm64BTIGuardedPage) {
            parts.push("Arm64 BTI");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::Arm64MTEAllocationTagsAreEnabled) {
            parts.push("Arm64 MTE");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::UserfaultfdMissingTracking) {
            parts.push("Userfaultfd Missing");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::UserfaultfdWriteProtectTracking) {
            parts.push("Userfaultfd Write Protect");
            parts.push(" - ");
        }

        if self.contains(VirtualMemoryFlags::ShadowStackPage) {
            parts.push("Shadow Stack");
            parts.push(" - ");
        }
        parts.remove(parts.len() - 1);
        parts.join("")
    }
}

#[derive(Debug, PartialEq)]
pub enum MappingKind {
    // [heap]
    Heap,
    // [stack]
    Stack,
    // [vdso]
    VirtualDynamicSharedObject,
    // [vvar]
    VirtualVariables,
    // [vsyscall]
    VirtualSystemCall,
    // [anon:<name>] or empty
    AnonymousPrivate(Option<String>),
    // [anon_shmem:<name>]
    AnonymousShared(Option<String>),
    // pathname
    File(FileInfo),
}

impl FromStr for MappingKind {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.starts_with('[') && s.ends_with(']') {
            let s = &s[1..s.len() - 1];
            if s == "heap" {
                Ok(MappingKind::Heap)
            } else if s == "stack" {
                Ok(MappingKind::Stack)
            } else if s == "vdso" {
                Ok(MappingKind::VirtualDynamicSharedObject)
            } else if s == "vvar" {
                Ok(MappingKind::VirtualVariables)
            } else if s == "vsyscall" {
                Ok(MappingKind::VirtualSystemCall)
            } else if s.starts_with("anon") {
                let s = &s[4..];
                if s.starts_with("_shmem:") {
                    if s.len() > 7 {
                        Ok(MappingKind::AnonymousShared(Some(s[7..].into())))
                    } else {
                        Ok(MappingKind::AnonymousShared(None))
                    }
                } else if s.starts_with(':') {
                    if s.len() == 1 {
                        Ok(MappingKind::AnonymousPrivate(None))
                    } else {
                        Ok(MappingKind::AnonymousPrivate(Some(s[1..].into())))
                    }
                } else {
                    Err("Invalid mapping kind".into())
                }
            } else {
                Err("Invalid mapping kind".into())
            }
        } else if s.is_empty() {
            Ok(MappingKind::AnonymousPrivate(None))
        } else {
            let fi = FileInfo::new(s);
            Ok(MappingKind::File(fi))
        }
    }
}

impl Display for MappingKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            MappingKind::Heap => format!("Heap").fmt(f),
            MappingKind::Stack => format!("Stack").fmt(f),
            MappingKind::VirtualDynamicSharedObject => format!("Virtual Dynamic Shared Object").fmt(f),
            MappingKind::VirtualVariables => format!("Virtual Variables").fmt(f),
            MappingKind::VirtualSystemCall => format!("Virtual System Call").fmt(f),
            MappingKind::AnonymousPrivate(None) => format!("Anonymous Private").fmt(f),
            MappingKind::AnonymousPrivate(Some(name)) => {
                format!("Anonymous Private ({})", name).fmt(f)
            }
            MappingKind::AnonymousShared(None) => format!("Anonymous Shared").fmt(f),
            MappingKind::AnonymousShared(Some(name)) => {
                format!("Anonymous Shared ({})", name).fmt(f)
            }
            MappingKind::File(fi) => format!("{}", fi.name()).fmt(f),
        }
    }
}

impl Clone for MappingKind {
    fn clone(&self) -> Self {
        match self {
            Self::Heap => Self::Heap,
            Self::Stack => Self::Stack,
            Self::VirtualDynamicSharedObject => Self::VirtualDynamicSharedObject,
            Self::VirtualVariables => Self::VirtualVariables,
            Self::VirtualSystemCall => Self::VirtualSystemCall,
            Self::AnonymousPrivate(arg0) => Self::AnonymousPrivate(arg0.clone()),
            Self::AnonymousShared(arg0) => Self::AnonymousShared(arg0.clone()),
            Self::File(arg0) => Self::File(FileInfo::new(arg0.full_name().clone())),
        }
    }
}
pub struct PMapVec(pub Vec<PMap>);

const MIN_SIZE_TO_DISPLAY: u64 = 10240;

impl Display for PMapVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pages_to_print = self.0
            .iter()
            .filter(|a| a.size_in_kibibyte >= MIN_SIZE_TO_DISPLAY)
            .collect::<Vec<_>>();
        let _ = &pages_to_print.sort_by(|a,b| b.size_in_kibibyte.cmp(&a.size_in_kibibyte));

        format!("|--------------|------------|--------------------------------|--------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|\n").fmt(f)?;
        format!("| {:^12} | {:^10} | {:^30} | {:^30} | {:150} |\n", "Address", "Size [KiB]", "Mapping Kind", "Permissions", "VM Flags").fmt(f)?;
        format!("|--------------|------------|--------------------------------|--------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|\n").fmt(f)?;
        for pmap in pages_to_print.iter() {
            pmap.fmt(f)?;
        }
        format!("|--------------|------------|--------------------------------|--------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|\n").fmt(f)?;

        writeln!(f)?;
        Ok(())
    }
}

impl Clone for PMapVec {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(test)]
mod pmap_tests {
    use super::*;
    use enumflags2::{bitflags, make_bitflags, BitFlags};

    #[test]
    fn mapping_kind_from_heap() {
        let input = "[heap]";
        let result: MappingKind = input.parse().unwrap();
        assert_eq!(result, MappingKind::Heap);
    }

    #[test]
    fn mapping_kind_from_stack() {
        let input = "[stack]";
        let result: MappingKind = input.parse().unwrap();
        assert_eq!(result, MappingKind::Stack);
    }

    #[test]
    fn mapping_kind_from_vdso() {
        let input = "[vdso]";
        let result: MappingKind = input.parse().unwrap();
        assert_eq!(result, MappingKind::VirtualDynamicSharedObject);
    }

    #[test]
    fn mapping_kind_from_anon() {
        let input = "[anon:]";
        let result: MappingKind = input.parse().unwrap();
        assert_eq!(result, MappingKind::AnonymousPrivate(None));
    }

    #[test]
    fn mapping_kind_from_empty() {
        let input = "";
        let result: MappingKind = input.parse().unwrap();
        assert_eq!(result, MappingKind::AnonymousPrivate(None));
    }

    #[test]
    fn mapping_kind_from_anon_named() {
        let input = "[anon:foo]";
        let result: MappingKind = input.parse().unwrap();
        assert_eq!(result, MappingKind::AnonymousPrivate(Some("foo".into())));
    }

    #[test]
    fn mapping_kind_from_anon_shmem() {
        let input = "[anon_shmem:]";
        let result: MappingKind = input.parse().unwrap();
        assert_eq!(result, MappingKind::AnonymousShared(None));
    }

    #[test]
    fn mapping_kind_from_anon_shmem_named() {
        let input = "[anon_shmem:bar]";
        let result: MappingKind = input.parse().unwrap();
        assert_eq!(result, MappingKind::AnonymousShared(Some("bar".into())));
    }

    #[test]
    fn vmflags_with_readable() {
        let input = "rd";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::Readable);
    }

    #[test]
    fn vmflags_with_writable() {
        let input = "wr";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::Writeable);
    }

    #[test]
    fn vmflags_with_executable() {
        let input = "ex";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::Executable);
    }

    #[test]
    fn vmflags_with_shared() {
        let input = "sh";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::Shared);
    }

    #[test]
    fn vmflags_with_may_read() {
        let input = "mr";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::MayRead);
    }

    #[test]
    fn vmflags_with_may_write() {
        let input = "mw";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::MayWrite);
    }

    #[test]
    fn vmflags_with_may_execute() {
        let input = "me";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::MayExecute);
    }

    #[test]
    fn vmflags_with_may_share() {
        let input = "ms";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::MayShare);
    }

    #[test]
    fn vmflags_with_grows_down() {
        let input = "gd";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::GrowsDown);
    }

    #[test]
    fn vmflags_with_pure_PFN_range() {
        let input = "pf";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::PurePFNRange);
    }

    #[test]
    fn vmflags_with_disable_write() {
        let input = "dw";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::DisabledWriteToMappedFile);
    }

    #[test]
    fn vmflags_with_locked() {
        let input = "lo";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::Locked);
    }

    #[test]
    fn vmflags_with_io() {
        let input = "io";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::Io);
    }

    #[test]
    fn vmflags_with_sequential_read_advise() {
        let input = "sr";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::SequentialReadAdviceProvided);
    }

    #[test]
    fn vmflags_with_random_read_advise() {
        let input = "rr";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::RandomReadAdviceProvided);
    }

    #[test]
    fn vmflags_with_do_not_copy() {
        let input = "dc";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::DoNotCopyOnFork);
    }

    #[test]
    fn vmflags_with_do_not_expand() {
        let input = "de";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::DoNotExpandOnRemapping);
    }

    #[test]
    fn vmflags_with_accountable() {
        let input = "ac";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::AreaIsAccountable);
    }

    #[test]
    fn vmflags_with_no_swap_space() {
        let input = "nr";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::SwapSpaceIsNotReservedForTheArea);
    }

    #[test]
    fn vmflags_with_area_uses_huge_tlb() {
        let input = "ht";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::AreaUsesHugeTlbPages);
    }

    #[test]
    fn vmflags_with_synchronous_page_fault() {
        let input = "sf";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::SynchronousPageFault);
    }

    #[test]
    fn vmflags_with_architecture_specific() {
        let input = "ar";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::ArchitectureSpecific);
    }

    #[test]
    fn vmflags_with_wipe_on_fork() {
        let input = "wf";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::WipeOnFork);
    }

    #[test]
    fn vmflags_with_not_include_in_dump() {
        let input = "dd";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::DoNotIncludeInCoreDump);
    }

    #[test]
    fn vmflags_with_soft_dirty_flag() {
        let input = "sd";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::SoftDirty);
    }

    #[test]
    fn vmflags_with_mixed_map() {
        let input = "mm";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::MixedMapArea);
    }

    #[test]
    fn vmflags_with_huge_page() {
        let input = "hg";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::HugePageAdvise);
    }

    #[test]
    fn vmflags_with_no_huge_page() {
        let input = "nh";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::NoHugePageAdvise);
    }

    #[test]
    fn vmflags_with_mergeable() {
        let input = "mg";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::MergeableAdvise);
    }

    #[test]
    fn vmflags_with_arm64_bti_guard() {
        let input = "bt";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::Arm64BTIGuardedPage);
    }

    #[test]
    fn vmflags_with_arm64_mte_allocation() {
        let input = "mt";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::Arm64MTEAllocationTagsAreEnabled);
    }

    #[test]
    fn vmflags_with_userfaultfd_missing_tracking() {
        let input = "um";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::UserfaultfdMissingTracking);
    }

    #[test]
    fn vmflags_with_userfaultfd_wr_protect() {
        let input = "uw";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::UserfaultfdWriteProtectTracking);
    }

    #[test]
    fn vmflags_with_shadow_stack() {
        let input = "ss";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(result, VirtualMemoryFlags::ShadowStackPage);
    }

    #[test]
    fn vmflags_combinatorics_test() {
        let input = "rd ex sh mr mw me ms sd";
        let result = BitFlags::<VirtualMemoryFlags>::from_str(input).unwrap();
        assert_eq!(
            result,
            make_bitflags!(VirtualMemoryFlags::{Readable | Executable | Shared | MayRead | MayWrite | MayExecute | MayShare | SoftDirty})
        );
    }

    #[test]
    fn permissions_with_read() {
        let input = "r---";
        let result = BitFlags::<Permissions>::from_str(input).unwrap();
        assert_eq!(result, Permissions::Read);
    }

    #[test]
    fn permissions_with_write() {
        let input = "-w--";
        let result = BitFlags::<Permissions>::from_str(input).unwrap();
        assert_eq!(result, Permissions::Write);
    }

    #[test]
    fn permissions_with_execute() {
        let input = "--x-";
        let result = BitFlags::<Permissions>::from_str(input).unwrap();
        assert_eq!(result, Permissions::Execute);
    }

    #[test]
    fn permissions_with_private() {
        let input = "---p";
        let result = BitFlags::<Permissions>::from_str(input).unwrap();
        assert_eq!(result, Permissions::Private);
    }

    #[test]
    fn permissions_with_shared() {
        let input = "---s";
        let result = BitFlags::<Permissions>::from_str(input).unwrap();
        assert_eq!(result, Permissions::Shared);
    }

    #[test]
    fn permissions_combinatorics_test() {
        let input = "r-xs";
        let result = BitFlags::<Permissions>::from_str(input).unwrap();
        assert_eq!(
            result,
            make_bitflags!(Permissions::{Read | Execute | Shared})
        );
    }

    #[test]
    fn pmap_from_str_test() {
        //                      Adresse Zugr  Versatz Gerät   Inode      Size KernelPageSize MMUPageSize    Rss    Pss Pss_Dirty Shared_Clean Shared_Dirty Private_Clean Private_Dirty Referenced Anonymous LazyFree AnonHugePages ShmemPmdMapped FilePmdMapped Shared_Hugetlb Private_Hugetlb Swap SwapPss Locked THPeligible                 VmFlags Zuordnung
        let input = "7faf68872000 rw-p 02743000  00:01    4128         4              4           4      1      2         3            4            5             6             7          8         9        1             2              3             4              5               6    7       8      9          -1 rd ex sh mr mw me ms sd memfd:doublemapper (deleted)";
        let result = PMap::from_str(input).unwrap();
        assert_eq!(result.address, 0x7faf68872000);
        assert_eq!(
            result.permissions,
            make_bitflags!(Permissions::{Read | Write | Private})
        );
        assert_eq!(result.offset, 0x02743000);
        assert_eq!(result.device_major, 0x00);
        assert_eq!(result.device_minor, 0x01);
        assert_eq!(result.inode, 4128);
        assert_eq!(result.size_in_kibibyte, 4);
        assert_eq!(result.kernel_page_size_in_kibibyte, 4);
        assert_eq!(result.mmu_page_size_in_kibibyte, 4);
        assert_eq!(result.resident_set_size_in_kibibyte, 1);
        assert_eq!(result.proportional_share_size_in_kibibyte, 2);
        assert_eq!(result.proportional_share_size_dirty_in_kibibyte, 3);
        assert_eq!(result.shared_clean_in_kibibyte, 4);
        assert_eq!(result.shared_dirty_in_kibibyte, 5);
        assert_eq!(result.private_clean_in_kibibyte, 6);
        assert_eq!(result.private_dirty_in_kibibyte, 7);
        assert_eq!(result.referenced_in_kibibyte, 8);
        assert_eq!(result.anonymous_in_kibibyte, 9);
        assert_eq!(result.lazy_free_in_kibibyte, 1);
        assert_eq!(result.anonymous_huge_pages_in_kibibyte, 2);
        assert_eq!(
            result.shared_memory_associated_with_huge_pages_in_kibibyte,
            3
        );
        assert_eq!(result.file_pme_mapped_in_kibibyte, 4);
        assert_eq!(result.shared_hugetlb_in_kibibyte, 5);
        assert_eq!(result.private_hugetlb_in_kibibyte, 6);
        assert_eq!(result.swap_in_kibibyte, 7);
        assert_eq!(result.swap_pss_in_kibibyte, 8);
        assert_eq!(result.locked_in_kibibyte, 9);
        assert_eq!(result.transparent_huge_page_eligible, true);
        assert_eq!(
            result.virtual_memory_flags,
            make_bitflags!(VirtualMemoryFlags::{Readable | Executable | Shared | MayRead | MayWrite | MayExecute | MayShare | SoftDirty})
        );
        assert_eq!(
            result.mapping_kind,
            MappingKind::File(FileInfo::new("memfd:doublemapper (deleted)"))
        );
    }
}
