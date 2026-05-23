
// fn euler_step(x: PackedPoint4, k: PackedFourVector, del_x: PackedPoint4, del_k: PackedFourVector) -> PackedPhoton4 {
//     var new_x = new_point4(
//         x.inner.x + EULER_STEP_SIZE * del_x.inner.x,
//         x.inner.y + EULER_STEP_SIZE * del_x.inner.y,
//         x.inner.z + EULER_STEP_SIZE * del_x.inner.z,
//         x.inner.w + EULER_STEP_SIZE * del_x.inner.w,
//         x.chart
//     );

//     var new_k = new_four_vector(
//         k.inner.x + EULER_STEP_SIZE * del_k.inner.x,
//         k.inner.y + EULER_STEP_SIZE * del_k.inner.y,
//         k.inner.z + EULER_STEP_SIZE * del_k.inner.z,
//         k.inner.w + EULER_STEP_SIZE * del_k.inner.w,
//         k.vector_space
//     );

//     if (is_spherical(x.chart)) {
//         var theta = new_x.inner.z;
//         var phi = new_x.inner.w;
//         var k_theta = new_k.inner.z;

//         if (new_x.inner.y < 0.0) {
//             new_x.inner.y = 0.0;
//         }

//         if (theta < 0.0) {
//             theta = -theta;
//             k_theta = -k_theta;
//             phi = phi + PI;
//         }

//         if (theta > PI) {
//             theta = TAU - theta;
//             k_theta = -k_theta;
//             phi = phi + PI;
//         }

//         new_x.inner.z = clamp(theta, 0.0, PI);
//         new_x.inner.w = phi - TAU * floor(phi / TAU); // mod tau
//         new_k.inner.z = k_theta;
//     }

//     return PackedPhoton4(new_x, new_k);
// }