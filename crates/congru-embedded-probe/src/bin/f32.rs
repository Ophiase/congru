#![cfg_attr(target_os = "none", no_std, no_main)]

#[cfg(target_os = "none")]
#[expect(unsafe_code, reason = "the linker looks for this exact symbol")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    congru_embedded_probe::run::<f32>()
}

#[cfg(not(target_os = "none"))]
fn main() -> Result<(), congru_embedded::Error> {
    core::hint::black_box(congru_embedded_probe::workout::<f32>()?);
    Ok(())
}
