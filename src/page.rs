use core::{mem::size_of, ptr::null_mut};

// below are computed and linked using the linker script
extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

static mut ALLOC_START: usize = 0;
const PAGE_ORDER: usize = 12;
pub const PAGE_SIZE: usize = 1 << 12; // 4096

// align value to a given order
pub const fn align_val(val: usize, order: usize) -> usize {
    let o = (1usize << order) - 1;
    (val + o) & !o
}

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
                    if (*ptr.add(i)).is_taken() {
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
                (*ptr.add(1 + pages - 1)).set_flag(PageBits::Taken);
                (*ptr.add(1 + pages - 1)).set_flag(PageBits::Last);

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
                (*big_ptr.add(i)) = 0;
            }
        }
    }
    ret
}
