@workgroup_size(1)
@compute
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var tmpvec: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
    let tmpmat: mat3x3<f32> = mat3x3<f32>(0.707107, -0.707107, 0.0, 0.707107, 0.707107, 0.0, 0.0, 0.0, 1.0);
    loop {
        tmpvec = tmpmat * tmpvec;
    }
}
