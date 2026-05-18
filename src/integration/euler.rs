use crate::graphics::vector::FourVector;

const EULER_STEP_SIZE: f32 = 0.01;

pub fn euler_step(x0: FourVector, x1: FourVector, del_x0: FourVector, del_x1: FourVector) -> (FourVector, FourVector) {
    (
        x0 + EULER_STEP_SIZE * del_x0,
        x1 + EULER_STEP_SIZE * del_x1,
    )
}