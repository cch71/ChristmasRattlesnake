// This project was originally started as a demonstration for my Engineering Merit Badge class I would
// give to the scouts.  It started as a C/C++ codebase targetting an Arduino Atmel ATMega 8 bit processor.
// With the release of the ESP32-C3-DevKitM-1 I felt like it was time to update it and modernize it with
// Rust.
//
// Since the Scouts are aways asking me where they could get the source code I am publishing it here.
//
// GPIO Ports used:
//   GPIO1,2 is I2C for LCD
//   GPIO5 is Channel 1 of SSR Relay
//   GPIO6 is HC-SR04 Trigger
//   GPIO7 is HC-SR04 Echo
//   GPIO8 is C3 WLED
//   GPIO9 is C3 Button
//   GPIO19 is HC-SR501

#![no_std]
#![no_main]
mod colorcontrol;
mod hc_sr04;
mod lcd_lm1602_i2c_driver;
mod lcdcontrol;
mod rattlercontrol;

use colorcontrol::ColorControl;
use hc_sr04::get_distance_as_inches;
use lcdcontrol::LcdControl;
use rattlercontrol::RattleControl;

use core::cell::RefCell;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};
use esp_println::println;
use hal::{
    clock::ClockControl,
    gpio::{Event, Gpio9, Input, InputPin, Pin, PullUp},
    i2c::I2C,
    interrupt,
    peripherals::{Interrupt, Peripherals},
    prelude::*,
    rmt::Rmt,
    timer::TimerGroup,
    Delay, Rtc, IO,
};

//CJRSLRB 5V 4 Channel SSR G3MB-202P Solid State Relay Module for Arduino Uno Duemilanove AVR Mega2560 Mega1280 ARM DSP PIC

static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));

////////////////////////////////////////////////////////////////////////////
///
#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let mut timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );

    rtc.swd.disable();
    rtc.rwdt.disable();
    timer_group0.wdt.disable();
    timer_group1.wdt.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let ssr_channel1 = io.pins.gpio5.into_push_pull_output();
    let mut hcsr04_trigger = io.pins.gpio6.into_push_pull_output();
    let hcsr04_echo = io.pins.gpio7.into_floating_input();
    let mut button = io.pins.gpio9.into_pull_up_input();
    let hcsr501_trigger = io.pins.gpio19.into_floating_input();

    // Initialize the I2C bus for communicating with the LCD Display
    let mut i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio1,
        io.pins.gpio2,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    // Initialize the RattleController
    let mut snake = RattleControl::new(ssr_channel1);

    // Setup onboard button to trigger interrupt
    button.listen(Event::FallingEdge); // raise interrupt on falling edge
    critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));
    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

    //Configure RGB LED
    // Configure RMT peripheral globally
    let rmt = Rmt::new(
        peripherals.RMT,
        80u32.MHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .unwrap();

    // We use one of the RMT channels to instantiate a `SmartLedsAdapter` which can
    // be used directly with all `smart_led` implementations
    let led_adapter = <smartLedAdapter!(0, 1)>::new(rmt.channel0, io.pins.gpio8);
    let mut led = ColorControl::new(led_adapter);

    // Initialize the Delay peripheral, and use it to toggle the LED state in a loop.
    let mut delay = Delay::new(&clocks);

    // Initialize the LM1602 Controller
    let mut lcd = LcdControl::new(&mut i2c, &mut delay).unwrap();

    // PIR doesn't start working until after a minute
    delay.delay_ms(60000u32);

    // Variables needed for the loop
    let mut prev_motion_detected = false;
    loop {
        // Iterate over the rainbow!
        led.next();

        // Check for motion
        let is_motion_detection = hcsr501_trigger.is_input_high();

        // Process motion detecion actions
        if is_motion_detection {
            if !prev_motion_detected && is_motion_detection {
                println!("Motion Detected\r");
            }

            // Get distance to whatever triggered motion
            let inches_to_target =
                get_distance_as_inches(&mut delay, &mut hcsr04_trigger, &hcsr04_echo);
            // println!("Dist to target: {:03}", inches_to_target);

            // Update the distance in the LCD
            lcd.update_distance(&mut i2c, &mut delay, inches_to_target)
                .unwrap();

            // Shake the rattler
            snake.set_distance_as_inches(Some(inches_to_target));
        } else {
            if prev_motion_detected && !is_motion_detection {
                println!("Motion Stopped\r");
                lcd.clear_distance(&mut i2c, &mut delay).unwrap();
            }
            // Stop rattling
            snake.set_distance_as_inches(None);
        };
        prev_motion_detected = is_motion_detection;

        // Determines if it is time to shake the rattle
        snake.next_maybe_rattle();

        delay.delay_ms(20u8);
    }
}

///////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
///
#[interrupt(GPIO)]
fn gpio_irq_handler() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");
        // TODO: Something interesting...
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
