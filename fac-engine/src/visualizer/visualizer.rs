use std::{fs, path::Path};

use tracing::info;

use crate::blueprint::contents::BlueprintContents;

const STYLE: &str = r"
<style>
.placed {
    position: absolute;
    border: 1px solid;
}
</style>";

pub fn visualize_blueprint(blueprint: &BlueprintContents) {
    let mut output = STYLE.to_string();
    output.push_str("<div>\n");
    for bpitem in blueprint.items() {
        let entity = bpitem.entity();
        let size = entity.rectangle_size();
        let name = entity.name();
        let pos = bpitem.position();

        let size_unit = 50;
        let size_unit_usize = 50usize;
        output.push_str(r#"<div style="word-break: break-word;"#);
        output.push_str(&format!("top: {}px;", pos.y() * size_unit));
        output.push_str(&format!("left: {}px;", pos.x() * size_unit));
        output.push_str(&format!("width: {}px;", size.width() * size_unit_usize));
        output.push_str(&format!("height: {}px;", size.height() * size_unit_usize));
        output.push_str(r#"" class="placed">"#);
        output.push('\n');

        output.push_str(name.as_ref());

        output.push_str("</div>\n");
    }

    output.push_str("</div>");

    let path = Path::new("out.html").to_path_buf().canonicalize().unwrap();
    fs::write(&path, output).unwrap();
    info!("wrote to {}", path.display());
}
