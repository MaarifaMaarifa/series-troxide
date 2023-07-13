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

    pub static ARROW_BAR_UP: &[u8; 376] = include_bytes!("../../assets/icons/arrow-bar-up.svg");
    pub static ARROW_BAR_DOWN: &[u8; 375] = include_bytes!("../../assets/icons/arrow-bar-down.svg");
    pub static ARROW_LEFT: &[u8; 311] = include_bytes!("../../assets/icons/arrow-left.svg");
    pub static CHECK_CIRCLE: &[u8; 387] = include_bytes!("../../assets/icons/check-circle.svg");
    pub static CHECK_CIRCLE_FILL: &[u8; 340] =
        include_bytes!("../../assets/icons/check-circle-fill.svg");
}

pub mod fonts {
    pub static NOTOSANS_REGULAR_STATIC: &[u8; 556216] =
        include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");
}
