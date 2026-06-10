// Diagnostic stub: the daemon arrives in a later experiment of issue 0002.
// For now this binary reports what the toolchain proof needs to know.

fn main() {
    println!("nutorchd PoC skeleton (issue 0002)");
    println!("tch crate: 0.24.0 (pairs with libtorch v2.11.0)");
    println!("MPS available: {}", tch::utils::has_mps());
    println!("CUDA available: {}", tch::Cuda::is_available());
}
