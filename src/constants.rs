#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum Pipeline {
    CPU,
    GPU,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Clone, Copy, PartialEq)]
pub enum IntegrationMethod {
    EULER = 0,
    RK4 = 1,
}