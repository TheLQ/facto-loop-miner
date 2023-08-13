use std::path::Path;

pub struct ImageArray {
    buffer: Vec<u32>,
    width: u32,
    height: u32,
}

// const BACKGROUND_COLOR: u32 = 0x3B4F3CFF;
const BACKGROUND_COLOR: u32 = 0xFF0000FF;

impl ImageArray {
    pub fn new(width: u32, height: u32) -> Self {
        let size = width * height;
        let buffer: Vec<u32> = (0..size).map(|_| BACKGROUND_COLOR).collect();
        println!("expected {} got {}", size, buffer.len());
        ImageArray {
            buffer,
            width,
            height,
        }
    }

    pub fn set_color(&mut self, color: u32, x: u32, y: u32) {
        let i: usize = (self.width * y + x).try_into().unwrap();
        // println!("size {}", self.buffer.len());
        self.buffer[i] = color;
    }

    pub fn save(self, path: &Path) {
        println!("Saving image to {}", path.display());
        let converted = unsafe { self.buffer.align_to::<u8>().1 };
        // let raw = self.buffer.as_slice();
        // let converted = raw as &[u8];
        // let converted = convert(raw);
        image::save_buffer(
            path,
            converted,
            self.width,
            self.height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        println!("Saved");
    }
}

pub fn color_for_resource(resource_name: &str) -> Option<u32> {
    return match resource_name {
        "iron-ore" => Some(color_from_hex("#688290ff")),
        "copper-ore" => Some(color_from_hex("#c86230ff")),
        "stone" => Some(color_from_hex("#b09868ff")),
        "coal" => Some(color_from_hex("#000000ff")),
        "uranium-ore" => Some(color_from_hex("#00b200ff")),
        "water" => Some(color_from_hex("#0000FFFF")),
        _ => None,
    };
}

fn color_from_hex(hex_code: &str) -> u32 {
    let hex_without_hash = &hex_code[1..];
    // println!("testing {}", hex_without_hash);
    u32::from_str_radix(hex_without_hash, 16).unwrap()
}
