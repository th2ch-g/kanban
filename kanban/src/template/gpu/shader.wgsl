// Burn GPU cycles so the process ranks high enough to be visible in nvtop.
//
// The loop is bounded on purpose. An unbounded one never lets the queue drain,
// and dropping the device then blocks forever; the host re-dispatches this
// shader until its own deadline instead. The result is written to a storage
// buffer so the arithmetic has an observable effect - a finite loop whose
// output nothing reads is free to be optimised away entirely.
@group(0) @binding(0)
var<storage, read_write> sink: array<f32>;

const ITERATIONS: u32 = 4194304u;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var tmpvec: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
    let tmpmat: mat3x3<f32> = mat3x3<f32>(0.707107, -0.707107, 0.0, 0.707107, 0.707107, 0.0, 0.0, 0.0, 1.0);
    for (var i: u32 = 0u; i < ITERATIONS; i = i + 1u) {
        tmpvec = tmpmat * tmpvec;
    }
    sink[global_id.x] = tmpvec.x + tmpvec.y + tmpvec.z;
}
