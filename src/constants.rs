#[derive(Clone, Copy, PartialEq)]
pub enum Pipeline {
    CPU,
    GPU,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq)]
pub enum IntegrationMethod {
    EULER = 0,
    RK4 = 1,
}