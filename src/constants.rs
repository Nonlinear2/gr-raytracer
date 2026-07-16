#[derive(Clone, Copy, PartialEq)]
pub enum Pipeline {
    CPU,
    GPU,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IntegrationMethod {
    EULER,
    RK4
}