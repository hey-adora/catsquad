use std::path::Path;

use tokio::fs;

#[derive(Clone, Debug)]
pub struct Assets {
    pub wasm: Vec<u8>,
    pub index: Vec<u8>,
    pub css: Vec<u8>,
    pub js: Vec<u8>,
    pub favicon: Vec<u8>,
    pub font_hi: Vec<u8>,
    pub font_lucky: Vec<u8>,
}

impl Assets {
    pub async fn new(assets_path: impl AsRef<str>) -> Self {
        let assets_path = assets_path.as_ref();

        let read_file = async |file_name: &str| {
            let asset_path = Path::new(&assets_path).join(file_name);
            let bytes = fs::read(&asset_path)
                .await
                .expect(&format!("file not found: {:?}", asset_path));
            bytes
        };

        let wasm = read_file("catsquad_bg.wasm").await;
        let js = read_file("catsquad.js").await;
        let css = read_file("catsquad.css").await;
        let index = read_file("index.html").await;
        let favicon = read_file("favicon.ico").await;
        let font_hi =
            read_file("atkinson_hyperlegible_next/atkinson_hyperlegible_next_vf-variable.woff2")
                .await;
        let font_lucky = read_file("LuckiestGuy-Regular.ttf").await;

        Self {
            wasm,
            index,
            css,
            js,
            favicon,
            font_hi,
            font_lucky,
        }
    }

    pub fn mem() -> Self {
        Self {
            wasm: b"wasm".to_vec(),
            index: b"index".to_vec(),
            css: b"css".to_vec(),
            js: b"js".to_vec(),
            favicon: b"favicon".to_vec(),
            font_hi: b"font_hi".to_vec(),
            font_lucky: b"font_lucky".to_vec(),
        }
    }
}

#[tokio::test]
async fn test_assets_reader() {
    fs::create_dir_all("/tmp/catsquad-dev/test_assets_reader/atkinson_hyperlegible_next")
        .await
        .unwrap();
    fs::write("/tmp/catsquad-dev/test_assets_reader/index.html", "index")
        .await
        .unwrap();
    fs::write(
        "/tmp/catsquad-dev/test_assets_reader/catsquad_bg.wasm",
        "wasm",
    )
    .await
    .unwrap();
    fs::write("/tmp/catsquad-dev/test_assets_reader/catsquad.js", "js")
        .await
        .unwrap();
    fs::write("/tmp/catsquad-dev/test_assets_reader/favicon.ico", "ico")
        .await
        .unwrap();
    fs::write("/tmp/catsquad-dev/test_assets_reader/catsquad.css", "css")
        .await
        .unwrap();
    fs::write("/tmp/catsquad-dev/test_assets_reader/atkinson_hyperlegible_next/atkinson_hyperlegible_next_vf-variable.woff2", "font_hi")
        .await
        .unwrap();
    fs::write(
        "/tmp/catsquad-dev/test_assets_reader/LuckiestGuy-Regular.ttf",
        "font_lucky",
    )
    .await
    .unwrap();
    let assets = Assets::new("/tmp/catsquad-dev/test_assets_reader").await;
    assert_eq!(String::from_utf8_lossy(&assets.index), "index");
    assert_eq!(String::from_utf8_lossy(&assets.wasm), "wasm");
    assert_eq!(String::from_utf8_lossy(&assets.js), "js");
    assert_eq!(String::from_utf8_lossy(&assets.favicon), "ico");
    assert_eq!(String::from_utf8_lossy(&assets.css), "css");
    assert_eq!(String::from_utf8_lossy(&assets.font_hi), "font_hi");
    assert_eq!(String::from_utf8_lossy(&assets.font_lucky), "font_lucky");
}
