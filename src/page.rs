use core::{mem::size_of, ptr::null_mut};

// MEMORY LAYOUT
// [PAGE TABLE]
// +--> Page table 1 bits {Empty, Taken, Last}
// +--> Page table 2 bits {Empty, Taken, Last}
// +--> Page table 3 bits {Empty, Taken, Last}
// ...
// [FREE PAGE 1] <-- ALLOC_START
// [FREE PAGE 2] <-- (ALLOC_START + 1 * PAGE_SIZE)
// [FREE PAGE 3] <-- (ALLOC_START + 2 * PAGE_SIZE)
// ...
// [MEMORY_END]

// Don't need pointers to our pages since our memory layout
// is indexed by page size

// below are computed and linked using the linker script
extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

static mut ALLOC_START: usize = 0;
const PAGE_ORDER: usize = 12;
pub const PAGE_SIZE: usize = 1 << 12; // 4096-byte pages

// align value to a given order
pub const fn align_val(val: usize, order: usize) -> usize {
    let o = (1usize << order) - 1;
    (val + o) & !o
}

// Could use a linked list structure for tracking memory,
// but instead we're tracking using indexing to save memory
// struct FreePages {
//     struct FreePages *next;
// };

#[repr(u8)]
pub enum PageBits {
    Empty = 0,
    Taken = 1 << 0, // page taken?
    Last = 1 << 1,  // last page in contiguous allocation?
}

impl PageBits {
    pub fn val(self) -> u8 {
        self as u8
    }
}

// num_pages of these structs are written at the start of memory
pub struct Page {
    flags: u8,
}

impl Page {
    pub fn is_last(&self) -> bool {
        if self.flags & PageBits::Last.val() != 0 {
            true
        } else {
            false
        }
    }

    pub fn is_taken(&self) -> bool {
        if self.flags & PageBits::Taken.val() != 0 {
            true
        } else {
            false
        }
    }

    pub fn is_free(&self) -> bool {
        !self.is_taken()
    }

    pub fn clear(&mut self) {
        self.flags = PageBits::Empty.val();
    }

    pub fn set_flag(&mut self, flag: PageBits) {
        self.flags |= flag.val();
    }

    pub fn clear_flag(&mut self, flag: PageBits) {
        self.flags &= !(flag.val());
    }
}

// initialize the page allocator
pub fn init() {
    unsafe {
        let num_pages = HEAP_SIZE / PAGE_SIZE;
        let ptr = HEAP_START as *mut Page;

        for i in 0..num_pages {
            (*ptr.add(i)).clear();
        }

        // start of usable memory is after page table
        // ALLOC_START = align_val(HEAP_START + num_pages * size_of::<Page>(), PAGE_ORDER);
        ALLOC_START = align_val(HEAP_START + num_pages * size_of::<Page>(), PAGE_ORDER);
    }
}

// allocate a new page in memory
pub fn alloc(pages: usize) -> *mut u8 {
    assert!(pages > 0);
    unsafe {
        let num_pages = HEAP_SIZE / PAGE_SIZE;
        let ptr = HEAP_START as *mut Page;
        for i in 0..num_pages - pages {
            let mut found = false;

            if (*ptr.add(i)).is_free() {
                // page is free
                found = true;
                for j in i..i + pages {
                    if (*ptr.add(j)).is_taken() {
                        found = false;
                        break;
                    }
                }
            }

            if found {
                for k in i..i + pages - 1 {
                    // set number pages requested to taken
                    (*ptr.add(k)).set_flag(PageBits::Taken);
                }
                (*ptr.add(i + pages - 1)).set_flag(PageBits::Taken);
                (*ptr.add(i + pages - 1)).set_flag(PageBits::Last);

                return (ALLOC_START + PAGE_SIZE * i) as *mut u8;
            }
        }
    }
    // return a null mutable pointer to indicate no available pages
    null_mut()
}

// deallocate a page given is pointer
pub fn dealloc(page_ptr: *mut u8) {
    assert!(!page_ptr.is_null());
    unsafe {
        let page_addr = HEAP_START + (page_ptr as usize - ALLOC_START) / PAGE_SIZE;
        // make sure address for page struct is within memory
        assert!(page_addr >= HEAP_START && page_addr < HEAP_START + HEAP_SIZE);
        let mut p = page_addr as *mut Page;

        while (*p).is_taken() && !(*p).is_last() {
            (*p).clear();
            p = p.add(1);
        }

        // didn't reach last page before hitting untaken page
        assert!((*p).is_last() == true, "Possible double-free!");

        (*p).clear();
    }
}

// allocate and zero a page(s)
pub fn zalloc(pages: usize) -> *mut u8 {
    let ret = alloc(pages);
    if !ret.is_null() {
        let size = (PAGE_SIZE * pages) / 8;
        let big_ptr = ret as *mut u64;
        for i in 0..size {
            // using big_ptr so we go double-word (DW) writes
            // instead of single byte (SB)
            unsafe {
            }
        }
    }
    ret
}

/// Print all page allocations
/// This is mainly used for debugging.
pub fn print_page_allocations() {
    unsafe {
        let num_pages = HEAP_SIZE / PAGE_SIZE;
        let mut beg = HEAP_START as *const Page;
        let end = beg.add(num_pages);
        let alloc_beg = ALLOC_START;
        let alloc_end = ALLOC_START + num_pages * PAGE_SIZE;
        println!();
        println!(
            "PAGE ALLOCATION TABLE\nMETA: {:p} -> {:p}\nPHYS: \
             0x{:x} -> 0x{:x}",
            beg, end, alloc_beg, alloc_end
        );
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        let mut num = 0;
        while beg < end {
            if (*beg).is_taken() {
                let start = beg as usize;
                let memaddr = ALLOC_START + (start - HEAP_START) * PAGE_SIZE;
                print!("0x{:x} => ", memaddr);
                loop {
                    num += 1;
                    if (*beg).is_last() {
                        let end = beg as usize;
                        let memaddr = ALLOC_START + (end - HEAP_START) * PAGE_SIZE + PAGE_SIZE - 1;
                        print!("0x{:x}: {:>3} page(s)", memaddr, (end - start + 1));
                        println!(".");
                        break;
                    }
                    beg = beg.add(1);
                }
            }
            beg = beg.add(1);
        }
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        println!(
            "Allocated: {:>5} pages ({:>9} bytes).",
            num,
            num * PAGE_SIZE
        );
        println!(
            "Free     : {:>5} pages ({:>9} bytes).",
            num_pages - num,
            (num_pages - num) * PAGE_SIZE
        );
        println!();
    }
}

// ==========================================================================================================
// MMU routine
// ==========================================================================================================

// all options are unsigned 64-bit regs
// and our copy/clone fn are implicit
#[repr(i64)]
#[derive(Copy, Clone)]
pub enum EntryBits {
    None = 0,
    Valid = 1 << 0,
    Read = 1 << 1,
    Write = 1 << 2,
    Execute = 1 << 3,
    User = 1 << 4,
    Global = 1 << 5,
    Access = 1 << 6,
    Dirty = 1 << 7,

    // convenience combinations
    RW = 1 << 1 | 1 << 2,
    RE = 1 << 1 | 1 << 3,
    RWE = 1 << 1 | 1<< 2 | 1 << 3,

    // user convenient combinations
    URW = 1 << 1 | 1 << 2 | 1 << 4,
    URE = 1 << 1 | 1 << 3 | 1 << 4,
    URWE = 1 << 1 | 1<< 2 | 1 << 3 | 1 << 4,
}

impl EntryBits {
    pub fn val(self) -> i64 {
        self as i64
    }
}

pub struct Entry {
    pub entry: i64,
}

impl Entry {
    // check if valid bit is set
    pub fn is_valid(&self) -> bool {
        (self.get_entry() & EntryBits::Valid.val()) != 0
    }

    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    // 0xe == W or X since only leaf would have these set
    pub fn is_leaf(&self) -> bool {
        (self.get_entry() & 0xe) != 0
    }

    pub fn is_branch(&self) -> bool {
        !self.is_leaf()
    }

    pub fn set_entry(&mut self, entry: i64) {
        self.entry = entry;
    }

    pub fn get_entry(&self) -> i64 {
        self.entry
    }
}

pub struct Table {
    pub entries: [Entry; 512],
}

impl Table {
    pub fn len() -> usize {
        size_of::<Table>()
    }
}

// Map a virt address to a physical address in a 4096-byte page
// root: top-level mapping table
// vaddr: virt addr to map
// paddr: phys addr to map
// bits: the privilege bits the page should have
// level: the level to start at (always 0)
pub fn map(root: &mut Table, vaddr: usize, paddr: usize, bits: i64, level: usize) {
    // make sure we have a leaf
    assert!(bits & 0xe != 0);

    // each vpn is 9 bits (0b1_1111_1111)
    let vpn = [
        // VPN[0] = virt addr bits 20-12
        (vaddr >> 12) & 0x1ff,
        // VPN[1] = virt addr 29-21
        (vaddr >> 21) & 0x1ff,
        // VPN[2] = virt addr 38-30
        (vaddr >> 30) & 0x1ff,

    ];

    // each ppn is 9 bits except the last 1 is 26 bits
    let ppn = [
        // PPN[0] = paddr[20:12]
        (paddr >> 12) & 0x1ff,
        // PPN[1] = paddr[29:21]
        (paddr >> 21) & 0x1ff,
        // PPN[2] = paddr[55:30]
        (paddr >> 30) & 0x3ff_ffff,
    ];

    let mut v = &mut root.entries[vpn[2]];

    for i in (level..2).rev() {
        if !v.is_valid() {
            let page = zalloc(1);

            // v's entry is a 64-bit heap address that's 4096 byte aligned
            // shifted right by 2 to make space for flags
            v.set_entry(
                (page as i64 >> 2)
                | EntryBits::Valid.val(),
            );
        }

        // the page we get should already be 4096 byte-aligned
        // and would be the page table for this lower set of pages
        let entry = ((v.get_entry() & !0x3ff) << 2) as *mut Entry;
        // get the address of the next page table starting point
        v = unsafe { entry.add(vpn[i]).as_mut().unwrap() };
    }
    // after the prev loop, v is now pointing to the
    // entry loc in the mapping table (virt->phys)

    // need to shift paddr vals to correct value for page table entry
    let entry = (ppn[2] << 28) as i64 |   // PPN[2] = [53:28]
    (ppn[1] << 19) as i64 |   // PPN[1] = [27:19]
    (ppn[0] << 10) as i64 |   // PPN[0] = [18:10]
    bits |                    // Specified bits, such as User, Read, Write, etc
    EntryBits::Valid.val();   // Valid bit

    v.set_entry(entry);

}

pub fn unmap(root: &mut Table) {
    for lv2 in 0..Table::len() {
        let ref entry_lv2 = root.entries[lv2];
        if entry_lv2.is_valid() && entry_lv2.is_branch() {
            // valid entry, free it and the lower table entries
            let memaddr_lv1 = (entry_lv2.get_entry() & !0x3ff) << 2;
            let table_lv1 = unsafe {
                (memaddr_lv1 as *mut Table).as_mut().unwrap()
            };
            for lv1 in 0..Table::len() {
                let ref entry_lv1 = table_lv1.entries[lv1];
                if entry_lv1.is_valid() && entry_lv1.is_branch() {
                    let memaddr_lv0 = (entry_lv1.get_entry() & !0x3ff) << 2;

                    // last level, free it
                    dealloc(memaddr_lv0 as *mut u8);
                }
            }

            dealloc(memaddr_lv1 as *mut u8);

        }
    }
}

pub fn virt_to_phys(root: &Table, vaddr: usize) ->  Option<usize> {
    // Walk the page table
    let vpn = [
        // VPN[0] = virt addr bits 20-12
        (vaddr >> 12) & 0x1ff,
        // VPN[1] = virt addr 29-21
        (vaddr >> 21) & 0x1ff,
        // VPN[2] = virt addr 38-30
        (vaddr >> 30) & 0x1ff,
    ];

    let mut v = &root.entries[vpn[2]];
    for i in (0..=2).rev() {
        if v.is_invalid() {
            // invalid, send a page fault
            break;
        }
        else if v.is_leaf() {
            // if we're at a leaf then read and return the PPN
            // PPN is 9 bits and starts at bit 12
            let off_mask = (1 << (12 + i * 9)) - 1;
            let vaddr_pgoff = vaddr & off_mask;
            let addr = ((v.get_entry() << 2) as usize) & !off_mask;
            return Some(addr | vaddr_pgoff);
        }

        let entry = ((v.get_entry() & !0x3ff) << 2) as *const Entry;

        v = unsafe { entry.add(vpn[i-1]).as_ref().unwrap() };
    }

    None
}

