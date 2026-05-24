#[cfg(target_os = "linux")]
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;
#[cfg(target_os = "linux")]
use smithay::utils::Transform;

#[cfg(target_os = "linux")]
use drm_fourcc::DrmFourcc;

#[cfg(target_os = "linux")]
const DEFAULT_THEME: &str = "Adwaita";
#[cfg(target_os = "linux")]
const DEFAULT_SIZE: u32 = 24;

#[cfg(target_os = "linux")]
const FALLBACK_HOTSPOT: (u32, u32) = (1, 1);

#[cfg(target_os = "linux")]
pub struct CursorTheme {
    theme: xcursor::CursorTheme,
    size: u32,
}

#[cfg(target_os = "linux")]
pub struct LoadedCursor {
    pub buffer: MemoryRenderBuffer,
    pub hotspot: (i32, i32),
}

#[cfg(target_os = "linux")]
impl CursorTheme {
    pub fn load() -> Self {
        let theme_name = std::env::var("XCURSOR_THEME").unwrap_or_else(|_| DEFAULT_THEME.into());
        let size: u32 = std::env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_SIZE);
        let theme = xcursor::CursorTheme::load(&theme_name);
        tracing::info!(%theme_name, size, "cursor theme loaded");
        Self { theme, size }
    }

    pub fn load_cursor(&self, name: &str) -> LoadedCursor {
        if let Some(path) = self.theme.load_icon(name) {
            if let Ok(data) = std::fs::read(&path) {
                if let Some(images) = xcursor::parser::parse_xcursor(&data) {
                    if let Some(img) = select_best_image(&images, self.size) {
                        return LoadedCursor {
                            buffer: image_to_buffer(img),
                            hotspot: (img.xhot as i32, img.yhot as i32),
                        };
                    }
                }
            }
            tracing::warn!(name, ?path, "failed to parse cursor file");
        } else {
            tracing::warn!(name, "cursor not found in theme");
        }
        fallback_cursor()
    }
}

#[cfg(target_os = "linux")]
fn select_best_image<'a>(
    images: &'a [xcursor::parser::Image],
    target_size: u32,
) -> Option<&'a xcursor::parser::Image> {
    images
        .iter()
        .min_by_key(|img| (img.size as i64 - target_size as i64).unsigned_abs())
}

#[cfg(target_os = "linux")]
fn image_to_buffer(img: &xcursor::parser::Image) -> MemoryRenderBuffer {
    let w = img.width as i32;
    let h = img.height as i32;
    let expected = (w as usize) * (h as usize) * 4;
    let data = if img.pixels_rgba.len() == expected {
        &img.pixels_rgba[..]
    } else {
        tracing::warn!(w, h, got = img.pixels_rgba.len(), expected, "xcursor pixel size mismatch, using fallback");
        return fallback_cursor().buffer;
    };
    MemoryRenderBuffer::from_slice(
        data,
        DrmFourcc::Abgr8888,
        (w, h),
        1,
        Transform::Normal,
        None,
    )
}

#[cfg(target_os = "linux")]
fn fallback_cursor() -> LoadedCursor {
    #[rustfmt::skip]
    let arrow: [[u8; 16]; 16] = [
        [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [1,2,1,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [1,2,2,1,0,0,0,0,0,0,0,0,0,0,0,0],
        [1,2,2,2,1,0,0,0,0,0,0,0,0,0,0,0],
        [1,2,2,2,2,1,0,0,0,0,0,0,0,0,0,0],
        [1,2,2,2,2,2,1,0,0,0,0,0,0,0,0,0],
        [1,2,2,2,2,2,2,1,0,0,0,0,0,0,0,0],
        [1,2,2,2,2,2,2,2,1,0,0,0,0,0,0,0],
        [1,2,2,2,2,2,1,1,1,1,0,0,0,0,0,0],
        [1,2,2,2,2,2,1,0,0,0,0,0,0,0,0,0],
        [1,2,2,1,1,2,2,1,0,0,0,0,0,0,0,0],
        [1,2,1,0,0,1,2,1,0,0,0,0,0,0,0,0],
        [1,1,0,0,0,1,2,2,1,0,0,0,0,0,0,0],
        [1,0,0,0,0,0,1,2,1,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0],
    ];
    let mut data = vec![0u8; 16 * 16 * 4];
    for y in 0..16 {
        for x in 0..16 {
            let i = (y * 16 + x) * 4;
            match arrow[y][x] {
                1 => { data[i] = 0; data[i + 1] = 0; data[i + 2] = 0; data[i + 3] = 255; }
                2 => { data[i] = 255; data[i + 1] = 255; data[i + 2] = 255; data[i + 3] = 255; }
                _ => {}
            }
        }
    }
    LoadedCursor {
        buffer: MemoryRenderBuffer::from_slice(
            &data,
            DrmFourcc::Abgr8888,
            (16, 16),
            1,
            Transform::Normal,
            None,
        ),
        hotspot: (FALLBACK_HOTSPOT.0 as i32, FALLBACK_HOTSPOT.1 as i32),
    }
}
