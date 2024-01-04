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

    let button = Input::new(p.PD1, Pull::None);
    let mut button = ExtiInput::new(button, p.EXTI1);

    let ch_a_right = PwmPin::new_ch1(p.PD12, OutputType::PushPull);
    let ch_b_right = PwmPin::new_ch2(p.PD13, OutputType::PushPull);

    let mut pwm_r = SimplePwm::new(p.TIM4, Some(ch_a_right), Some(ch_b_right), None, None, khz(10), Default::default());
    let max_r = pwm_r.get_max_duty();

    pwm_r.enable(Channel::Ch1);
    pwm_r.enable(Channel::Ch2);

    let ch_a_left = PwmPin::new_ch1(p.PE3, OutputType::PushPull);
    let ch_b_left = PwmPin::new_ch2(p.PE4, OutputType::PushPull);
    let mut pwm_l = SimplePwm::new(p.TIM3, Some(ch_a_left), Some(ch_b_left), None, None, khz(10), Default::default());
    pwm_l.enable(Channel::Ch1);
    pwm_l.enable(Channel::Ch2);
    let max_l = pwm_l.get_max_duty();

    info!("PWM initialized");
    info!("Right PWM max duty {}", max_r);
    info!("Left PWM max duty {}", max_l);

    loop {
        button.wait_for_rising_edge().await;
        pwm_r.set_duty(Channel::Ch1, 0);
        pwm_r.set_duty(Channel::Ch2, max_r);
        pwm_l.set_duty(Channel::Ch1, max_l);
        pwm_l.set_duty(Channel::Ch2, 0);
        Timer::after_millis(500).await;
        pwm_r.set_duty(Channel::Ch1, max_r);
        pwm_r.set_duty(Channel::Ch2, 0);
        pwm_l.set_duty(Channel::Ch1, 0);
        pwm_l.set_duty(Channel::Ch2, max_l);
        Timer::after_millis(500).await;
        pwm_r.set_duty(Channel::Ch1, 0);
        pwm_l.set_duty(Channel::Ch2, 0);
    }
}
