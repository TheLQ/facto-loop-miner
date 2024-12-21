#[derive(Debug, Clone)]
pub enum FacEntBeltType {
    Basic,
    Fast,
    Express,
}

impl FacEntBeltType {
    pub fn name_prefix(&self) -> &str {
        match self {
            FacEntBeltType::Basic => "",
            FacEntBeltType::Fast => "fast-",
            FacEntBeltType::Express => "express-",
        }
    }
}
