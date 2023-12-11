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
use embassy_stm32::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Muskrat OS!");

    let chA = PwmPin::new_ch1(p.PB6, OutputType::PushPull);
    let chB = PwmPin::new_ch1(p.PB4, OutputType::PushPull);
    let mut pwmA = SimplePwm::new(p.TIM4, Some(chA), None, None, None, khz(10), Default::default());
    let mut pwmB = SimplePwm::new(p.TIM3, Some(chB), None, None, None, khz(10), Default::default());
    let maxA = pwmA.get_max_duty();
    let maxB = pwmB.get_max_duty();
    pwmA.enable(Channel::Ch1);
    pwmB.enable(Channel::Ch1);

    info!("PWM initialized");
    info!("PWM max duty {}", maxA);
//    let button = Input::new(p.PC13, Pull::Down);

    loop {
        pwmA.set_duty(Channel::Ch1, maxA / 2);
        pwmB.set_duty(Channel::Ch1, maxB / 2);
        Timer::after_millis(100).await;
    }
}
