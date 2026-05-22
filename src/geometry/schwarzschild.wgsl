
// CONSTS

fn preferred_chart_for_point(point: PackedPoint3) -> u32 {
    let point_world = transition_point(point, CARTESIAN_WORLD);
    let p_rel = point_world.inner - SUBATLAS_CENTER;
    let dist_to_z_axis_sq = p_rel.x * p_rel.x + p_rel.y * p_rel.y;
    let dist_to_x_axis_sq = p_rel.y * p_rel.y + p_rel.z * p_rel.z;

    if (dist_to_z_axis_sq < dist_to_x_axis_sq) {
        return SPHERICAL_X;
    }
    return SPHERICAL_Z;
}

fn transition_point(point: PackedPoint3, to: u32) -> PackedPoint3 {
    if (point.chart == to) {
        return point;
    }

    switch (point.chart) {
        case CARTESIAN_WORLD: {
            let world_pos = point.inner;
            let p_rel = world_pos - SUBATLAS_CENTER;
            let r = length(p_rel);

            switch (to) {
                case SPHERICAL_Z: {
                    let new_theta = acos(p_rel.z / r);
                    var new_phi = atan2(p_rel.y, p_rel.x);
                    if (new_phi < 0.0) { new_phi = new_phi + TAU; }
                    return new_point3(r, new_theta, new_phi, SPHERICAL_Z);
                }
                case SPHERICAL_X: {
                    let new_theta = acos(p_rel.x / r);
                    var new_phi = atan2(p_rel.z, p_rel.y);
                    if (new_phi < 0.0) { new_phi = new_phi + TAU; }
                    return new_point3(r, new_theta, new_phi, SPHERICAL_X);
                }
                default: {
                    unreachable();
                }
            }
        }
        case SPHERICAL_Z: {
            switch (to) {
                case CARTESIAN_WORLD: {
                    let r = point.inner.x;
                    let theta = point.inner.y;
                    let phi = point.inner.z;

                    let x = r * sin(theta) * cos(phi);
                    let y = r * sin(theta) * sin(phi);
                    let z = r * cos(theta);
                    return new_point3(x + SUBATLAS_CENTER.x, y + SUBATLAS_CENTER.y, z + SUBATLAS_CENTER.z, CARTESIAN_WORLD);
                }
                case SPHERICAL_X: {
                    let world_point = transition_point(point, CARTESIAN_WORLD);
                    return transition_point(world_point, SPHERICAL_X);
                }
                default: {
                    unreachable();
                }
            }
        }
        case SPHERICAL_X: {
            switch (to) {
                case CARTESIAN_WORLD: {
                    let r = point.inner.x;
                    let theta = point.inner.y;
                    let phi = point.inner.z;

                    let x = r * cos(theta);
                    let y = r * sin(theta) * cos(phi);
                    let z = r * sin(theta) * sin(phi);
                    return new_point3(x + SUBATLAS_CENTER.x, y + SUBATLAS_CENTER.y, z + SUBATLAS_CENTER.z, CARTESIAN_WORLD);
                }
                case SPHERICAL_Z: {
                    let world_point = transition_point(point, CARTESIAN_WORLD);
                    return transition_point(world_point, SPHERICAL_Z);
                }
                default: {
                    unreachable();
                }
            }
        }
        default: {
            unreachable();
        }
    }
}

fn transition_vector(point: PackedPoint3, v: PackedThreeVector, to: u32) -> PackedThreeVector {
    if (point.chart == to) {
        return v;
    }

    let r = point.inner.x;
    let theta = point.inner.y;
    let phi = point.inner.z;

    switch (point.chart) {
        case CARTESIAN_WORLD: {
            let world_pos = point.inner;
            let p_rel = world_pos - SUBATLAS_CENTER;
            let x = p_rel.x;
            let y = p_rel.y;
            let z = p_rel.z;
            let rho_z = sqrt(x * x + y * y);
            let rho_x = sqrt(y * y + z * z);
            let r = sqrt(x * x + y * y + z * z);

            switch (to) {
                case SPHERICAL_Z: {
                    let dr_dx = x / r;
                    let dr_dy = y / r;
                    let dr_dz = z / r;
                    let dth_dx = x * z / (r * r * rho_z);
                    let dth_dy = y * z / (r * r * rho_z);
                    let dth_dz = -rho_z / (r * r);
                    let dph_dx = -y / (rho_z * rho_z);
                    let dph_dy = x / (rho_z * rho_z);

                    return new_three_vector(
                        dr_dx * v.inner.x + dr_dy * v.inner.y + dr_dz * v.inner.z,
                        dth_dx * v.inner.x + dth_dy * v.inner.y + dth_dz * v.inner.z,
                        dph_dx * v.inner.x + dph_dy * v.inner.y,
                        SPHERICAL_Z,
                    );
                }
                case SPHERICAL_X: {
                    let dr_dx = x / r;
                    let dr_dy = y / r;
                    let dr_dz = z / r;
                    let dth_dx = -rho_x / (r * r);
                    let dth_dy = x * y / (r * r * rho_x);
                    let dth_dz = x * z / (r * r * rho_x);
                    let dph_dy = -z / (rho_x * rho_x);
                    let dph_dz = y / (rho_x * rho_x);

                    return new_three_vector(
                        dr_dx * v.inner.x + dr_dy * v.inner.y + dr_dz * v.inner.z,
                        dth_dx * v.inner.x + dth_dy * v.inner.y + dth_dz * v.inner.z,
                        dph_dy * v.inner.y + dph_dz * v.inner.z,
                        SPHERICAL_X,
                    );
                }
                default: {
                    unreachable();
                }
            }
        }
        case SPHERICAL_Z: {
            switch (to) {
                case CARTESIAN_WORLD: {
                    let v_r = v.inner.x;
                    let v_theta = v.inner.y;
                    let v_phi = v.inner.z;
                    let sin_theta = sin(theta);
                    let cos_theta = cos(theta);
                    let sin_phi = sin(phi);
                    let cos_phi = cos(phi);

                    return new_three_vector(
                        sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
                        sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
                        cos_theta * v_r - r * sin_theta * v_theta,
                        CARTESIAN_WORLD,
                    );
                }
                case SPHERICAL_X: {
                    let world_point = transition_point(point, CARTESIAN_WORLD);
                    let world_vector = transition_vector(point, v, CARTESIAN_WORLD);
                    return transition_vector(world_point, world_vector, SPHERICAL_X);
                }
                default: {
                    unreachable();
                }
            }
        }
        case SPHERICAL_X: {
            switch (to) {
                case CARTESIAN_WORLD: {
                    let v_r = v.inner.x;
                    let v_theta = v.inner.y;
                    let v_phi = v.inner.z;
                    let sin_theta = sin(theta);
                    let cos_theta = cos(theta);
                    let sin_phi = sin(phi);
                    let cos_phi = cos(phi);

                    return new_three_vector(
                        cos_theta * v_r - r * sin_theta * v_theta,
                        sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
                        sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
                        CARTESIAN_WORLD,
                    );
                }
                case SPHERICAL_Z: {
                    let world_point = transition_point(point, CARTESIAN_WORLD);
                    let world_vector = transition_vector(point, v, CARTESIAN_WORLD);
                    return transition_vector(world_point, world_vector, SPHERICAL_Z);
                }
                default: {
                    unreachable();
                }
            }
        }
        default: {
            unreachable();
        }
    }
}

fn photon_to_world_pos(photon: PackedPhoton4) -> vec3<f32> {
    return transition_point(PackedPoint3(photon.pos.inner.yzw, photon.pos.chart), CARTESIAN_WORLD).inner;
}

fn photon_to_world_vel(photon: PackedPhoton4) -> vec3<f32> {
    let pos = PackedPoint3(photon.pos.inner.yzw, photon.pos.chart);
    return transition_vector(pos, PackedThreeVector(photon.vel.inner.yzw, photon.vel.vector_space), CARTESIAN_WORLD).inner;
}

fn world_photon3_to_photon4(world_photon: PackedPhoton3) -> PackedPhoton4 {
    let chart = preferred_chart_for_point(world_photon.pos);
    let pos = transition_point(world_photon.pos, chart);
    let vel = transition_vector(world_photon.pos, world_photon.vel, chart);
    let photon_x = new_point4(0.0, pos.inner.x, pos.inner.y, pos.inner.z, chart);

    let g = sch_g(photon_x);
    let b = 2.0 * (g[0][1] * vel.inner.y + g[0][2] * vel.inner.z + g[0][3] * vel.inner.w);
    let c =
          g[1][1] * vel.inner.y * vel.inner.y
        + 2.0 * g[1][2] * vel.inner.y * vel.inner.z
        + 2.0 * g[1][3] * vel.inner.y * vel.inner.w
        + g[2][2] * vel.inner.z * vel.inner.z
        + 2.0 * g[2][3] * vel.inner.z * vel.inner.w
        + g[3][3] * vel.inner.w * vel.inner.w;

    let k_0 = quadratic_positive_root(g[0][0], b, c);
    return PackedPhoton4(photon_x, PackedFourVector(vec4<f32>(k_0, vel.inner.x, vel.inner.y, vel.inner.z), chart));
}

fn sch_g(pos: PackedPoint4) -> mat4x4<f32> {
    let r = pos.inner.y;
    let theta = pos.inner.z;
    let f = 1.0 - R_S / r;

    return mat4x4<f32>(
        vec4<f32>(f, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, -1.0 / f, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, -r * r, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, -r * r * sin(theta) * sin(theta))
    );
}

fn sch_g_inv(pos: PackedPoint4) -> mat4x4<f32> {
    let r = pos.inner.y;
    let theta = pos.inner.z;
    let f = 1.0 - R_S / r;

    return mat4x4<f32>(
        vec4<f32>(1.0 / f, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, R_S / r - 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, -1.0 / (r * r), 0.0),
        vec4<f32>(0.0, 0.0, 0.0, -1.0 / (r * r * sin(theta) * sin(theta)))
    );
}

fn sch_del_g(pos: PackedPoint4, i: u32) -> mat4x4<f32> {
    let r = pos.inner.y;
    let theta = pos.inner.z;

    if (i == 1u) {
        let f = 1.0 - R_S / r;
        return mat4x4<f32>(
            vec4<f32>(R_S / (r * r), 0.0, 0.0, 0.0),
            vec4<f32>(0.0, R_S / (r * r * f * f), 0.0, 0.0),
            vec4<f32>(0.0, 0.0, -2.0 * r, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, -2.0 * r * sin(theta) * sin(theta))
        );
    }

    if (i == 2u) {
        return mat4x4<f32>(
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, -2.0 * r * r * cos(theta) * sin(theta))
        );
    }

    return zero_matrix();
}

fn sch_christoffel(pos: PackedPoint4, mu: u32, nu: u32, lambda: u32) -> f32 {
    let g_inv = sch_g_inv(pos);
    let d_mu_g = sch_del_g(pos, mu);
    let d_nu_g = sch_del_g(pos, nu);

    var gamma = 0.0;
    for (var alpha: u32 = 0u; alpha < 4u; alpha = alpha + 1u) {
        let d_alpha_g = sch_del_g(pos, alpha);
        gamma = gamma + 0.5 * g_inv[lambda][alpha] * (
            d_mu_g[alpha][nu] + d_nu_g[alpha][mu] - d_alpha_g[mu][nu]
        );
    }

    return gamma;
}

fn sch_step_along_null_geodesic(photon: PackedPhoton4) -> PackedPhoton4 {
    var del_k = four_vector_zero(photon.vel.vector_space);

    for (var mu: u32 = 0u; mu < 4u; mu = mu + 1u) {
        for (var alpha: u32 = 0u; alpha < 4u; alpha = alpha + 1u) {
            for (var beta: u32 = 0u; beta < 4u; beta = beta + 1u) {
                let gamma = sch_christoffel(photon.pos, alpha, beta, mu);
                del_k.inner[mu] = del_k.inner[mu] - gamma * photon.vel.inner[alpha] * photon.vel.inner[beta];
            }
        }
    }

    return euler_step(photon.pos, photon.vel, four_vector_as_point4(photon.vel), del_k);
}

fn evolve_schwarzschild_ray(photon: PackedPhoton4) -> PackedRayResult {
    var current = photon;

    for (var step: u32 = 0u; step < MAX_STEPS; step = step + 1u) {
        current = sch_step_along_null_geodesic(current);

        let world_pos = photon_to_world_pos(current);

        if (current.pos.inner.y <= R_S) {
            return PackedRayResult(current, HORIZON_HIT, vec3<u32>(0u));
        }

        if (length(world_pos - SUBATLAS_CENTER) > SCENE_SIZE) {
            return PackedRayResult(current, BACKGROUND_REACHED, vec3<u32>(0u));
        }

        let preferred_chart = preferred_chart_for_point(PackedPoint3(world_pos, CARTESIAN_WORLD));
        if (preferred_chart != current.pos.chart) {
            let world_vel = photon_to_world_vel(current);
            let world_photon = PackedPhoton3(PackedPoint3(world_pos, CARTESIAN_WORLD), PackedThreeVector(world_vel, CARTESIAN_WORLD));
            current = world_photon3_to_photon4(world_photon);
        }
    }

    return PackedRayResult(current, MAX_STEPS_REACHED, vec3<u32>(0u));
}

struct InputPhotons {
    data: array<PackedPhoton4>,
}

struct OutputResults {
    data: array<PackedRayResult>,
}

@group(0) @binding(0)
var<storage, read> input_photons: InputPhotons;

@group(0) @binding(1)
var<storage, read_write> output_results: OutputResults;

@compute @workgroup_size(64)
fn evolve_rays(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ray_index = global_id.x;
    let ray_count = arrayLength(&input_photons.data);

    if (ray_index >= ray_count) {
        return;
    }

    output_results.data[ray_index] = evolve_schwarzschild_ray(input_photons.data[ray_index]);
}