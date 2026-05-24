//! Virtual keyboard and mouse input devices for libkrun.

#![allow(dead_code)]

#[cfg(target_os = "macos")]
use std::collections::VecDeque;
#[cfg(target_os = "macos")]
use std::ffi::c_void;
#[cfg(target_os = "macos")]
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "macos")]
const EV_SYN: u16 = 0x00;
#[cfg(target_os = "macos")]
const EV_KEY: u16 = 0x01;
#[cfg(target_os = "macos")]
const EV_REL: u16 = 0x02;
#[cfg(target_os = "macos")]
const EV_ABS: u16 = 0x03;

#[cfg(target_os = "macos")]
const SYN_REPORT: u16 = 0x00;

#[cfg(target_os = "macos")]
const REL_X: u16 = 0x00;
#[cfg(target_os = "macos")]
const REL_Y: u16 = 0x01;
#[cfg(target_os = "macos")]
const REL_HWHEEL: u16 = 0x06;
#[cfg(target_os = "macos")]
const REL_WHEEL: u16 = 0x08;

#[cfg(target_os = "macos")]
const ABS_X: u16 = 0x00;
#[cfg(target_os = "macos")]
const ABS_Y: u16 = 0x01;

#[cfg(target_os = "macos")]
const ABS_MAX: u32 = 32767;

#[cfg(target_os = "macos")]
const INPUT_PROP_DIRECT: u16 = 0x01;

#[cfg(target_os = "macos")]
const BTN_LEFT: u16 = 0x110;
#[cfg(target_os = "macos")]
const BTN_RIGHT: u16 = 0x111;
#[cfg(target_os = "macos")]
const BTN_MIDDLE: u16 = 0x112;

#[cfg(target_os = "macos")]
const BUS_VIRTUAL: u16 = 0x00;

#[cfg(target_os = "macos")]
const KRUN_INPUT_CONFIG_FEATURE_QUERY: u64 = 1;
#[cfg(target_os = "macos")]
const KRUN_INPUT_EVENT_PROVIDER_FEATURE_QUEUE: u64 = 1;

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KrunInputEvent {
    pub r#type: u16,
    pub code: u16,
    pub value: u32,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KrunInputDeviceIds {
    pub bustype: u16,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KrunInputAbsinfo {
    pub min: u32,
    pub max: u32,
    pub fuzz: u32,
    pub flat: u32,
    pub res: u32,
}

#[cfg(target_os = "macos")]
pub type KrunInputCreateFn = Option<
    unsafe extern "C" fn(
        instance: *mut *mut c_void,
        userdata: *const c_void,
        reserved: *const c_void,
    ) -> i32,
>;
#[cfg(target_os = "macos")]
pub type KrunInputDestroyFn = Option<unsafe extern "C" fn(instance: *mut c_void) -> i32>;
#[cfg(target_os = "macos")]
pub type KrunInputQueryDeviceNameFn =
    Option<unsafe extern "C" fn(instance: *mut c_void, name_buf: *mut u8, name_buf_len: usize) -> i32>;
#[cfg(target_os = "macos")]
pub type KrunInputQuerySerialNameFn =
    Option<unsafe extern "C" fn(instance: *mut c_void, name_buf: *mut u8, name_buf_len: usize) -> i32>;
#[cfg(target_os = "macos")]
pub type KrunInputQueryDeviceIdsFn =
    Option<unsafe extern "C" fn(instance: *mut c_void, ids: *mut KrunInputDeviceIds) -> i32>;
#[cfg(target_os = "macos")]
pub type KrunInputQueryEventCapabilitiesFn = Option<
    unsafe extern "C" fn(
        instance: *mut c_void,
        ev_type: u8,
        bitmap: *mut u8,
        bitmap_len: usize,
    ) -> i32,
>;
#[cfg(target_os = "macos")]
pub type KrunInputQueryAbsInfoFn = Option<
    unsafe extern "C" fn(instance: *mut c_void, code: u8, absinfo: *mut KrunInputAbsinfo) -> i32,
>;
#[cfg(target_os = "macos")]
pub type KrunInputQueryPropertiesFn =
    Option<unsafe extern "C" fn(instance: *mut c_void, bitmap: *mut u8, bitmap_len: usize) -> i32>;

#[cfg(target_os = "macos")]
pub type KrunInputGetReadyEfdFn = Option<unsafe extern "C" fn(instance: *mut c_void) -> i32>;
#[cfg(target_os = "macos")]
pub type KrunInputNextEventFn =
    Option<unsafe extern "C" fn(instance: *mut c_void, event: *mut KrunInputEvent) -> i32>;

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct KrunInputConfigVtable {
    pub destroy: KrunInputDestroyFn,
    pub query_device_name: KrunInputQueryDeviceNameFn,
    pub query_serial_name: KrunInputQuerySerialNameFn,
    pub query_device_ids: KrunInputQueryDeviceIdsFn,
    pub query_event_capabilities: KrunInputQueryEventCapabilitiesFn,
    pub query_abs_info: KrunInputQueryAbsInfoFn,
    pub query_properties: KrunInputQueryPropertiesFn,
}

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct KrunInputConfig {
    pub features: u64,
    pub create_userdata: *mut c_void,
    pub create: KrunInputCreateFn,
    pub vtable: KrunInputConfigVtable,
}

#[cfg(target_os = "macos")]
unsafe impl Send for KrunInputConfig {}
#[cfg(target_os = "macos")]
unsafe impl Sync for KrunInputConfig {}

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct KrunInputEventProviderVtable {
    pub destroy: KrunInputDestroyFn,
    pub get_ready_efd: KrunInputGetReadyEfdFn,
    pub next_event: KrunInputNextEventFn,
}

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct KrunInputEventProvider {
    pub features: u64,
    pub create_userdata: *mut c_void,
    pub create: KrunInputCreateFn,
    pub vtable: KrunInputEventProviderVtable,
}

#[cfg(target_os = "macos")]
unsafe impl Send for KrunInputEventProvider {}
#[cfg(target_os = "macos")]
unsafe impl Sync for KrunInputEventProvider {}

#[cfg(target_os = "macos")]
struct DeviceQueue {
    events: Mutex<VecDeque<KrunInputEvent>>,
    pipe: [libc::c_int; 2],
}

#[cfg(target_os = "macos")]
impl DeviceQueue {
    fn new() -> Self {
        let mut fds = [0i32; 2];
        unsafe {
            if libc::pipe(fds.as_mut_ptr()) != 0 {
                panic!("pipe() failed: {}", std::io::Error::last_os_error());
            }
            libc::fcntl(fds[0], libc::F_SETFL, libc::O_NONBLOCK);
            libc::fcntl(fds[1], libc::F_SETFL, libc::O_NONBLOCK);
        }
        DeviceQueue {
            events: Mutex::new(VecDeque::new()),
            pipe: fds,
        }
    }

    fn push_batch(&self, events: &[KrunInputEvent]) {
        if let Ok(mut q) = self.events.lock() {
            q.extend(events.iter().copied());
            unsafe {
                libc::write(self.pipe[1], &1u8 as *const u8 as *const _, 1);
            }
        }
    }

    fn pop(&self) -> Option<KrunInputEvent> {
        let mut q = self.events.lock().ok()?;
        let event = q.pop_front();
        if event.is_none() {
            let mut buf = [0u8; 64];
            loop {
                let n = unsafe { libc::read(self.pipe[0], buf.as_mut_ptr() as *mut _, buf.len()) };
                if n <= 0 { break; }
            }
        }
        event
    }

    fn read_fd(&self) -> i32 {
        self.pipe[0]
    }
}

#[cfg(target_os = "macos")]
static KEYBOARD_QUEUE: OnceLock<DeviceQueue> = OnceLock::new();
#[cfg(target_os = "macos")]
static MOUSE_QUEUE: OnceLock<DeviceQueue> = OnceLock::new();

#[cfg(target_os = "macos")]
fn keyboard_queue() -> &'static DeviceQueue {
    KEYBOARD_QUEUE.get_or_init(DeviceQueue::new)
}

#[cfg(target_os = "macos")]
fn mouse_queue() -> &'static DeviceQueue {
    MOUSE_QUEUE.get_or_init(DeviceQueue::new)
}

#[cfg(target_os = "macos")]
unsafe fn write_bitmap(bitmap: *mut u8, bitmap_len: usize, n: u16) {
    let byte_idx = (n / 8) as usize;
    let bit_idx = n % 8;
    if byte_idx < bitmap_len {
        unsafe {
            let ptr = bitmap.add(byte_idx);
            *ptr |= 1 << bit_idx;
        }
    }
}

#[cfg(target_os = "macos")]
const SUPPORTED_KEYBOARD_KEYS: &[u16] = &[
    // KEY_ESC(1) .. KEY_EQUAL(13)
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
    // KEY_BACKSPACE(14) .. KEY_TAB(15) .. KEY_P(25)
    14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    // KEY_LEFTBRACE(26) .. KEY_ENTER(28)
    26, 27, 28,
    // KEY_LEFTCTRL(29) .. KEY_SEMICOLON(39)
    29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    // KEY_GRAVE(41) .. KEY_LEFTSHIFT(42) .. KEY_BACKSLASH(43)
    40, 41, 42, 43,
    // KEY_Z(44) .. KEY_SLASH(53)
    44, 45, 46, 47, 48, 49, 50, 51, 52, 53,
    // KEY_RIGHTSHIFT(54) .. KEY_LEFTALT(56) .. KEY_SPACE(57) .. KEY_CAPSLOCK(58)
    54, 56, 57, 58,
    // KEY_F1(59) .. KEY_F10(68)
    59, 60, 61, 62, 63, 64, 65, 66, 67, 68,
    // KEY_SCROLLLOCK(70)
    70,
    // KEY_F11(87) .. KEY_F12(88)
    87, 88,
    // KEY_RIGHTCTRL(97) .. KEY_RIGHTALT(100)
    97, 100,
    // KEY_HOME(102) .. KEY_UP(103) .. KEY_PAGEUP(104) .. KEY_LEFT(105)
    102, 103, 104, 105,
    // KEY_RIGHT(106) .. KEY_END(107) .. KEY_DOWN(108) .. KEY_PAGEDOWN(109)
    106, 107, 108, 109,
    // KEY_INSERT(110) .. KEY_DELETE(111)
    110, 111,
    // KEY_MUTE(113), KEY_VOLUMEDOWN(114), KEY_VOLUMEUP(115), KEY_POWER(116)
    113, 114, 115, 116,
    // KEY_PAUSE(119)
    119,
    // KEY_LEFTMETA(125) — Command key
    125,
    // KEY_F13(183)..KEY_F16 — actually using codes from mapping: 210, 70, 110
    210,
];


#[cfg(target_os = "macos")]
static KEYBOARD_NAME: &[u8] = b"AgentOS Virtual Keyboard";
#[cfg(target_os = "macos")]
static KEYBOARD_SERIAL: &[u8] = b"agentos-kbd-0";

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_config_create(
    instance: *mut *mut c_void,
    _userdata: *const c_void,
    _reserved: *const c_void,
) -> i32 {
    // We use global state — no per-instance data needed
    unsafe { *instance = std::ptr::null_mut() };
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_config_destroy(_instance: *mut c_void) -> i32 {
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_query_device_name(
    _instance: *mut c_void,
    name_buf: *mut u8,
    name_buf_len: usize,
) -> i32 {
    let copy_len = KEYBOARD_NAME.len().min(name_buf_len);
    unsafe { std::ptr::copy_nonoverlapping(KEYBOARD_NAME.as_ptr(), name_buf, copy_len) };
    copy_len as i32
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_query_serial_name(
    _instance: *mut c_void,
    name_buf: *mut u8,
    name_buf_len: usize,
) -> i32 {
    let copy_len = KEYBOARD_SERIAL.len().min(name_buf_len);
    unsafe { std::ptr::copy_nonoverlapping(KEYBOARD_SERIAL.as_ptr(), name_buf, copy_len) };
    copy_len as i32
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_query_device_ids(
    _instance: *mut c_void,
    ids: *mut KrunInputDeviceIds,
) -> i32 {
    unsafe {
        *ids = KrunInputDeviceIds {
            bustype: BUS_VIRTUAL,
            vendor: 0x0001,
            product: 0x0001,
            version: 0x0001,
        };
    }
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_query_event_capabilities(
    _instance: *mut c_void,
    ev_type: u8,
    bitmap: *mut u8,
    bitmap_len: usize,
) -> i32 {
    let max_bit: u16 = match ev_type as u16 {
        EV_SYN => {
            unsafe { write_bitmap(bitmap, bitmap_len, SYN_REPORT) };
            SYN_REPORT
        }
        EV_KEY => {
            let mut max = 0u16;
            for &key in SUPPORTED_KEYBOARD_KEYS {
                unsafe { write_bitmap(bitmap, bitmap_len, key) };
                if key > max { max = key; }
            }
            max
        }
        _ => return 0,
    };
    (max_bit / 8 + 1) as i32
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_query_abs_info(
    _instance: *mut c_void,
    _code: u8,
    _absinfo: *mut KrunInputAbsinfo,
) -> i32 {
    -1
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_query_properties(
    _instance: *mut c_void,
    _bitmap: *mut u8,
    _bitmap_len: usize,
) -> i32 {
    0
}


#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_event_create(
    instance: *mut *mut c_void,
    _userdata: *const c_void,
    _reserved: *const c_void,
) -> i32 {
    keyboard_queue();
    unsafe { *instance = std::ptr::null_mut() };
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_event_destroy(_instance: *mut c_void) -> i32 {
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_get_ready_efd(_instance: *mut c_void) -> i32 {
    keyboard_queue().read_fd()
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn kbd_next_event(
    _instance: *mut c_void,
    event: *mut KrunInputEvent,
) -> i32 {
    match keyboard_queue().pop() {
        Some(e) => {
            unsafe { *event = e };
            1
        }
        None => 0,
    }
}


#[cfg(target_os = "macos")]
static MOUSE_NAME: &[u8] = b"AgentOS Virtual Tablet";
#[cfg(target_os = "macos")]
static MOUSE_SERIAL: &[u8] = b"agentos-tablet-0";

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_config_create(
    instance: *mut *mut c_void,
    _userdata: *const c_void,
    _reserved: *const c_void,
) -> i32 {
    unsafe { *instance = std::ptr::null_mut() };
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_config_destroy(_instance: *mut c_void) -> i32 {
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_query_device_name(
    _instance: *mut c_void,
    name_buf: *mut u8,
    name_buf_len: usize,
) -> i32 {
    let copy_len = MOUSE_NAME.len().min(name_buf_len);
    unsafe { std::ptr::copy_nonoverlapping(MOUSE_NAME.as_ptr(), name_buf, copy_len) };
    copy_len as i32
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_query_serial_name(
    _instance: *mut c_void,
    name_buf: *mut u8,
    name_buf_len: usize,
) -> i32 {
    let copy_len = MOUSE_SERIAL.len().min(name_buf_len);
    unsafe { std::ptr::copy_nonoverlapping(MOUSE_SERIAL.as_ptr(), name_buf, copy_len) };
    copy_len as i32
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_query_device_ids(
    _instance: *mut c_void,
    ids: *mut KrunInputDeviceIds,
) -> i32 {
    unsafe {
        *ids = KrunInputDeviceIds {
            bustype: BUS_VIRTUAL,
            vendor: 0x0001,
            product: 0x0002,
            version: 0x0001,
        };
    }
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_query_event_capabilities(
    _instance: *mut c_void,
    ev_type: u8,
    bitmap: *mut u8,
    bitmap_len: usize,
) -> i32 {
    let max_bit: u16 = match ev_type as u16 {
        EV_SYN => {
            unsafe { write_bitmap(bitmap, bitmap_len, SYN_REPORT) };
            SYN_REPORT
        }
        EV_KEY => {
            unsafe {
                write_bitmap(bitmap, bitmap_len, BTN_LEFT);
                write_bitmap(bitmap, bitmap_len, BTN_RIGHT);
                write_bitmap(bitmap, bitmap_len, BTN_MIDDLE);
            }
            BTN_MIDDLE
        }
        EV_ABS => {
            unsafe {
                write_bitmap(bitmap, bitmap_len, ABS_X);
                write_bitmap(bitmap, bitmap_len, ABS_Y);
            }
            ABS_Y
        }
        EV_REL => {
            unsafe {
                write_bitmap(bitmap, bitmap_len, REL_WHEEL);
                write_bitmap(bitmap, bitmap_len, REL_HWHEEL);
            }
            REL_HWHEEL
        }
        _ => return 0,
    };
    (max_bit / 8 + 1) as i32
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_query_abs_info(
    _instance: *mut c_void,
    code: u8,
    absinfo: *mut KrunInputAbsinfo,
) -> i32 {
    match code as u16 {
        ABS_X | ABS_Y => {
            unsafe {
                *absinfo = KrunInputAbsinfo {
                    min: 0,
                    max: ABS_MAX,
                    fuzz: 0,
                    flat: 0,
                    res: 0,
                };
            }
            0
        }
        _ => 0,
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_query_properties(
    _instance: *mut c_void,
    bitmap: *mut u8,
    bitmap_len: usize,
) -> i32 {
    unsafe { write_bitmap(bitmap, bitmap_len, INPUT_PROP_DIRECT) };
    (INPUT_PROP_DIRECT / 8 + 1) as i32
}


#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_event_create(
    instance: *mut *mut c_void,
    _userdata: *const c_void,
    _reserved: *const c_void,
) -> i32 {
    mouse_queue();
    unsafe { *instance = std::ptr::null_mut() };
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_event_destroy(_instance: *mut c_void) -> i32 {
    0
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_get_ready_efd(_instance: *mut c_void) -> i32 {
    mouse_queue().read_fd()
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn mouse_next_event(
    _instance: *mut c_void,
    event: *mut KrunInputEvent,
) -> i32 {
    match mouse_queue().pop() {
        Some(e) => {
            unsafe { *event = e };
            1
        }
        None => 0,
    }
}


#[cfg(target_os = "macos")]
pub fn create_keyboard_backend() -> (KrunInputConfig, KrunInputEventProvider) {
    let config = KrunInputConfig {
        features: KRUN_INPUT_CONFIG_FEATURE_QUERY,
        create_userdata: std::ptr::null_mut(),
        create: Some(kbd_config_create),
        vtable: KrunInputConfigVtable {
            destroy: Some(kbd_config_destroy),
            query_device_name: Some(kbd_query_device_name),
            query_serial_name: Some(kbd_query_serial_name),
            query_device_ids: Some(kbd_query_device_ids),
            query_event_capabilities: Some(kbd_query_event_capabilities),
            query_abs_info: Some(kbd_query_abs_info),
            query_properties: Some(kbd_query_properties),
        },
    };

    let provider = KrunInputEventProvider {
        features: KRUN_INPUT_EVENT_PROVIDER_FEATURE_QUEUE,
        create_userdata: std::ptr::null_mut(),
        create: Some(kbd_event_create),
        vtable: KrunInputEventProviderVtable {
            destroy: Some(kbd_event_destroy),
            get_ready_efd: Some(kbd_get_ready_efd),
            next_event: Some(kbd_next_event),
        },
    };

    (config, provider)
}

#[cfg(target_os = "macos")]
pub fn create_mouse_backend() -> (KrunInputConfig, KrunInputEventProvider) {
    let config = KrunInputConfig {
        features: KRUN_INPUT_CONFIG_FEATURE_QUERY,
        create_userdata: std::ptr::null_mut(),
        create: Some(mouse_config_create),
        vtable: KrunInputConfigVtable {
            destroy: Some(mouse_config_destroy),
            query_device_name: Some(mouse_query_device_name),
            query_serial_name: Some(mouse_query_serial_name),
            query_device_ids: Some(mouse_query_device_ids),
            query_event_capabilities: Some(mouse_query_event_capabilities),
            query_abs_info: Some(mouse_query_abs_info),
            query_properties: Some(mouse_query_properties),
        },
    };

    let provider = KrunInputEventProvider {
        features: KRUN_INPUT_EVENT_PROVIDER_FEATURE_QUEUE,
        create_userdata: std::ptr::null_mut(),
        create: Some(mouse_event_create),
        vtable: KrunInputEventProviderVtable {
            destroy: Some(mouse_event_destroy),
            get_ready_efd: Some(mouse_get_ready_efd),
            next_event: Some(mouse_next_event),
        },
    };

    (config, provider)
}


// value: 0 = release, 1 = press, 2 = repeat
#[cfg(target_os = "macos")]
pub fn send_key_event(code: u16, value: u32) {
    keyboard_queue().push_batch(&[
        KrunInputEvent {
            r#type: EV_KEY,
            code,
            value,
        },
        KrunInputEvent {
            r#type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        },
    ]);
}

// x, y in range 0..=32767
#[cfg(target_os = "macos")]
pub fn send_mouse_move_abs(x: u32, y: u32) {
    mouse_queue().push_batch(&[
        KrunInputEvent {
            r#type: EV_ABS,
            code: ABS_X,
            value: x,
        },
        KrunInputEvent {
            r#type: EV_ABS,
            code: ABS_Y,
            value: y,
        },
        KrunInputEvent {
            r#type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        },
    ]);
}

#[cfg(target_os = "macos")]
pub fn send_mouse_button(button: u16, pressed: bool) {
    mouse_queue().push_batch(&[
        KrunInputEvent {
            r#type: EV_KEY,
            code: button,
            value: if pressed { 1 } else { 0 },
        },
        KrunInputEvent {
            r#type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        },
    ]);
}

#[cfg(target_os = "macos")]
pub fn send_mouse_scroll(dx: i32, dy: i32) {
    let mut batch = [KrunInputEvent { r#type: 0, code: 0, value: 0 }; 3];
    let mut n = 0;
    if dy != 0 {
        batch[n] = KrunInputEvent { r#type: EV_REL, code: REL_WHEEL, value: dy as u32 };
        n += 1;
    }
    if dx != 0 {
        batch[n] = KrunInputEvent { r#type: EV_REL, code: REL_HWHEEL, value: dx as u32 };
        n += 1;
    }
    batch[n] = KrunInputEvent { r#type: EV_SYN, code: SYN_REPORT, value: 0 };
    n += 1;
    mouse_queue().push_batch(&batch[..n]);
}


#[cfg(target_os = "macos")]
pub fn macos_keycode_to_linux(keycode: u16) -> u16 {
    match keycode {
        // Letters
        0 => 30,   // KEY_A
        1 => 31,   // KEY_S
        2 => 32,   // KEY_D
        3 => 33,   // KEY_F
        4 => 35,   // KEY_H
        5 => 34,   // KEY_G
        6 => 44,   // KEY_Z
        7 => 45,   // KEY_X
        8 => 46,   // KEY_C
        9 => 47,   // KEY_V
        11 => 48,  // KEY_B
        12 => 16,  // KEY_Q
        13 => 17,  // KEY_W
        14 => 18,  // KEY_E
        15 => 19,  // KEY_R
        16 => 21,  // KEY_Y
        17 => 20,  // KEY_T

        // Numbers
        18 => 2,   // KEY_1
        19 => 3,   // KEY_2
        20 => 4,   // KEY_3
        21 => 5,   // KEY_4
        22 => 7,   // KEY_6
        23 => 6,   // KEY_5
        24 => 13,  // KEY_EQUAL
        25 => 10,  // KEY_9
        26 => 8,   // KEY_7
        27 => 12,  // KEY_MINUS
        28 => 9,   // KEY_8
        29 => 11,  // KEY_0

        // Punctuation / brackets
        30 => 27,  // KEY_RIGHTBRACE
        31 => 24,  // KEY_O
        32 => 22,  // KEY_U
        33 => 26,  // KEY_LEFTBRACE
        34 => 23,  // KEY_I
        35 => 25,  // KEY_P
        36 => 28,  // KEY_ENTER
        37 => 38,  // KEY_L
        38 => 36,  // KEY_J
        39 => 40,  // KEY_APOSTROPHE
        40 => 37,  // KEY_K
        41 => 39,  // KEY_SEMICOLON
        42 => 43,  // KEY_BACKSLASH
        43 => 51,  // KEY_COMMA
        44 => 53,  // KEY_SLASH
        45 => 49,  // KEY_N
        46 => 50,  // KEY_M
        47 => 52,  // KEY_DOT

        // Whitespace / editing
        48 => 15,  // KEY_TAB
        49 => 57,  // KEY_SPACE
        50 => 41,  // KEY_GRAVE
        51 => 14,  // KEY_BACKSPACE

        // Escape
        53 => 1,   // KEY_ESC

        // Modifiers
        56 => 42,  // KEY_LEFTSHIFT
        57 => 58,  // KEY_CAPSLOCK
        58 => 56,  // KEY_LEFTALT
        59 => 29,  // KEY_LEFTCTRL
        60 => 54,  // KEY_RIGHTSHIFT
        61 => 100, // KEY_RIGHTALT
        62 => 97,  // KEY_RIGHTCTRL

        // Function keys
        96 => 63,  // KEY_F5
        97 => 64,  // KEY_F6
        98 => 65,  // KEY_F7
        99 => 61,  // KEY_F3
        100 => 66, // KEY_F8
        101 => 67, // KEY_F9
        103 => 87, // KEY_F11
        105 => 210, // KEY_F13 (mapped to KEY_PRINT area)
        107 => 70, // KEY_F14 (mapped to KEY_SCROLLLOCK)
        109 => 68, // KEY_F10
        111 => 88, // KEY_F12
        113 => 110, // KEY_F15 (mapped to KEY_INSERT)
        114 => 102, // KEY_HOME (macOS Help key area)
        115 => 102, // KEY_HOME
        116 => 104, // KEY_PAGEUP
        117 => 111, // KEY_DELETE (forward delete)
        118 => 62, // KEY_F4
        119 => 107, // KEY_END
        120 => 60, // KEY_F2
        121 => 109, // KEY_PAGEDOWN
        122 => 59, // KEY_F1

        // Arrow keys
        123 => 105, // KEY_LEFT
        124 => 106, // KEY_RIGHT
        125 => 108, // KEY_DOWN
        126 => 103, // KEY_UP

        _ => 0, // unmapped
    }
}


#[cfg(target_os = "macos")]
pub fn macos_mouse_button_to_linux(button: u16) -> u16 {
    match button {
        0 => BTN_LEFT,
        1 => BTN_RIGHT,
        2 => BTN_MIDDLE,
        _ => BTN_LEFT,
    }
}

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[cfg(target_os = "macos")]
static LAST_MODIFIER_FLAGS: AtomicUsize = AtomicUsize::new(0);

#[cfg(target_os = "macos")]
struct ModifierMapping {
    flag: usize,
    linux_code: u16,
}

#[cfg(target_os = "macos")]
const MODIFIER_MAP: &[ModifierMapping] = &[
    ModifierMapping { flag: 1 << 17, linux_code: 42 },  // Shift → KEY_LEFTSHIFT
    ModifierMapping { flag: 1 << 18, linux_code: 29 },  // Control → KEY_LEFTCTRL
    ModifierMapping { flag: 1 << 19, linux_code: 56 },  // Option → KEY_LEFTALT
    ModifierMapping { flag: 1 << 20, linux_code: 125 }, // Command → KEY_LEFTMETA
];

#[cfg(target_os = "macos")]
pub fn sync_modifiers(new_flags: objc2_app_kit::NSEventModifierFlags) {
    let new_raw = new_flags.bits();
    let old_raw = LAST_MODIFIER_FLAGS.swap(new_raw, AtomicOrdering::SeqCst);
    for m in MODIFIER_MAP {
        let was = old_raw & m.flag != 0;
        let is = new_raw & m.flag != 0;
        if was && !is {
            send_key_event(m.linux_code, 0);
        } else if !was && is {
            send_key_event(m.linux_code, 1);
        }
    }
}

#[cfg(target_os = "macos")]
pub fn release_all_modifiers() {
    LAST_MODIFIER_FLAGS.store(0, AtomicOrdering::SeqCst);
    for m in MODIFIER_MAP {
        send_key_event(m.linux_code, 0);
    }
}

#[cfg(target_os = "macos")]
pub fn send_capslock_toggle() {
    keyboard_queue().push_batch(&[
        KrunInputEvent { r#type: EV_KEY, code: 58, value: 1 },
        KrunInputEvent { r#type: EV_SYN, code: SYN_REPORT, value: 0 },
        KrunInputEvent { r#type: EV_KEY, code: 58, value: 0 },
        KrunInputEvent { r#type: EV_SYN, code: SYN_REPORT, value: 0 },
    ]);
}
