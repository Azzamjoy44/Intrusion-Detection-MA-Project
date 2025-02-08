#![no_std]
#![no_main]

// Declare the hc_sr04 module, which contains the driver implementation for the HC-SR04 ultrasonic sensor.
pub mod hc_sr04;

// Import the necessary modules and types from the Embassy and other crates.
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::USB;
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::usb::{Driver, InterruptHandler as USBInterruptHandler};
use embassy_time::{Delay, Duration, Timer};
use log::info;
use panic_probe as _;
use core::str::FromStr;
use core::fmt::Write as FmtWrite;
use heapless::String;

// Import the HC-SR04 ultrasonic sensor driver implementation from the declared module.
use hc_sr04::HCSR04;

// I2C
use embassy_rp::i2c::{Config as I2cConfig, I2c, InterruptHandler as I2CInterruptHandler};
use embassy_rp::peripherals::I2C0;

// LCD1602 crates
use lcd1602_driver::command::State;
use lcd1602_driver::lcd::{self, Basic, Ext};
use lcd1602_driver::sender;
const DISPLAY_FREQUENCY: u32 = 100_000;

// Macro to bind specific interrupts to their handlers for the USB driver and LCD1602 I2C peripheral.
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => USBInterruptHandler<USB>;
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});

// Define an asynchronous task for logging over USB.
#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

// Helper function to convert f64 to heapless::String in order to display the distance on the LCD.
fn float_to_string(value: f64) -> String<64> {
    let mut s = String::new();
    write!(&mut s, "{:.2}", value).unwrap();
    s
}

// The main function of the application.
#[embassy_executor::main]
async fn main(spawner: Spawner) {

    let peripherals = embassy_rp::init(Default::default());
    let usb_driver = Driver::new(peripherals.USB, Irqs);
    spawner.spawn(logger_task(usb_driver)).unwrap();

    Timer::after_millis(1000).await;

    // Initialize the LCD 1602 I2C peripheral with the necessary configurations and the SDA and SCL pins.
    let sda = peripherals.PIN_16;
    let scl = peripherals.PIN_17;

    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());
    let mut sender = sender::I2cSender::new(&mut i2c, 0x27);
    let lcd_config = lcd::Config::default();
    let mut delayer = Delay;
    let mut lcd = lcd::Lcd::new(&mut sender, &mut delayer, lcd_config, DISPLAY_FREQUENCY);
    lcd.set_cursor_blink_state(State::Off);

    // Declare the necessary string variables in order to display the distance on the LCD.
    let mut str_unit: String<64>;
    let mut displayed_distance: String<32>;

    // Initialize the ultrasonic sensor with GPIO 21 and GPIO 20 for Trigger and Echo respectively.
    let mut ultrasonic = HCSR04::new(
        peripherals.PIN_21, // TRIGGER -> GPIO 21
        peripherals.PIN_20, // ECHO -> GPIO 20
    )
    .unwrap();

    let mut led = Output::new(peripherals.PIN_2, Level::Low); // The LED is connected to GPIO 2.

    // Create a PWM device for the buzzer
    let mut config_pwm: PwmConfig = Default::default();
    config_pwm.top = 0xFFFF;
    config_pwm.compare_b = 0;

    // Initialize PWM
    let mut buzzer = Pwm::new_output_b(peripherals.PWM_CH1, peripherals.PIN_3, config_pwm.clone()); // The buzzer is connected to GPIO 3.

    let mut sg90_1 = Output::new(peripherals.PIN_4, Level::Low); // Servo motor 1 is connected to GPIO 4.
    let mut sg90_2 = Output::new(peripherals.PIN_6, Level::Low); // Servo motor 2 is connected to GPIO 6.

    // Main loop of the application.
    loop {
        Timer::after_millis(200).await;

        // Attempt to measure distance with the ultrasonic sensor.
        let unit: f64;
        unit = match ultrasonic.measure().await {
            Ok(unit) => unit.centimeters,
            Err(_) => -1.0,
        };

        // Check if the measurement was successful.
        if unit < 0.0 {
            info!("Error in distance measurement");
            continue;
        }

        // If the measurement was successful, the application will proceed to control the LED, buzzer, and servo motors based on the distance measurement.

        // Control the LED based on the distance measurement.
        if unit <= 50.0 {
            led.set_high(); // Turn on the LED if the object is less than or equal to 60 cm away from the sensor.
        } else {
            led.set_low(); // Turn off the LED if the object is beyond 60 cm away from the sensor.
        }

        // Control the buzzer based on the distance measurement.
        if unit <= 35.0 {
            config_pwm.compare_b = config_pwm.top / 2; // Turn on the buzzer if the object is less than or equal to 45 cm away from the sensor.
        } else {
            config_pwm.compare_b = 0; // Turn off the buzzer if the object is beyond 45 cm away from the sensor.
        }
        buzzer.set_config(&config_pwm);

        // Control the servo motors based on the distance measurement.
        if unit <= 20.0 {

            // Rotate servo motor 1 90 degrees if the object is less than or equal to 30 cm away from the sensor.
            {
                sg90_1.set_high();
                Timer::after(Duration::from_millis(2)).await;
                sg90_1.set_low();
                Timer::after(Duration::from_millis(10)).await;
            }

            // Rotate servo motor 2 90 degrees in the opposite direction if the object is less than or equal to 30 cm away from the sensor.
            {
                sg90_2.set_high();
                Timer::after(Duration::from_millis(1)).await;
                sg90_2.set_low();
                Timer::after(Duration::from_millis(10)).await;
            }
        } else {

            // Rotate servo motor 1 back to its original position if the object is beyond 30 cm away from the sensor.
            {
                sg90_1.set_high();
                Timer::after(Duration::from_millis(1)).await;
                sg90_1.set_low();
                Timer::after(Duration::from_millis(10)).await;
            }
            
            // Rotate servo motor 2 back to its original position if the object is beyond 30 cm away from the sensor.
            {
                sg90_2.set_high();
                Timer::after(Duration::from_millis(2)).await;
                sg90_2.set_low();
                Timer::after(Duration::from_millis(10)).await;
            }
        }

        // Convert the measured distance to a string in order to display it on the LCD.
        str_unit = float_to_string(unit);
        displayed_distance = String::from_str("Distance: ").unwrap();
        displayed_distance.push_str(&mut str_unit).unwrap();

        // Log the successfully measured distance and display it on the LCD.
        info!("Distance: {:.2} cm", unit);
        lcd.set_cursor_blink_state(State::Off);
        lcd.clean_display();
        lcd.set_cursor_pos((0, 0));
        lcd.write_str_to_cur(&mut displayed_distance);
    }
}
