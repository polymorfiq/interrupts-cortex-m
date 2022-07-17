#![no_std]
#![no_main]

use core::panic::PanicInfo;
use arduino_nano33iot as bsp;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use bsp::hal::gpio;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use hal::pac::{interrupt};

struct Machine {
    led: gpio::v2::Pin<gpio::v2::PA17, gpio::v2::Output<gpio::v2::PushPull>>,
    ext_led: gpio::v2::Pin<gpio::v2::PA21, gpio::v2::Output<gpio::v2::PushPull>>,
    tc3: hal::timer::TimerCounter3,
    tc4: hal::timer::TimerCounter4
}

impl Machine {
    fn new() -> Self {
        let mut peripherals = Peripherals::take().unwrap();
        let mut core = CorePeripherals::take().unwrap();
        
        let mut clocks = GenericClockController::with_internal_32kosc(
            peripherals.GCLK,   
            &mut peripherals.PM,
            &mut peripherals.SYSCTRL,
            &mut peripherals.NVMCTRL,
        );
        let pins = bsp::Pins::new(peripherals.PORT);
        let led: bsp::Led = pins.led_sck.into();
        let ext_led = pins.d10.into();

        let gclk0 = clocks.gclk0();
        let tc3 = hal::timer::TimerCounter::tc3_(
            &clocks.tcc2_tc3(&gclk0).unwrap(),
            peripherals.TC3,
            &mut peripherals.PM,
        );

        let tc4 = hal::timer::TimerCounter::tc4_(
            &clocks.tc4_tc5(&gclk0).unwrap(),
            peripherals.TC4,
            &mut peripherals.PM,
        );

        unsafe {
            // Enable the interrupt line for the timer
            NVIC::unmask(hal::pac::Interrupt::TC3);
            NVIC::unmask(hal::pac::Interrupt::TC4);

            // Set priority for the timer interrupt
            core.NVIC.set_priority(hal::pac::Interrupt::TC3, 1);
            core.NVIC.set_priority(hal::pac::Interrupt::TC4, 2);
        }

        Machine{
            led,
            ext_led,
            tc3,
            tc4
        }
    }
}

static mut MACHINE: Option<Machine> = None;

#[entry]
fn main() -> ! {
    let machine = unsafe {
        MACHINE = Some(Machine::new());
        MACHINE.as_mut().unwrap()
    };

    cortex_m::interrupt::free(|_cs| {
        machine.led.set_low().unwrap();
        machine.ext_led.set_low().unwrap();
    });

    machine.tc3.start(2.hz());
    machine.tc3.enable_interrupt();

    machine.tc4.start(10.hz());
    machine.tc4.enable_interrupt();

    loop {
        cortex_m::asm::wfi();
    }
}

#[interrupt]
fn TC3() {
    let machine = unsafe { MACHINE.as_mut().unwrap() };
    machine.led.toggle().unwrap();

    // Unset the timer so it can fire again
    machine.tc3.wait().unwrap();
}

#[interrupt]
fn TC4() {
    let machine = unsafe { MACHINE.as_mut().unwrap() };
    machine.ext_led.toggle().unwrap();

    // Unset the timer so it can fire again
    machine.tc4.wait().unwrap();
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}