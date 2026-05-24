#[cfg(target_os = "linux")]
use anyhow::Result;

#[cfg(target_os = "linux")]
use smithay::{
    backend::{
        allocator::gbm::GbmAllocator,
        drm::{
            compositor::{DrmCompositor, FrameFlags, PrimaryPlaneElement},
            exporter::gbm::GbmFramebufferExporter,
            DrmDeviceFd,
        },
        renderer::{
            damage::OutputDamageTracker,
            element::{
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                Kind,
            },
            gles::{GlesRenderer, GlesRenderbuffer},
            Bind, ExportMem, ImportAll, ImportMem, Offscreen,
        },
    },
    desktop::{space::space_render_elements, space::SpaceRenderElements},
    output::Mode as OutputMode,
    utils::{Buffer as BufferCoord, Physical, Rectangle, Scale, Size, Transform},
};

#[cfg(target_os = "linux")]
use drm_fourcc::DrmFourcc;

#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
use super::state::AgentCompositor;

#[cfg(target_os = "linux")]
use super::taskbar::get_window_title;

#[cfg(target_os = "linux")]
smithay::backend::renderer::element::render_elements! {
    pub(crate) OutputRenderElements<R, E> where R: ImportAll + ImportMem;
    Space=SpaceRenderElements<R, E>,
    Cursor=MemoryRenderBufferRenderElement<R>,
}

#[cfg(target_os = "linux")]
pub(crate) type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    (),
    DrmDeviceFd,
>;

#[cfg(target_os = "linux")]
pub(crate) const TASKBAR_HEIGHT: i32 = 36;
#[cfg(target_os = "linux")]
pub(crate) const TASKBAR_BTN_WIDTH: i32 = 140;
#[cfg(target_os = "linux")]
pub(crate) const TASKBAR_BTN_HEIGHT: i32 = 28;
#[cfg(target_os = "linux")]
pub(crate) const TASKBAR_BTN_GAP: i32 = 4;
#[cfg(target_os = "linux")]
pub(crate) const TASKBAR_BTN_MARGIN: i32 = 4;

#[cfg(target_os = "linux")]
#[derive(Debug, Default)]
pub(crate) enum RedrawState {
    #[default]
    Idle,
    Queued,
    WaitingForVBlank {
        redraw_needed: bool,
    },
}

#[cfg(target_os = "linux")]
pub(crate) fn create_solid_buffer(w: i32, h: i32, r: u8, g: u8, b: u8, a: u8) -> MemoryRenderBuffer {
    let data = vec![[r, g, b, a]; (w * h) as usize].into_iter().flatten().collect::<Vec<u8>>();
    MemoryRenderBuffer::from_slice(
        &data,
        DrmFourcc::Abgr8888,
        (w, h),
        1,
        Transform::Normal,
        None,
    )
}

#[cfg(target_os = "linux")]
pub(crate) fn queue_redraw(state: &mut AgentCompositor) {
    match &state.redraw_state {
        RedrawState::Idle => {
            state.redraw_state = RedrawState::Queued;
        }
        RedrawState::Queued => {}
        RedrawState::WaitingForVBlank { .. } => {
            state.redraw_state = RedrawState::WaitingForVBlank {
                redraw_needed: true,
            };
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn render_frame(state: &mut AgentCompositor) {
    let space_elements: Vec<SpaceRenderElements<GlesRenderer, _>> = space_render_elements(
        &mut state.renderer,
        [&state.space],
        &state.output,
        1.0,
    )
    .unwrap_or_default();

    let pointer_loc = state.pointer.current_location();
    let mut elements: Vec<OutputRenderElements<GlesRenderer, _>> = Vec::new();

    let cursor = match state.cursor_shape {
        super::input::CursorShape::Default => &state.cursor_default,
        super::input::CursorShape::ResizeNWSE => &state.cursor_resize_nwse,
        super::input::CursorShape::ResizeNESW => &state.cursor_resize_nesw,
        super::input::CursorShape::ResizeNS => &state.cursor_resize_ns,
        super::input::CursorShape::ResizeEW => &state.cursor_resize_ew,
    };
    let cursor_pos = (
        pointer_loc.x - cursor.hotspot.0 as f64,
        pointer_loc.y - cursor.hotspot.1 as f64,
    );
    if let Ok(cursor_elem) = MemoryRenderBufferRenderElement::from_buffer(
        &mut state.renderer,
        cursor_pos,
        &cursor.buffer,
        None,
        None,
        None,
        Kind::Cursor,
    ) {
        elements.push(OutputRenderElements::Cursor(cursor_elem));
    }

    let output_h = state.output.current_mode().map(|m| m.size.h).unwrap_or(1080);
    let taskbar_y = (output_h - TASKBAR_HEIGHT) as f64;

    let focused_surface = state.seat.get_keyboard().and_then(|kb| kb.current_focus());
    let mut new_buttons: Vec<(String, bool, bool, MemoryRenderBuffer)> = Vec::new();
    for window in state.space.elements() {
        let title = get_window_title(window);
        let is_focused = window
            .toplevel()
            .map(|t| focused_surface.as_ref() == Some(t.wl_surface()))
            .unwrap_or(false);
        let label = if title.is_empty() { "Window".to_string() } else { title };
        let (r, g, b) = if is_focused { (80, 80, 120) } else { (50, 50, 50) };
        let btn_buf = create_solid_buffer(TASKBAR_BTN_WIDTH, TASKBAR_BTN_HEIGHT, r, g, b, 255);
        new_buttons.push((label, is_focused, false, btn_buf));
    }
    for (window, _) in &state.minimized_windows {
        let title = get_window_title(window);
        let label = if title.is_empty() { "Window".to_string() } else { title };
        let btn_buf = create_solid_buffer(TASKBAR_BTN_WIDTH, TASKBAR_BTN_HEIGHT, 35, 35, 35, 255);
        new_buttons.push((label, false, true, btn_buf));
    }
    state.taskbar_buttons = new_buttons;

    for (i, (_, _, _, btn_buf)) in state.taskbar_buttons.iter().enumerate() {
        let x = (TASKBAR_BTN_MARGIN + i as i32 * (TASKBAR_BTN_WIDTH + TASKBAR_BTN_GAP)) as f64;
        let y = taskbar_y + ((TASKBAR_HEIGHT - TASKBAR_BTN_HEIGHT) / 2) as f64;
        if let Ok(btn) = MemoryRenderBufferRenderElement::from_buffer(
            &mut state.renderer, (x, y), btn_buf, None, None, None, Kind::Unspecified,
        ) {
            elements.push(OutputRenderElements::Cursor(btn));
        }
    }

    if let Ok(bg) = MemoryRenderBufferRenderElement::from_buffer(
        &mut state.renderer,
        (0.0, taskbar_y),
        &state.taskbar_bg,
        None,
        None,
        None,
        Kind::Unspecified,
    ) {
        elements.push(OutputRenderElements::Cursor(bg));
    }

    elements.extend(space_elements.into_iter().map(OutputRenderElements::Space));

    static FRAME_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let frame = FRAME_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let epsilon = if frame % 2 == 0 { 0.0f32 } else { 1.0 / 255.0 };
    let clear_color = [0.1, 0.1, 0.3 + epsilon, 1.0];

    match state
        .drm_compositor
        .render_frame(&mut state.renderer, &elements, clear_color, FrameFlags::empty())
    {
        Ok(result) => {
            if result.needs_sync() {
                if let PrimaryPlaneElement::Swapchain(ref element) = result.primary_element {
                    let _ = element.sync.wait();
                }
            }
            if !result.is_empty {
                match state.drm_compositor.queue_frame(()) {
                    Ok(()) => {
                        state.redraw_state = RedrawState::WaitingForVBlank {
                            redraw_needed: false,
                        };
                    }
                    Err(e) => {
                        tracing::error!("queue_frame failed: {e}");
                        state.redraw_state = RedrawState::Idle;
                    }
                }
            } else {
                state.redraw_state = RedrawState::Idle;
                let time = state.start_time.elapsed();
                let output = &state.output;
                state.space.elements().for_each(|window| {
                    window.send_frame(output, time, Some(Duration::ZERO), |_, _| {
                        Some(output.clone())
                    });
                });
            }
        }
        Err(e) => {
            tracing::error!("render failed: {e}");
            state.redraw_state = RedrawState::Idle;
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn capture_screen(state: &mut AgentCompositor) -> Result<(u32, u32, String)> {
    use base64::Engine;

    let output_mode = state.output.current_mode().unwrap_or(OutputMode {
        size: (1920, 1080).into(),
        refresh: 60000,
    });
    let w = output_mode.size.w;
    let h = output_mode.size.h;
    let buf_size: Size<i32, BufferCoord> = (w, h).into();

    let mut rbo: GlesRenderbuffer = state
        .renderer
        .create_buffer(drm_fourcc::DrmFourcc::Abgr8888, buf_size)
        .map_err(|e| anyhow::anyhow!("create offscreen buffer: {e}"))?;

    let phys_size: Size<i32, Physical> = (w, h).into();

    let space_elements: Vec<SpaceRenderElements<GlesRenderer, _>> = space_render_elements(
        &mut state.renderer,
        [&state.space],
        &state.output,
        1.0,
    )
    .unwrap_or_default();

    let pointer_loc = state.pointer.current_location();
    let mut elements: Vec<OutputRenderElements<GlesRenderer, _>> = Vec::new();

    let cursor_pos = (
        pointer_loc.x - state.cursor_default.hotspot.0 as f64,
        pointer_loc.y - state.cursor_default.hotspot.1 as f64,
    );
    if let Ok(cursor_elem) = MemoryRenderBufferRenderElement::from_buffer(
        &mut state.renderer,
        cursor_pos,
        &state.cursor_default.buffer,
        None,
        None,
        None,
        Kind::Cursor,
    ) {
        elements.push(OutputRenderElements::Cursor(cursor_elem));
    }

    let output_h = h;
    let taskbar_y = (output_h - TASKBAR_HEIGHT) as f64;
    if let Ok(bg) = MemoryRenderBufferRenderElement::from_buffer(
        &mut state.renderer,
        (0.0, taskbar_y),
        &state.taskbar_bg,
        None,
        None,
        None,
        Kind::Unspecified,
    ) {
        elements.push(OutputRenderElements::Cursor(bg));
    }

    for (i, (_, _, _, btn_buf)) in state.taskbar_buttons.iter().enumerate() {
        let x = (TASKBAR_BTN_MARGIN + i as i32 * (TASKBAR_BTN_WIDTH + TASKBAR_BTN_GAP)) as f64;
        let y = taskbar_y + ((TASKBAR_HEIGHT - TASKBAR_BTN_HEIGHT) / 2) as f64;
        if let Ok(btn) = MemoryRenderBufferRenderElement::from_buffer(
            &mut state.renderer, (x, y), btn_buf, None, None, None, Kind::Unspecified,
        ) {
            elements.push(OutputRenderElements::Cursor(btn));
        }
    }
    elements.extend(space_elements.into_iter().map(OutputRenderElements::Space));

    let mut damage_tracker = OutputDamageTracker::new(phys_size, Scale::from(1.0), Transform::Normal);
    {
        let mut target = state
            .renderer
            .bind(&mut rbo)
            .map_err(|e| anyhow::anyhow!("bind offscreen: {e}"))?;
        let _ = damage_tracker.render_output(
            &mut state.renderer,
            &mut target,
            0,
            &elements,
            [0.1, 0.1, 0.3, 1.0],
        ).map_err(|e| anyhow::anyhow!("offscreen render: {e}"))?;
    }

    let region: Rectangle<i32, BufferCoord> = Rectangle::from_size(buf_size);
    let target = state
        .renderer
        .bind(&mut rbo)
        .map_err(|e| anyhow::anyhow!("rebind for readback: {e}"))?;
    let mapping = state
        .renderer
        .copy_framebuffer(&target, region, drm_fourcc::DrmFourcc::Abgr8888)
        .map_err(|e| anyhow::anyhow!("copy_framebuffer: {e}"))?;
    let pixels = state
        .renderer
        .map_texture(&mapping)
        .map_err(|e| anyhow::anyhow!("map_texture: {e}"))?;

    let mut png_buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_buf, w as u32, h as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| anyhow::anyhow!("png header: {e}"))?;
        writer.write_image_data(pixels).map_err(|e| anyhow::anyhow!("png write: {e}"))?;
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_buf);
    Ok((w as u32, h as u32, b64))
}
