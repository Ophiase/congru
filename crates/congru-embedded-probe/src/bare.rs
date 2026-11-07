use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::hint::black_box;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicUsize, Ordering};

use congru_embedded::Real;

const ARENA_BYTES: usize = 64 * 1024;

struct Arena {
    memory: UnsafeCell<[u8; ARENA_BYTES]>,
    used: AtomicUsize,
}

#[expect(unsafe_code, reason = "the claimed ranges never overlap")]
unsafe impl Sync for Arena {}

#[expect(unsafe_code, reason = "a global allocator can only be written so")]
unsafe impl GlobalAlloc for Arena {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let base = self.memory.get().cast::<u8>();
        let start =
            (self.used.load(Ordering::Relaxed) + layout.align() - 1) & !(layout.align() - 1);
        let end = start + layout.size();
        if end > ARENA_BYTES {
            return core::ptr::null_mut();
        }
        self.used.store(end, Ordering::Relaxed);
        unsafe { base.add(start) }
    }

    unsafe fn dealloc(&self, _pointer: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ARENA: Arena = Arena {
    memory: UnsafeCell::new([0; ARENA_BYTES]),
    used: AtomicUsize::new(0),
};

#[panic_handler]
fn halt(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// The body of `_start`: runs the workout, then parks.
pub fn run<Float: Real>() -> ! {
    black_box(crate::workout::<Float>()).ok();
    loop {
        core::hint::spin_loop();
    }
}
