#![no_std]
#![no_main]

use core::panic::PanicInfo;
use arduino_nano33iot as bsp;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use bsp::hal::gpio;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use hal::pac::{interrupt};

struct Machine {
    led: gpio::v2::Pin<gpio::v2::PA17, gpio::v2::Output<gpio::v2::PushPull>>,
    ext_led: gpio::v2::Pin<gpio::v2::PA21, gpio::v2::Output<gpio::v2::PushPull>>,
    delay: Delay,
    nvic: NVIC
}


impl Machine {
    fn new() -> Self {
        let mut peripherals = Peripherals::take().unwrap();
        let core = CorePeripherals::take().unwrap();
        let nvic = core.NVIC;
        
        let mut clocks = GenericClockController::with_internal_32kosc(
            peripherals.GCLK,   
            &mut peripherals.PM,
            &mut peripherals.SYSCTRL,
            &mut peripherals.NVMCTRL,
        );
        let pins = bsp::Pins::new(peripherals.PORT);
        let led: bsp::Led = pins.led_sck.into();
        let ext_led = pins.d10.into_push_pull_output();
        let delay = Delay::new(core.SYST, &mut clocks);

        Machine{
            led,
            ext_led,
            delay,
            nvic
        }
    }
}

static mut MACHINE: Option<Machine> = None;
static mut INTERRUPT_COUNT: u32 = 0;

#[entry]
fn main() -> ! {
    let machine = unsafe {
        MACHINE = Some(Machine::new());
        MACHINE.as_mut().unwrap()
    };

    unsafe {
        NVIC::unmask(hal::pac::Interrupt::I2S);
        machine.nvic.set_priority(hal::pac::Interrupt::I2S, 1 << 0);
    }

    loop {
        machine.delay.delay_ms(500u16);

        // Fire I2S() and then come back to this loop.
        NVIC::pend(hal::pac::Interrupt::I2S);
    }
}

#[interrupt]
fn I2S() {
    let machine = unsafe { MACHINE.as_mut().unwrap() };
    let i = unsafe { INTERRUPT_COUNT };

    if i % 5 == 0 {
        machine.led.set_high().unwrap();
    } else {
        machine.led.set_low().unwrap();
    }

    unsafe { INTERRUPT_COUNT += 1;}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}