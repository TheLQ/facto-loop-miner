use std::path::Path;

pub struct ImageArray {
    buffer: Vec<u32>,
    width: u32,
    height: u32,
}

impl ImageArray {
    pub fn new(width: u32, height: u32) -> Self {
        let size = width * height;
        let buffer: Vec<u32> = (0..size).map(|_| 0x3B4F3CFF).collect();
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
    }
}

pub fn color_for_resource(resource_name: &str) -> Option<u32> {
    return if resource_name == "iron-ore" {
        Some(color_from_hex("#688290ff"))
    } else if resource_name == "copper-ore" {
        Some(color_from_hex("#c86230ff"))
    } else if resource_name == "stone" {
        Some(color_from_hex("#b09868ff"))
    } else if resource_name == "coal" {
        Some(color_from_hex("#000000ff"))
    } else if resource_name == "uranium-ore" {
        Some(color_from_hex("#00b200ff"))
    } else {
        None
    };
}

fn color_from_hex(hex_code: &str) -> u32 {
    let hex_without_hash = &hex_code[1..];
    // println!("testing {}", hex_without_hash);
    u32::from_str_radix(hex_without_hash, 16).unwrap()
}
