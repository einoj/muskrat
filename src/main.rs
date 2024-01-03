#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_time::Timer;
//use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, Pin, Pull, Speed};
use embassy_stm32::exti::ExtiInput;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Muskrat OS!");

    // Configure the button pin (if needed) and obtain handler.
    // On the Nucleo FR401 there is a button connected to pin PC13.
    let button = Input::new(p.PD1, Pull::None);
    let mut button = ExtiInput::new(button, p.EXTI1);

    // Create and initialize a delay variable to manage delay loop
    let mut del_var = 2000;

    let chA: PwmPin<'_, embassy_stm32::peripherals::TIM3, embassy_stm32::timer::simple_pwm::Ch1> = PwmPin::new_ch1(p.PE3, OutputType::PushPull);
    //let chB: PwmPin<'_, embassy_stm32::peripherals::TIM3, embassy_stm32::timer::simple_pwm::Ch3> = PwmPin::new_ch3(p.PE5, OutputType::PushPull);
    let chC: PwmPin<'_, embassy_stm32::peripherals::TIM4, embassy_stm32::timer::simple_pwm::Ch2> = PwmPin::new_ch2(p.PD13, OutputType::PushPull);

    //let mut pwmA = SimplePwm::new(p.TIM3, Some(chA), None, Some(chB), None, khz(10), Default::default());
    let mut pwmA = SimplePwm::new(p.TIM3, Some(chA), None, None, None, khz(10), Default::default());
    let mut pwmB: SimplePwm<'_, embassy_stm32::peripherals::TIM4> = SimplePwm::new(p.TIM4, None, Some(chC), None, None, khz(10), Default::default());

    let maxA = pwmA.get_max_duty();
    let maxB = pwmB.get_max_duty();

    pwmA.enable(Channel::Ch1);
    pwmB.enable(Channel::Ch2);

    info!("PWM initialized");
    info!("PWM max duty {}", maxA);
//    let button = Input::new(p.PC13, Pull::Down);

    loop {
        button.wait_for_rising_edge().await;
        pwmA.set_duty(Channel::Ch1, maxA);
        pwmB.set_duty(Channel::Ch2, maxB);
        Timer::after_millis(1500).await;
        pwmA.set_duty(Channel::Ch1, 0);
        pwmB.set_duty(Channel::Ch2, 0);
        Timer::after_millis(1000).await;
    }
}
