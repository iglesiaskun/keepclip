//! Sizes, speeds and times as people read them.

use keepclip_core::units::{bytes, duration, speed};

#[test]
fn sizes_use_the_unit_that_fits() {
    assert_eq!(bytes(0), "0 bytes");
    assert_eq!(bytes(820), "820 bytes");
    assert_eq!(bytes(1024), "1 KB");
    assert_eq!(bytes(524_288), "512 KB");
    assert_eq!(bytes(50_541_363), "48.2 MB");
    assert_eq!(bytes(1_331_439_861), "1.24 GB");
}

#[test]
fn speeds_are_sizes_per_second() {
    assert_eq!(speed(2_516_582.4), "2.4 MB/s");
    assert_eq!(speed(0.5), "0 bytes/s");
}

#[test]
fn durations_show_hours_only_when_needed() {
    assert_eq!(duration(0), "0:00");
    assert_eq!(duration(42), "0:42");
    assert_eq!(duration(185), "3:05");
    assert_eq!(duration(3723), "1:02:03");
}
