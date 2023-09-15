use hal::{
    prelude::*,
    systimer::SystemTimer,
    Delay,
    gpio::{InputPin, OutputPin},
};

///////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
/// Get distance to target in inches
pub(super) fn get_distance_as_inches<O: OutputPin, I: InputPin>(delay: &mut Delay, trigger: &mut O, echo: &I) -> u8 {
    
    // Set low to get a clean read
    trigger.set_output_high(false);
    delay.delay_us(5u8);

    // Set trigger high to start measurement.
    trigger.set_output_high(true);
    delay.delay_us(10u8);
    trigger.set_output_high(false);

    // TODO: Use pulse reader to measure clocks
    // Wait until pin goes high
    while !echo.is_input_high() {}

    // Start time clock count measurement
    let echo_start = SystemTimer::now();

    // Wait until pin goes low
    while echo.is_input_high() {}

    // Collect current timer count
    let echo_end = SystemTimer::now();

    // Calculate the elapsed timer count
    let echo_dur = echo_end.wrapping_sub(echo_start);

    // Calculate the distance in cms using formula in datasheet
    // SystemTimer is in clocks and with a 16MHz clock. 
    (echo_dur / 16 / 148) as u8

}
