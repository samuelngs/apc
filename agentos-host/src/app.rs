#[cfg(target_os = "macos")]
use crate::vm::{self, VmConfig};

#[cfg(target_os = "macos")]
use anyhow::Result;

#[cfg(target_os = "macos")]
use objc2::{define_class, msg_send, rc::Retained, runtime::NSObject, DefinedClass, MainThreadMarker, MainThreadOnly};

#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType,
    NSEvent, NSWindow, NSWindowStyleMask,
};

#[cfg(target_os = "macos")]
use objc2_foundation::{NSNotification, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};

#[cfg(target_os = "macos")]
use std::cell::OnceCell;

#[cfg(target_os = "macos")]
use std::sync::Mutex;

#[cfg(target_os = "macos")]
use crate::display;

#[cfg(target_os = "macos")]
fn ca_transaction_begin() {
    unsafe {
        let cls: &objc2::runtime::AnyClass = objc2::runtime::AnyClass::get(c"CATransaction").unwrap();
        let _: () = msg_send![cls, begin];
    }
}

#[cfg(target_os = "macos")]
fn ca_transaction_set_disable_actions(disable: bool) {
    unsafe {
        let cls: &objc2::runtime::AnyClass = objc2::runtime::AnyClass::get(c"CATransaction").unwrap();
        let _: () = msg_send![cls, setDisableActions: disable];
    }
}

#[cfg(target_os = "macos")]
fn ca_transaction_commit() {
    unsafe {
        let cls: &objc2::runtime::AnyClass = objc2::runtime::AnyClass::get(c"CATransaction").unwrap();
        let _: () = msg_send![cls, commit];
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn dispatch_async_f(
        queue: *mut std::ffi::c_void,
        context: *mut std::ffi::c_void,
        work: unsafe extern "C" fn(*mut std::ffi::c_void),
    );

    fn dispatch_source_create(
        source_type: *const std::ffi::c_void,
        handle: usize,
        mask: usize,
        queue: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;

    fn dispatch_source_set_timer(
        source: *mut std::ffi::c_void,
        start: u64,
        interval: u64,
        leeway: u64,
    );

    fn dispatch_source_set_event_handler_f(
        source: *mut std::ffi::c_void,
        handler: unsafe extern "C" fn(*mut std::ffi::c_void),
    );

    fn dispatch_set_context(
        object: *mut std::ffi::c_void,
        context: *mut std::ffi::c_void,
    );

    fn dispatch_resume(object: *mut std::ffi::c_void);

    fn dispatch_walltime(when: *const std::ffi::c_void, delta: i64) -> u64;

    #[link_name = "_dispatch_main_q"]
    static DISPATCH_MAIN_Q: std::ffi::c_void;

    #[link_name = "_dispatch_source_type_timer"]
    static DISPATCH_SOURCE_TYPE_TIMER: std::ffi::c_void;
}

#[cfg(target_os = "macos")]
const NSEC_PER_MSEC: u64 = 1_000_000;


#[cfg(target_os = "macos")]
static LAST_DISPLAYED_SURFACE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(target_os = "macos")]
static PRESSED_KEYS: Mutex<Option<std::collections::HashSet<u16>>> = Mutex::new(None);

#[cfg(target_os = "macos")]
fn track_key_press(code: u16) -> u32 {
    let mut guard = PRESSED_KEYS.lock().unwrap();
    let set = guard.get_or_insert_with(std::collections::HashSet::new);
    if set.insert(code) { 1 } else { 2 }
}

#[cfg(target_os = "macos")]
fn track_key_release(code: u16) {
    if let Ok(mut guard) = PRESSED_KEYS.lock() {
        if let Some(set) = guard.as_mut() {
            set.remove(&code);
        }
    }
}

#[cfg(target_os = "macos")]
pub struct FramebufferViewIvars;

#[cfg(target_os = "macos")]
define_class!(
    #[unsafe(super(objc2_app_kit::NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "FramebufferView"]
    #[ivars = FramebufferViewIvars]
    pub struct FramebufferView;

    unsafe impl NSObjectProtocol for FramebufferView {}

    #[allow(non_snake_case)]
    impl FramebufferView {
        #[unsafe(method(acceptsFirstResponder))]
        fn acceptsFirstResponder(&self) -> bool { true }

        #[unsafe(method(keyDown:))]
        fn keyDown(&self, event: &NSEvent) {
            let code = crate::input::macos_keycode_to_linux(event.keyCode());
            if code != 0 {
                let value = track_key_press(code);
                crate::input::send_key_event(code, value);
            }
        }

        #[unsafe(method(keyUp:))]
        fn keyUp(&self, event: &NSEvent) {
            let code = crate::input::macos_keycode_to_linux(event.keyCode());
            if code != 0 {
                track_key_release(code);
                crate::input::send_key_event(code, 0);
            }
        }

        #[unsafe(method(flagsChanged:))]
        fn flagsChanged(&self, event: &NSEvent) {
            let keycode = event.keyCode();
            if keycode == 57 {
                crate::input::send_capslock_toggle();
                return;
            }
            crate::input::sync_modifiers(event.modifierFlags());
        }

        #[unsafe(method(resignFirstResponder))]
        fn resignFirstResponder_(&self) -> bool {
            crate::input::release_all_modifiers();
            true
        }

        #[unsafe(method(mouseDown:))]
        fn mouseDown(&self, _event: &NSEvent) {
            crate::input::send_mouse_button(crate::input::macos_mouse_button_to_linux(0), true);
        }

        #[unsafe(method(mouseUp:))]
        fn mouseUp(&self, _event: &NSEvent) {
            crate::input::send_mouse_button(crate::input::macos_mouse_button_to_linux(0), false);
        }

        #[unsafe(method(rightMouseDown:))]
        fn rightMouseDown(&self, _event: &NSEvent) {
            crate::input::send_mouse_button(crate::input::macos_mouse_button_to_linux(1), true);
        }

        #[unsafe(method(rightMouseUp:))]
        fn rightMouseUp(&self, _event: &NSEvent) {
            crate::input::send_mouse_button(crate::input::macos_mouse_button_to_linux(1), false);
        }

        #[unsafe(method(otherMouseDown:))]
        fn otherMouseDown(&self, event: &NSEvent) {
            let btn = crate::input::macos_mouse_button_to_linux(event.buttonNumber() as u16);
            crate::input::send_mouse_button(btn, true);
        }

        #[unsafe(method(otherMouseUp:))]
        fn otherMouseUp(&self, event: &NSEvent) {
            let btn = crate::input::macos_mouse_button_to_linux(event.buttonNumber() as u16);
            crate::input::send_mouse_button(btn, false);
        }

        #[unsafe(method(mouseMoved:))]
        fn mouseMoved(&self, event: &NSEvent) {
            self.send_abs_position(event);
        }

        #[unsafe(method(mouseDragged:))]
        fn mouseDragged(&self, event: &NSEvent) {
            self.send_abs_position(event);
        }

        #[unsafe(method(rightMouseDragged:))]
        fn rightMouseDragged(&self, event: &NSEvent) {
            self.send_abs_position(event);
        }

        #[unsafe(method(scrollWheel:))]
        fn scrollWheel(&self, event: &NSEvent) {
            let dy = event.scrollingDeltaY() as i32;
            let dx = event.scrollingDeltaX() as i32;
            if dx != 0 || dy != 0 { crate::input::send_mouse_scroll(dx, dy); }
        }
    }
);

#[cfg(target_os = "macos")]
impl FramebufferView {
    fn send_abs_position(&self, event: &NSEvent) {
        let loc = event.locationInWindow();
        let local = self.convertPoint_fromView(loc, None);
        let bounds = self.bounds();
        if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
            return;
        }
        let nx = (local.x / bounds.size.width).clamp(0.0, 1.0);
        // NSView Y is bottom-up, flip to top-down
        let ny = (1.0 - local.y / bounds.size.height).clamp(0.0, 1.0);
        let abs_x = (nx * 32767.0) as u32;
        let abs_y = (ny * 32767.0) as u32;
        crate::input::send_mouse_move_abs(abs_x, abs_y);
    }

    fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        let this = mtm.alloc::<Self>();
        let this = this.set_ivars(FramebufferViewIvars);
        let view: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
        view.setWantsLayer(true);
        if let Some(layer) = view.layer() {
            layer.setContentsScale(1.0);
            layer.setContentsGravity(unsafe { objc2_quartz_core::kCAGravityResize });
            layer.setOpaque(true);
        }
        unsafe {
            use objc2_app_kit::NSTrackingAreaOptions;
            let options = NSTrackingAreaOptions::MouseMoved
                | NSTrackingAreaOptions::MouseEnteredAndExited
                | NSTrackingAreaOptions::ActiveAlways
                | NSTrackingAreaOptions::InVisibleRect;
            let tracking_area = objc2_app_kit::NSTrackingArea::initWithRect_options_owner_userInfo(
                mtm.alloc::<objc2_app_kit::NSTrackingArea>(),
                NSRect::ZERO,
                options,
                Some(&view),
                None,
            );
            view.addTrackingArea(&tracking_area);
        }
        view
    }

    pub fn update_framebuffer(&self) {
        let state = display::global_display();
        let Some(surface) = state.get_front_surface() else {
            return;
        };
        let surface_usize = surface as usize;
        let prev = LAST_DISPLAYED_SURFACE.swap(surface_usize, std::sync::atomic::Ordering::Relaxed);
        if surface_usize == prev {
            return;
        }
        if let Some(layer) = self.layer() {
            ca_transaction_begin();
            ca_transaction_set_disable_actions(true);
            unsafe {
                let contents = &*(surface as *const objc2::runtime::AnyObject);
                layer.setContents(Some(contents));
            }
            ca_transaction_commit();
        }
    }
}


#[cfg(target_os = "macos")]
struct AppDelegateIvars {
    config: OnceCell<VmConfig>,
    window: OnceCell<Retained<NSWindow>>,
}

#[cfg(target_os = "macos")]
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "AgentOSAppDelegate"]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _notification: &NSNotification) {
            self.setup_vm_and_viewer();
        }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        fn should_terminate_after_last_window_closed(
            &self,
            _sender: &NSApplication,
        ) -> bool {
            true
        }
    }
);

#[cfg(target_os = "macos")]
impl AppDelegate {
    fn new(mtm: MainThreadMarker, config: VmConfig) -> Retained<Self> {
        let this = mtm.alloc::<Self>();
        let this = this.set_ivars(AppDelegateIvars {
            config: OnceCell::from(config),
            window: OnceCell::new(),
        });
        unsafe { msg_send![super(this), init] }
    }

    fn setup_vm_and_viewer(&self) {
        let config = self.ivars().config.get().expect("config not set");
        let mtm = MainThreadMarker::new().expect("not on main thread");

        let mcp_socket_path = format!("/tmp/agentos-mcp-{}.sock", std::process::id());

        let ctx = match vm::krun::configure_vm(config, &mcp_socket_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to configure VM: {e}");
                std::process::exit(1);
            }
        };

        self.create_viewer_window(mtm, config);
        vm::krun::start_vm(ctx);
        self.start_display_timer();

        if config.mcp_test {
            crate::mcp::run_mcp_test(&mcp_socket_path);
        }
    }

    fn create_viewer_window(&self, mtm: MainThreadMarker, config: &VmConfig) {
        let w = config.display_width as f64;
        let h = config.display_height as f64;

        let frame = NSRect::new(NSPoint::new(100.0, 100.0), NSSize::new(w, h));

        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Miniaturizable
            | NSWindowStyleMask::Resizable;

        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc::<NSWindow>(),
                frame,
                style,
                NSBackingStoreType::Buffered,
                false,
            )
        };

        let view = FramebufferView::new(mtm, frame);
        window.setContentView(Some(&view));
        window.setTitle(&NSString::from_str("AgentOS"));
        window.setAcceptsMouseMovedEvents(true);
        window.center();
        window.makeKeyAndOrderFront(None);
        window.makeFirstResponder(Some(&view));

        let _ = self.ivars().window.set(window);
    }

    fn start_display_timer(&self) {
        let window_ptr = self.ivars().window.get().unwrap() as *const Retained<NSWindow> as usize;

        unsafe extern "C" fn timer_handler(ctx: *mut std::ffi::c_void) {
            let window_ptr = ctx as usize;
            unsafe {
                let window = &*(window_ptr as *const Retained<NSWindow>);
                if let Some(view) = window.contentView() {
                    let fb_view: &FramebufferView =
                        &*((&*view) as *const objc2_app_kit::NSView as *const FramebufferView);
                    fb_view.update_framebuffer();
                }
            }
        }

        unsafe {
            let main_q = &raw const DISPATCH_MAIN_Q as *mut std::ffi::c_void;
            let timer_type = &raw const DISPATCH_SOURCE_TYPE_TIMER as *const std::ffi::c_void;
            let source = dispatch_source_create(timer_type, 0, 0, main_q);
            let interval_ns = 8 * NSEC_PER_MSEC; // ~120Hz polling
            let leeway_ns = 1 * NSEC_PER_MSEC;
            let start = dispatch_walltime(std::ptr::null(), 0);
            dispatch_source_set_timer(source, start, interval_ns, leeway_ns);
            dispatch_set_context(source, window_ptr as *mut std::ffi::c_void);
            dispatch_source_set_event_handler_f(source, timer_handler);
            dispatch_resume(source);
        }
    }
}


#[cfg(target_os = "macos")]
pub fn run(config: VmConfig) -> Result<()> {
    let mtm = MainThreadMarker::new().expect("must run on main thread");

    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    setup_menu(mtm, &app);

    let delegate = AppDelegate::new(mtm, config);
    app.setDelegate(Some(objc2::runtime::ProtocolObject::from_ref(&*delegate)));

    app.run();

    Ok(())
}

#[cfg(target_os = "macos")]
fn setup_menu(mtm: MainThreadMarker, app: &NSApplication) {
    use objc2_app_kit::{NSMenu, NSMenuItem};

    unsafe {
        let menubar = NSMenu::new(mtm);
        let app_menu_item = NSMenuItem::new(mtm);
        menubar.addItem(&app_menu_item);
        app.setMainMenu(Some(&menubar));

        let app_menu = NSMenu::new(mtm);
        let quit_item = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc::<NSMenuItem>(),
            &NSString::from_str("Quit AgentOS"),
            Some(objc2::sel!(terminate:)),
            &NSString::from_str("q"),
        );
        app_menu.addItem(&quit_item);
        app_menu_item.setSubmenu(Some(&app_menu));
    }
}
