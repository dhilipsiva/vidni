//! SVG pixels and semantic targets measured in the same pinned browser.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema: u32,
    pub image: String,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub browser: String,
    pub targets: BTreeMap<String, Vec<[f64; 4]>>,
}

pub(super) fn picture(dir: &Path, scene: &Scene, key: &str) -> Result<(), String> {
    let bytes = fs::read(dir.join(&scene.url)).map_err(|e| e.to_string())?;
    // Only the document size is read by usvg; Chromium lays out and draws text.
    let tree = resvg::usvg::Tree::from_data(&bytes, &resvg::usvg::Options::default())
        .map_err(|e| e.to_string())?;
    let size = [
        tree.size().width().ceil() as u32,
        tree.size().height().ceil() as u32,
    ];
    let scale = scene.scale.unwrap_or(1);
    let mut page = Page::open(
        size,
        scale,
        "dark",
        Limits {
            offline: true,
            ..Limits::default()
        },
    )?;
    page.local = Some(Local::new(dir)?);
    page.whole = true;
    page.goto(&address(dir, &scene.url)?)?;

    // An explicit @font-face for every used family prevents host-font fallback.
    let families = page.eval(r#"[...document.querySelectorAll('text,tspan')].map(e => ({family:getComputedStyle(e).fontFamily.split(',')[0].replace(/["']/g,'').trim(),weight:getComputedStyle(e).fontWeight,text:e.textContent}))"#)?;
    let mut mapped = BTreeMap::new();
    let mut css = String::new();
    let catalog = vidni_core::fonts::all();
    for family in families.as_array().ok_or("SVG font list is not an array")? {
        let name = family["family"].as_str().ok_or("SVG font has no family")?;
        let weight = family["weight"].as_str().unwrap_or("400");
        let font = catalog
            .iter()
            .find(|f| {
                f.family == name
                    && (f.style == "Bold") == (weight.parse::<u16>().unwrap_or(400) >= 600)
            })
            .or_else(|| catalog.iter().find(|f| f.family == name))
            .ok_or_else(|| format!("{}: font family {name:?} is not pinned", scene.url))?;
        let lookup = format!("{name}/{weight}");
        let data = fs::read(vidni_core::fonts::find(font, &vidni_core::fonts::dirs())?)
            .map_err(|e| e.to_string())?;
        let face = ttf_parser::Face::parse(&data, 0).map_err(|e| e.to_string())?;
        if let Some(missing) = family["text"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .find(|c| !c.is_control() && !c.is_whitespace() && face.glyph_index(*c).is_none())
        {
            return Err(format!(
                "{}: pinned font {} has no glyph for {missing:?}; choose a pinned font that covers it",
                scene.url, font.id
            ));
        }
        if mapped.contains_key(&lookup) {
            continue;
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        let alias = format!("vidni-{}", font.id);
        css.push_str(&format!("@font-face{{font-family:'{alias}';font-weight:100 900;src:url(data:font/otf;base64,{encoded})}}\n"));
        mapped.insert(lookup, alias);
    }
    page.eval(&format!(r#"(async () => {{
      const style=document.createElementNS('http://www.w3.org/2000/svg','style');
      style.textContent={css}; document.documentElement.prepend(style);
      const map={map};
      const nodes=[...document.querySelectorAll('text,tspan')];
      const names=nodes.map(e=>{{const s=getComputedStyle(e); return s.fontFamily.split(',')[0].replace(/["']/g,'').trim()+'/'+s.fontWeight;}});
      nodes.forEach((e,i)=>e.style.fontFamily='"'+map[names[i]]+'"');
      await document.fonts.ready; return true;
    }})()"#, css=json!(css), map=json!(mapped)))?;
    page.settle()?;

    let mut imports = BTreeMap::new();
    let mut extra_files = Files::new();
    for (name, target) in &scene.targets {
        let Some(import) = &target.capture else {
            continue;
        };
        let receipt_bytes = fs::read(dir.join(&import.receipt)).map_err(|e| e.to_string())?;
        let receipt: Receipt = serde_json::from_slice(&receipt_bytes).map_err(|e| e.to_string())?;
        if receipt.schema != 1
            || receipt.width == 0
            || receipt.height == 0
            || receipt.browser.is_empty()
        {
            return Err(format!("target {name}: invalid capture receipt"));
        }
        if Path::new(&receipt.image)
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
        {
            return Err(format!(
                "target {name}: receipt image must be inside the project"
            ));
        }
        let image = fs::read(dir.join(&receipt.image)).map_err(|e| e.to_string())?;
        if keys::sha256_hex(&image) != receipt.sha256 {
            return Err(format!(
                "target {name}: screenshot differs from its measured receipt"
            ));
        }
        let pixels = crate::compose::image::Image::load_png(&dir.join(&receipt.image))?;
        if [pixels.width, pixels.height] != [receipt.width, receipt.height] {
            return Err(format!(
                "target {name}: screenshot dimensions differ from its measured receipt"
            ));
        }
        let element = target.element.as_ref().ok_or("capture needs element")?;
        let href = page.eval(&format!(
            "document.getElementById({}).href.baseVal",
            json!(element)
        ))?;
        let href = href.as_str().ok_or("capture image has no href")?;
        let embedded = href
            .strip_prefix("data:image/png;base64,")
            .ok_or("measured capture images must embed their exact PNG bytes")?;
        let embedded = base64::engine::general_purpose::STANDARD
            .decode(embedded)
            .map_err(|e| e.to_string())?;
        if keys::sha256_hex(&embedded) != receipt.sha256 {
            return Err(format!(
                "target {name}: embedded image differs from its measured receipt"
            ));
        }
        let boxes = receipt
            .targets
            .get(&import.target)
            .ok_or_else(|| format!("target {name}: receipt has no target {}", import.target))?;
        imports.insert(
            name,
            json!({"boxes":boxes,"width":receipt.width,"height":receipt.height}),
        );
        extra_files.insert(
            import.receipt.clone(),
            Some(keys::sha256_hex(&receipt_bytes)),
        );
        extra_files.insert(receipt.image, Some(receipt.sha256));
    }
    let targets = page.eval(&format!(
        "({})({},{})",
        include_str!("measured.js"),
        json!(scene.targets),
        json!(imports)
    ))?;
    let targets = serde_json::from_value(targets).map_err(|e| e.to_string())?;
    let png = page.settle()?;
    page.faults()?;
    let browser = page.browser.call(None, "Browser.getVersion", json!({}))?["product"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let frames = Frames {
        viewport: size,
        scale,
        targets,
        highlights: BTreeMap::new(),
        browser: format!(
            "{browser}; measured-svg-3; fonts:{}",
            keys::sha256_hex(vidni_core::fonts::DATA.as_bytes())
        ),
        rows: None,
    };
    let mut files = page.record()?;
    files.insert(scene.url.clone(), Some(keys::sha256_hex(&bytes)));
    files.extend(extra_files);
    write(
        &keys::files_path(dir, key),
        keys::files_json(&files).as_bytes(),
    )?;
    write(
        &keys::frames_path(dir, key),
        serde_json::to_string_pretty(&frames)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    write(&keys::path(dir, key), &png)
}
