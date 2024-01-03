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
use embassy_stm32::gpio::{Input, Pull};
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
    let _del_var = 2000;

    let ch_a = PwmPin::new_ch1(p.PE3, OutputType::PushPull);
    let ch_b = PwmPin::new_ch2(p.PD13, OutputType::PushPull);

    let mut pwm_a = SimplePwm::new(p.TIM3, Some(ch_a), None, None, None, khz(10), Default::default());
    let mut pwm_b = SimplePwm::new(p.TIM4, None, Some(ch_b), None, None, khz(10), Default::default());

    let max_a = pwm_a.get_max_duty();
    let max_b = pwm_b.get_max_duty();

    pwm_a.enable(Channel::Ch1);
    pwm_b.enable(Channel::Ch2);

    info!("PWM initialized");
    info!("PWM max duty {}", max_a);
//    let button = Input::new(p.PC13, Pull::Down);

    loop {
        button.wait_for_rising_edge().await;
        pwm_a.set_duty(Channel::Ch1, max_a);
        pwm_b.set_duty(Channel::Ch2, max_b);
        Timer::after_millis(1500).await;
        pwm_a.set_duty(Channel::Ch1, 0);
        pwm_b.set_duty(Channel::Ch2, 0);
        Timer::after_millis(1000).await;
    }
}
