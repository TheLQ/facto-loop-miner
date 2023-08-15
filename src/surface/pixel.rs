#[derive(Clone, Debug, PartialEq)]
pub enum Pixel {
    Iron,
    Copper,
    Stone,
    Coal,
    Uranium,
    Water,
    CrudeOil,
    //
    Empty,
    EdgeWall,
}

impl Pixel {
    pub fn parse(name: &str) -> Pixel {
        match name {
            "iron-ore" => Pixel::Iron,
            "copper-ore" => Pixel::Copper,
            "stone" => Pixel::Stone,
            "coal" => Pixel::Coal,
            "uranium-ore" => Pixel::Uranium,
            "water" => Pixel::Water,
            "crude-oil" => Pixel::CrudeOil,
            //
            "loop-empty" => Pixel::Empty,
            "loop-edge" => Pixel::EdgeWall,
            _ => panic!("unknown name {}", name),
        }
    }

    pub fn lua_resource_name(&self) -> &str {
        match self {
            Pixel::Iron => "iron-ore",
            Pixel::Copper => "copper-ore",
            Pixel::Stone => "stone",
            Pixel::Coal => "coal",
            Pixel::Uranium => "uranium-ore",
            Pixel::Water => "water",
            Pixel::CrudeOil => "crude-oil",
            //
            Pixel::Empty => "loop-empty",
            Pixel::EdgeWall => "loop-edge",
        }
    }

    pub fn color(&self) -> [u8; 3] {
        match self {
            Pixel::Iron => [0x68, 0x82, 0x90],
            Pixel::Copper => [0xc8, 0x62, 0x30],
            Pixel::Stone => [0xb0, 0x98, 0x68],
            Pixel::Coal => [0x5e, 0x62, 0x66],
            Pixel::Uranium => [0x0b, 0x20, 0x00],
            Pixel::Water => [0xFF, 0xFF, 0xFF],
            Pixel::CrudeOil => [0x20, 0x66, 0xFF],
            //
            Pixel::Empty => [0x00, 0x00, 0x00],
            Pixel::EdgeWall => [0xBD, 0x5F, 0x5F],
        }
    }
}
