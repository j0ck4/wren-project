//! "Share tunnel" dialog: renders the .conf as a QR code so a
//! mobile WireGuard app can import it by camera.
//!
//! We render the QR matrix directly into a 24-bit RGB pixbuf
//! and wrap it in a `gdk::Texture`. This keeps `qrcode`'s
//! default features (which pull in the heavy `image` crate)
//! disabled.

use adw::prelude::*;
use anyhow::Result;
use gtk::{gdk, gdk_pixbuf, glib};
use qrcode::{Color as QrColor, QrCode};

const MODULE_PIXELS: usize = 8;
const QUIET_MODULES: usize = 2;

pub fn show(parent: &impl IsA<gtk::Widget>, name: &str, conf: &str) {
    let dialog = adw::Dialog::builder()
        .title(format!("{name} — Share"))
        .content_width(420)
        .build();

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());

    let outer = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    match render_qr(conf) {
        Ok(texture) => {
            let frame = gtk::Frame::builder().halign(gtk::Align::Center).build();
            let picture = gtk::Picture::for_paintable(&texture);
            picture.set_size_request(300, 300);
            picture.set_content_fit(gtk::ContentFit::Contain);
            picture.set_can_shrink(false);
            frame.set_child(Some(&picture));
            outer.append(&frame);
        }
        Err(e) => {
            tracing::error!("QR render: {e:#}");
            let label = gtk::Label::new(Some(&format!("Could not generate QR: {e}")));
            outer.append(&label);
        }
    }

    let desc = gtk::Label::builder()
        .label("Scan with the WireGuard mobile app to import this tunnel.")
        .wrap(true)
        .justify(gtk::Justification::Center)
        .halign(gtk::Align::Center)
        .build();
    desc.add_css_class("dim-label");
    outer.append(&desc);

    let copy = gtk::Button::builder()
        .label("Copy Configuration")
        .halign(gtk::Align::Center)
        .build();
    copy.add_css_class("pill");
    let conf_owned = conf.to_string();
    copy.connect_clicked(move |btn| {
        btn.clipboard().set_text(&conf_owned);
        btn.set_label("Copied");
        btn.set_sensitive(false);
        let btn_weak = btn.downgrade();
        glib::timeout_add_seconds_local_once(2, move || {
            if let Some(btn) = btn_weak.upgrade() {
                btn.set_label("Copy Configuration");
                btn.set_sensitive(true);
            }
        });
    });
    outer.append(&copy);

    toolbar.set_content(Some(&outer));
    dialog.set_child(Some(&toolbar));
    dialog.present(Some(parent));
}

// pixel_width is a small QR raster dimension (a few hundred px at most),
// so the usize -> i32 casts for the pixbuf API can never truncate or wrap.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn render_qr(text: &str) -> Result<gdk::Texture> {
    let code = QrCode::new(text.as_bytes())?;
    let modules = code.to_colors();
    let width = code.width();
    let pixel_width = (width + 2 * QUIET_MODULES) * MODULE_PIXELS;
    let quiet_pixels = QUIET_MODULES * MODULE_PIXELS;

    let pixel_count = pixel_width * pixel_width;
    let mut buf = vec![255u8; pixel_count * 3]; // start fully white

    for y in 0..width {
        for x in 0..width {
            if modules[y * width + x] != QrColor::Dark {
                continue;
            }
            for dy in 0..MODULE_PIXELS {
                for dx in 0..MODULE_PIXELS {
                    let py = quiet_pixels + y * MODULE_PIXELS + dy;
                    let px = quiet_pixels + x * MODULE_PIXELS + dx;
                    let offset = (py * pixel_width + px) * 3;
                    buf[offset] = 0;
                    buf[offset + 1] = 0;
                    buf[offset + 2] = 0;
                }
            }
        }
    }

    let bytes = glib::Bytes::from(&buf);
    let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
        &bytes,
        gdk_pixbuf::Colorspace::Rgb,
        false,
        8,
        pixel_width as i32,
        pixel_width as i32,
        (pixel_width * 3) as i32,
    );

    Ok(gdk::Texture::for_pixbuf(&pixbuf))
}
