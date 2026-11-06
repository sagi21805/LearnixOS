use pci::PciDevice;
use vga_display::println;
use x86::{instructions::cpuid, memory_map::MemoryMap};

/// Prints the memory map, the PCI devices and the CPU's names.
pub fn print_hardware(mmap: &MemoryMap, pci_devices: &[PciDevice]) {
    println!("{}", mmap);

    for device in pci_devices {
        println!("{}", device);
    }
    println!("");

    print_cpu();
}

/// Prints the CPU's vendor and, if it reports one, its model name.
fn print_cpu() {
    let vendor = cpuid::get_vendor_string();
    // SAFETY: the vendor string is ASCII, so it is valid UTF-8.
    let vendor = unsafe { str::from_utf8_unchecked(&vendor) };
    println!("CPU vendor: {}", vendor);
    if let Some(brand) = cpuid::get_brand_string() {
        println!("CPU brand: {}", brand.as_str());
    }
}
