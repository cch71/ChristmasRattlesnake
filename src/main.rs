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
use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Event, Input, InputConfig, Io, Level, Output, OutputConfig, Pull},
    handler,
    i2c::master::I2c,
    main,
    rmt::Rmt,
    time::Rate,
};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use esp_println::println;

//CJRSLRB 5V 4 Channel SSR G3MB-202P Solid State Relay Module for Arduino Uno Duemilanove AVR Mega2560 Mega1280 ARM DSP PIC

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

////////////////////////////////////////////////////////////////////////////
/// Main entry point
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(gpio_irq_handler);

    // Setup onboard button to trigger interrupt
    {
        let config = InputConfig::default().with_pull(Pull::Up);
        let mut button = Input::new(peripherals.GPIO9, config);
        critical_section::with(|cs| {
            button.listen(Event::FallingEdge);
            BUTTON.borrow_ref_mut(cs).replace(button)
        });
    }

    // Push/Pull Outputs
    let ssr_channel1 = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let mut hcsr04_trigger = Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default());
    // Floating input
    let mut hcsr04_echo = Input::new(peripherals.GPIO7, InputConfig::default());
    let hcsr501_trigger = Input::new(peripherals.GPIO19, InputConfig::default());

    // Initialize the I2C bus for communicating with the LCD Display
    let mut i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    // Initialize the RattleController
    let mut snake = RattleControl::new(ssr_channel1);

    // Configure RMT LED peripheral globally
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    // We use one of the RMT channels to instantiate a `SmartLedsAdapter` which can
    // be used directly with all `smart_led` implementations
    let rmt_buffer = smartLedBuffer!(1);
    let led_adapter = SmartLedsAdapter::new(rmt.channel0, peripherals.GPIO8, rmt_buffer);
    let mut led = ColorControl::new(led_adapter);

    // Initialize the Delay peripheral, and use it to toggle the LED state in a loop.
    let mut delay = Delay::new();

    // Initialize the LM1602 Controller
    let mut lcd = LcdControl::new(&mut i2c, &mut delay).unwrap();

    // PIR doesn't start working until after a minute
    delay.delay_ms(60000);

    // Variables needed for the loop
    let mut prev_motion_detected = false;
    loop {
        // Iterate over the rainbow!
        led.next();

        // Check for motion
        let is_motion_detection = hcsr501_trigger.is_high();

        // Process motion detecion actions
        if is_motion_detection {
            if !prev_motion_detected && is_motion_detection {
                println!("Motion Detected\r");
            }

            // Get distance to whatever triggered motion
            let inches_to_target =
                get_distance_as_inches(&mut delay, &mut hcsr04_trigger, &mut hcsr04_echo);
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

        delay.delay_ms(20);
    }
}

///////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
/// Interrupt handler for gpio specifically the button for this
//#[interrupt(GPIO)]
#[handler]
fn gpio_irq_handler() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");

        if critical_section::with(|cs| {
            BUTTON
                .borrow_ref_mut(cs)
                .as_mut()
                .unwrap()
                .is_interrupt_set()
        }) {
            println!("Button was the source of the interrupt");
        } else {
            println!("Button was not the source of the interrupt");
        }

        // TODO: Something interesting...
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
