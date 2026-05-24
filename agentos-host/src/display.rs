#[cfg(target_os = "macos")]
use std::ffi::c_void;
#[cfg(target_os = "macos")]
use std::ptr;
#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
#[cfg(target_os = "macos")]
use std::sync::{Arc, Mutex};

#[cfg(target_os = "macos")]
use crate::krun_ffi::*;

#[cfg(target_os = "macos")]
type IOSurfaceRef = *mut c_void;
#[cfg(target_os = "macos")]
type CFDictionaryRef = *const c_void;
#[cfg(target_os = "macos")]
type CFStringRef = *const c_void;
#[cfg(target_os = "macos")]
type CFNumberRef = *const c_void;
#[cfg(target_os = "macos")]
type CFAllocatorRef = *const c_void;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn IOSurfaceCreate(properties: CFDictionaryRef) -> IOSurfaceRef;
    fn IOSurfaceGetBaseAddress(surface: IOSurfaceRef) -> *mut c_void;
    fn IOSurfaceLock(surface: IOSurfaceRef, options: u32, seed: *mut u32) -> i32;
    fn IOSurfaceUnlock(surface: IOSurfaceRef, options: u32, seed: *mut u32) -> i32;
    fn IOSurfaceGetAllocSize(surface: IOSurfaceRef) -> usize;
    fn CFRelease(obj: *const c_void);

    static kIOSurfaceWidth: CFStringRef;
    static kIOSurfaceHeight: CFStringRef;
    static kIOSurfaceBytesPerRow: CFStringRef;
    static kIOSurfaceBytesPerElement: CFStringRef;
    static kIOSurfacePixelFormat: CFStringRef;

    fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CFDictionaryRef;
    fn CFNumberCreate(
        allocator: CFAllocatorRef,
        the_type: isize,
        value_ptr: *const c_void,
    ) -> CFNumberRef;

    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
}

#[cfg(target_os = "macos")]
const K_CF_NUMBER_INT_TYPE: isize = 9; // kCFNumberIntType

#[cfg(target_os = "macos")]
fn create_iosurface(width: u32, height: u32) -> IOSurfaceRef {
    let stride = width * 4;
    let pixel_format: u32 = 0x42475241; // 'BGRA'
    let bytes_per_element: u32 = 4;

    unsafe {
        let w_num = CFNumberCreate(ptr::null(), K_CF_NUMBER_INT_TYPE, &width as *const _ as *const _);
        let h_num = CFNumberCreate(ptr::null(), K_CF_NUMBER_INT_TYPE, &height as *const _ as *const _);
        let stride_num = CFNumberCreate(ptr::null(), K_CF_NUMBER_INT_TYPE, &stride as *const _ as *const _);
        let bpe_num = CFNumberCreate(ptr::null(), K_CF_NUMBER_INT_TYPE, &bytes_per_element as *const _ as *const _);
        let fmt_num = CFNumberCreate(ptr::null(), K_CF_NUMBER_INT_TYPE, &pixel_format as *const _ as *const _);

        let keys: [*const c_void; 5] = [
            kIOSurfaceWidth,
            kIOSurfaceHeight,
            kIOSurfaceBytesPerRow,
            kIOSurfaceBytesPerElement,
            kIOSurfacePixelFormat,
        ];
        let values: [*const c_void; 5] = [
            w_num as *const _,
            h_num as *const _,
            stride_num as *const _,
            bpe_num as *const _,
            fmt_num as *const _,
        ];

        let dict = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            5,
            &kCFTypeDictionaryKeyCallBacks as *const _ as *const _,
            &kCFTypeDictionaryValueCallBacks as *const _ as *const _,
        );

        let surface = IOSurfaceCreate(dict);

        CFRelease(dict);
        CFRelease(w_num as *const _);
        CFRelease(h_num as *const _);
        CFRelease(stride_num as *const _);
        CFRelease(bpe_num as *const _);
        CFRelease(fmt_num as *const _);

        surface
    }
}

#[cfg(target_os = "macos")]
const NUM_SURFACES: usize = 4;

#[cfg(target_os = "macos")]
pub struct DisplayState {
    inner: Mutex<SurfacePool>,
    frame_seq: AtomicU64,
    last_seen: AtomicU64,
    width: AtomicU32,
    height: AtomicU32,
    stride: AtomicU32,
}

#[cfg(target_os = "macos")]
struct SurfacePool {
    surfaces: [IOSurfaceRef; NUM_SURFACES],
    write_idx: usize,
    ready_idx: Option<usize>,
    display_idx: Option<usize>,
    prev_display_idx: Option<usize>,
}

#[cfg(target_os = "macos")]
unsafe impl Send for SurfacePool {}
#[cfg(target_os = "macos")]
unsafe impl Sync for SurfacePool {}

#[cfg(target_os = "macos")]
impl DisplayState {
    fn new() -> Self {
        Self {
            inner: Mutex::new(SurfacePool {
                surfaces: [ptr::null_mut(); NUM_SURFACES],
                write_idx: 0,
                ready_idx: None,
                display_idx: None,
                prev_display_idx: None,
            }),
            frame_seq: AtomicU64::new(0),
            last_seen: AtomicU64::new(0),
            width: AtomicU32::new(0),
            height: AtomicU32::new(0),
            stride: AtomicU32::new(0),
        }
    }

    pub fn vm_width(&self) -> u32 {
        self.width.load(Ordering::Relaxed)
    }

    pub fn vm_height(&self) -> u32 {
        self.height.load(Ordering::Relaxed)
    }

    pub fn get_front_surface(&self) -> Option<IOSurfaceRef> {
        let seq = self.frame_seq.load(Ordering::Acquire);
        let seen = self.last_seen.load(Ordering::Relaxed);
        if seq == seen {
            return None;
        }
        self.last_seen.store(seq, Ordering::Relaxed);

        let mut guard = self.inner.lock().ok()?;
        let ready = guard.ready_idx.take()?;
        guard.prev_display_idx = guard.display_idx;
        guard.display_idx = Some(ready);
        let surface = guard.surfaces[ready];
        if surface.is_null() { return None; }

        Some(surface)
    }
}

#[cfg(target_os = "macos")]
static DISPLAY: std::sync::OnceLock<Arc<DisplayState>> = std::sync::OnceLock::new();

#[cfg(target_os = "macos")]
pub fn global_display() -> Arc<DisplayState> {
    DISPLAY.get_or_init(|| Arc::new(DisplayState::new())).clone()
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn cb_create(
    instance: *mut *mut c_void,
    _userdata: *const c_void,
    _reserved: *const c_void,
) -> i32 {
    unsafe { *instance = ptr::null_mut() };
    tracing::info!("display backend created");
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn cb_destroy(_instance: *mut c_void) -> i32 {
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn cb_configure_scanout(
    _instance: *mut c_void,
    _scanout_id: u32,
    _display_width: u32,
    _display_height: u32,
    width: u32,
    height: u32,
    _format: u32,
) -> i32 {
    let stride = width * 4;
    let display = global_display();
    let old_w = display.width.load(Ordering::Relaxed);
    let old_h = display.height.load(Ordering::Relaxed);
    if old_w == width && old_h == height {
        return 0;
    }
    tracing::info!(width, height, "display scanout configured");
    display.width.store(width, Ordering::Relaxed);
    display.height.store(height, Ordering::Relaxed);
    display.stride.store(stride, Ordering::Relaxed);

    let mut new_surfaces = [ptr::null_mut(); NUM_SURFACES];
    for s in &mut new_surfaces {
        *s = create_iosurface(width, height);
        if s.is_null() {
            tracing::error!("failed to create IOSurface");
            for prev in new_surfaces.iter().filter(|p| !p.is_null()) {
                unsafe { CFRelease(*prev as *const _); }
            }
            return -1;
        }
    }
    tracing::info!("created IOSurface quad buffer ({NUM_SURFACES} surfaces)");

    let mut guard = display.inner.lock().unwrap();
    let old_surfaces = guard.surfaces;
    guard.surfaces = new_surfaces;
    guard.write_idx = 0;
    guard.ready_idx = None;
    guard.display_idx = None;
    guard.prev_display_idx = None;
    drop(guard);

    unsafe {
        for s in &old_surfaces {
            if !s.is_null() {
                CFRelease(*s as *const _);
            }
        }
    }

    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn cb_disable_scanout(_instance: *mut c_void, _scanout_id: u32) -> i32 {
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn cb_alloc_frame(
    _instance: *mut c_void,
    _scanout_id: u32,
    buffer: *mut *mut u8,
    buffer_size: *mut usize,
) -> i32 {
    let display = global_display();
    let guard = display.inner.lock().unwrap();

    let surface = guard.surfaces[guard.write_idx];
    if surface.is_null() {
        return -1;
    }

    unsafe {
        IOSurfaceLock(surface, 0, ptr::null_mut());
        let base = IOSurfaceGetBaseAddress(surface);
        let size = IOSurfaceGetAllocSize(surface);
        *buffer = base as *mut u8;
        *buffer_size = size;
    }
    1
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn cb_present_frame(
    _instance: *mut c_void,
    _scanout_id: u32,
    _frame_id: u32,
    _damage_area: *const KrunRect,
) -> i32 {
    let display = global_display();
    let mut guard = display.inner.lock().unwrap();

    let finished_idx = guard.write_idx;
    let surface = guard.surfaces[finished_idx];
    unsafe {
        IOSurfaceUnlock(surface, 0, ptr::null_mut());
    }

    guard.ready_idx = Some(finished_idx);
    let disp = guard.display_idx.unwrap_or(usize::MAX);
    let prev_disp = guard.prev_display_idx.unwrap_or(usize::MAX);
    for candidate in 0..NUM_SURFACES {
        if candidate != finished_idx && candidate != disp && candidate != prev_disp {
            guard.write_idx = candidate;
            break;
        }
    }

    display.frame_seq.fetch_add(1, Ordering::Release);

    0
}

#[cfg(target_os = "macos")]
pub fn create_backend() -> KrunDisplayBackend {
    let mut backend: KrunDisplayBackend = unsafe { std::mem::zeroed() };
    backend.features = KRUN_DISPLAY_FEATURE_BASIC_FRAMEBUFFER;
    backend.create_userdata = ptr::null_mut();
    backend.create = Some(cb_create);
    backend.vtable.basic_framebuffer = std::mem::ManuallyDrop::new(KrunDisplayBasicFramebufferVtable {
        destroy: Some(cb_destroy),
        disable_scanout: Some(cb_disable_scanout),
        configure_scanout: Some(cb_configure_scanout),
        alloc_frame: Some(cb_alloc_frame),
        present_frame: Some(cb_present_frame),
    });
    backend
}
