use std::time::Duration;

use libbpf_rs::skel::OpenSkel;
use libbpf_rs::skel::Skel;
use libbpf_rs::skel::SkelBuilder;
use libbpf_rs::RingBufferBuilder;
use plain::Plain;
mod sensor {
    include!(concat!(env!("OUT_DIR"), "/sensor.skel.rs"));
}

use sensor::*;

unsafe impl Plain for sensor_bss_types::exec_event {}

fn handle(raw: &[u8]) -> i32 {
    let mut event = sensor_bss_types::exec_event::default();
    plain::copy_from_bytes(&mut event, raw).expect("Data buffer was too short");
    let filename = event.filename.as_ptr();
    let filename = unsafe {std::ffi::CStr::from_ptr(filename).to_str().unwrap()};
    println!("{}: {}", event.pid, filename);
    0
}

fn bump_memlock_rlimit() -> Result<(), String> {
    let rlimit = libc::rlimit {
        rlim_cur: 128 << 20,
        rlim_max: 128 << 20,
    };

    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        return Err("Failed to increase rlimit".to_string())
    }

    Ok(())
}

fn main() {
    let skel_builder = SensorSkelBuilder::default();

    bump_memlock_rlimit().unwrap();
    let open_skel = skel_builder.open().unwrap();

    let mut skel = open_skel.load().unwrap();
    skel.attach().unwrap();

    let mut rbb = RingBufferBuilder::new();
    let mut binding = skel.maps_mut();
    rbb.add(binding.exec_rb(), handle).expect("Failed to add ringbuf");

    let rb = rbb.build().expect("Failed to build ringbuf");
    
    loop {
        rb.poll(Duration::from_millis(1)).unwrap();
    }
}