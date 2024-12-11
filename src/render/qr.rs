use regex::Regex;

use crate::{
    Group, Error, Path, QRBill, QrCode, Polygon, Rectangle,
    mm,
};

impl QRBill {

    pub fn section_qr(&self) -> Result<Group, Error> {
        let x_lhs = mm(5.0);
        let x_mid = crate::RECEIPT_WIDTH + x_lhs;

        let path_re = Regex::new(r"<path [^>]*>").unwrap();
        let data_re = Regex::new(r#" d="([^"]*)""#).unwrap();
        let size_re = Regex::new(r#"<svg .* width="(\d*)" [^>]*>"#).unwrap();

        let qr_image = self.qr_image()?;

        let size = size_re
            .captures_iter(&qr_image)
            .next()
            .expect("This is a bug. Please report it.");

        let path = path_re
            .captures_iter(&qr_image)
            .next()
            .expect("This is a bug. Please report it.");

        let data = data_re
            .captures_iter(&path[0])
            .next()
            .expect("This is a bug. Please report it.");

        let qr_left = x_mid;
        let qr_top = 60.0;
        let scale_factor = mm(45.8)
            / size[1]
            .parse::<f64>()
            .expect("This is a bug. Please report it.");

        let mut group = Group::new();
        group = group.add(
            Path::new()
                .set("d", &data[1])
                .set(
                    "style",
                    "fill:black; fill-opacity:1; fill-rule:nonzero; stroke:none; margin: 0",
                )
                .set(
                    "transform",
                    format!("translate({}, {}) scale({})", qr_left, qr_top, scale_factor),
                ),
        );

        group = group.add(Self::draw_swiss_cross(x_mid, 60.0, mm(45.8)));
        Ok(group)
    }

    /// Generate the QR image in string form.
    pub fn qr_image(&self) -> Result<String, Error> {
        let code = QrCode::with_error_correction_level(self.qr_data(), qrcode::EcLevel::M)?;
        Ok(code
           .render()
           .dark_color(qrcode::render::svg::Color("black"))
           .light_color(qrcode::render::svg::Color("white"))
           .quiet_zone(false)
           .build())
    }

    /// Draw the swiss cross in the middle of the QR code.
    pub fn draw_swiss_cross(x: f64, y: f64, size: f64) -> Group {
        let scale_factor = mm(7.0) / 19.0;
        Group::new()
            .add(
                Polygon::new()
                    .set("points", "18.3,0.7 1.6,0.7 0.7,0.7 0.7,1.6 0.7,18.3 0.7,19.1 1.6,19.1 18.3,19.1 19.1,19.1 19.1,18.3 19.1,1.6 19.1,0.7")
                    .set("fill", "black")
            )
            .add(
                Rectangle::new()
                    .set("x", 8.3)
                    .set("y", 4.0)
                    .set("width", 3.3)
                    .set("height", 11.0)
                    .set("fill", "white")
            )
            .add(
                Rectangle::new()
                    .set("x", 4.4)
                    .set("y", 7.9)
                    .set("width", 11.0)
                    .set("height", 3.3)
                    .set("fill", "white")
            )
            .add(
                Polygon::new()
                    .set("points", "0.7,1.6 0.7,18.3 0.7,19.1 1.6,19.1 18.3,19.1 19.1,19.1 19.1,18.3 19.1,1.6 19.1,0.7 18.3,0.7 1.6,0.7 0.7,0.7")
                    .set("fill", "none")
                    .set("stroke", "white")
                    .set("stroke_width", 1.4357)
            )
            .set("transform", format!("translate({}, {}) scale({})", x + size / 2.0 - 10.0 * scale_factor, y + size / 2.0 - 10.0 * scale_factor, scale_factor))
            .set("id", "swiss-cross")
    }
}
