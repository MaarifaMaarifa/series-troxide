use std::borrow::Cow;

/// Creates a Cow from static bytes array of assets
///
/// This is usefull when creating handles for SVG and IMAGE in iced
pub fn get_static_cow_from_asset(static_asset: &'static [u8]) -> Cow<'static, [u8]> {
    Cow::Borrowed(static_asset)
}

pub mod icons {

    pub static BINOCULARS_FILL: &[u8; 639] =
        include_bytes!("../../assets/icons/binoculars-fill.svg");
    pub static CARD_CHECKLIST: &[u8; 730] = include_bytes!("../../assets/icons/card-checklist.svg");
    pub static FILM: &[u8; 384] = include_bytes!("../../assets/icons/film.svg");
    pub static GRAPH_UP_ARROW: &[u8; 402] = include_bytes!("../../assets/icons/graph-up-arrow.svg");
    pub static GEAR_WIDE_CONNECTED: &[u8; 1312] =
        include_bytes!("../../assets/icons/gear-wide-connected.svg");

    pub static CLOCK_FILL: &[u8; 267] = include_bytes!("../../assets/icons/clock-fill.svg");
    pub static CHEVRON_UP: &[u8; 271] = include_bytes!("../../assets/icons/chevron-up.svg");
    pub static CHEVRON_DOWN: &[u8; 290] = include_bytes!("../../assets/icons/chevron-down.svg");
    pub static STAR: &[u8; 635] = include_bytes!("../../assets/icons/star.svg");
    pub static STAR_FILL: &[u8; 399] = include_bytes!("../../assets/icons/star-fill.svg");
    pub static PATCH_PLUS: &[u8; 1039] = include_bytes!("../../assets/icons/patch-plus.svg");
    pub static PATCH_PLUS_FILL: &[u8; 607] =
        include_bytes!("../../assets/icons/patch-plus-fill.svg");
}

pub mod fonts {
    pub static NOTOSANS_REGULAR_STATIC: &[u8; 556216] =
        include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");
}
