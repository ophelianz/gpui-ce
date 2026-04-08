use std::io::Cursor;
use std::rc::Rc;

use anyhow::Result;

use crate::{Image, SharedString, SvgRenderer};

/// A tray action queued by the platform for the app to handle later on a safe tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrayIntent {
    /// Open or focus the main application window.
    OpenMainWindow,
    /// Open the primary download modal.
    OpenDownloadModal,
    /// Open the settings window.
    OpenSettings,
    /// Open the about surface.
    OpenAbout,
    /// Quit the application.
    Quit,
}

/// A menu item shown in the tray menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrayMenuItem {
    /// A separator line.
    Separator,
    /// A clickable tray item that enqueues an intent.
    Action {
        /// The user-visible label.
        name: SharedString,
        /// The intent to enqueue when clicked.
        intent: TrayIntent,
    },
}

/// A menu-bar or system-tray item owned by the application.
#[derive(Clone)]
pub struct Tray {
    /// Tooltip shown by the platform when supported.
    pub tooltip: Option<SharedString>,
    /// Text shown next to the icon when supported.
    pub title: Option<SharedString>,
    /// Source image for the tray icon.
    pub icon: Option<Rc<Image>>,
    /// Whether the platform should render the icon as a template/monochrome icon when supported.
    pub icon_is_template: bool,
    /// Menu items displayed by the tray.
    pub menu_items: Vec<TrayMenuItem>,
    /// Visibility of the tray item.
    pub visible: bool,
    pub(crate) icon_data: Option<TrayIconData>,
}

impl Tray {
    /// Create a new tray with default properties.
    pub fn new() -> Self {
        Self {
            tooltip: None,
            title: None,
            icon: None,
            icon_is_template: false,
            menu_items: Vec::new(),
            visible: true,
            icon_data: None,
        }
    }

    /// Set the tooltip.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the icon image.
    pub fn icon(mut self, icon: impl Into<Image>) -> Self {
        self.icon = Some(Rc::new(icon.into()));
        self
    }

    /// Mark the icon as a platform template icon when supported.
    pub fn icon_template(mut self, icon_is_template: bool) -> Self {
        self.icon_is_template = icon_is_template;
        self
    }

    /// Set the tray menu items.
    pub fn menu_items(mut self, menu_items: impl IntoIterator<Item = TrayMenuItem>) -> Self {
        self.menu_items = menu_items.into_iter().collect();
        self
    }

    /// Set the visibility.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub(crate) fn render_icon(&mut self, svg_renderer: SvgRenderer) -> Result<()> {
        let Some(icon) = &self.icon else {
            self.icon_data = None;
            return Ok(());
        };

        let image = icon.to_image_data(svg_renderer)?;
        let bytes = image.as_bytes(0).unwrap_or_default();
        let size = image.size(0);
        let rgba =
            image::RgbaImage::from_raw(size.width.0 as u32, size.height.0 as u32, bytes.to_vec())
                .ok_or_else(|| anyhow::anyhow!("failed to read tray icon pixels"))?;
        let mut encoded = Vec::new();
        image::DynamicImage::ImageRgba8(rgba)
            .write_to(&mut Cursor::new(&mut encoded), image::ImageFormat::Png)?;
        self.icon_data = Some(TrayIconData {
            png_bytes: Rc::new(encoded),
        });
        Ok(())
    }
}

impl Default for Tray {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub(crate) struct TrayIconData {
    pub(crate) png_bytes: Rc<Vec<u8>>,
}
