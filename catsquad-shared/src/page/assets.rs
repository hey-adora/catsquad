pub const LINK_WEB_CSS: &str = "/catsquad.css";
pub const LINK_WEB_WASM: &str = "/catsquad_bg.wasm";
pub const LINK_WEB_JS: &str = "/catsquad.js";
pub const LINK_WEB_FAVICON: &str = "/favicon.ico";
pub const LINK_WEB_FONT_HI: &str = "/font_hi.woff2";
pub const LINK_WEB_FONT_LUCKY: &str = "/font_lucky.ttf";

pub fn link_relative_img(hash: impl AsRef<str>, extension: impl AsRef<str>) -> String {
    format!("/file/{}.{}", hash.as_ref(), extension.as_ref())
}
