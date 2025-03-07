use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
};

///////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
/// Get distance to target in inches
pub(super) fn get_distance_as_inches<O: OutputPin, I: InputPin, D: DelayNs>(
    delay: &mut D,
    trigger: &mut O,
    echo: &mut I,
) -> u8 {
    // Set low vget a clean read
    trigger.set_low().unwrap();
    delay.delay_us(5);

    // Set trigger high to start measurement.
    trigger.set_high().unwrap();
    delay.delay_us(10);
    trigger.set_low().unwrap();

    // TODO: Use pulse reader to measure clocks
    // Wait until pin goes high
    while !echo.is_high().unwrap() {}

    // Start time measurement

    let echo_start = esp_hal::time::Instant::now();

    // Wait until pin goes low
    while echo.is_high().unwrap() {}

    use core::ops::Sub;
    // Collect current timer count and get duration
    let echo_dur = echo_start.sub(esp_hal::time::Instant::now());

    // Calculate the distance in uS/58=cm using formula in datasheet
    // echo_dur is in microsecond precision.
    (echo_dur.as_micros() / 58) as u8
}
