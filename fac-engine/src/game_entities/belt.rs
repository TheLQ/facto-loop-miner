use exhaustive::Exhaustive;

#[derive(Debug, Clone, Exhaustive)]
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
