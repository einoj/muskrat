#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_time::{Delay, Timer};
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::rcc::{AdcClockSource, ClockSrc, Pll, PllMul, PllPreDiv, PllRDiv, PllSource};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let mut config = Config::default();
    config.rcc.hsi = true;
    config.rcc.mux = ClockSrc::PLL1_R;
    config.rcc.adc_clock_source = AdcClockSource::SYS;
    config.rcc.pll = Some(Pll {
        // 64Mhz clock (16 / 1 * 8 / 2)
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL8,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::DIV2),
    });
    let mut p = embassy_stm32::init(config);
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

    //ADC12_IN7
    let mut mf_emitter_1 = Output::new(p.PD4, Level::High, Speed::High);
    let mut mf_emitter_2 = Output::new(p.PD5, Level::High, Speed::High);
    let mut lf_emitter = Output::new(p.PD7, Level::High, Speed::High);
    let mut l_emitter = Output::new(p.PD6, Level::High, Speed::High);
    let mut rf_emitter = Output::new(p.PD2, Level::High, Speed::High);
    let mut r_emitter = Output::new(p.PD3, Level::High, Speed::High);
    mf_emitter_1.set_high();
    mf_emitter_2.set_high();
    lf_emitter.set_high();
    l_emitter.set_high();
    rf_emitter.set_high();
    r_emitter.set_high();
    let mut diode_adc = Adc::new(p.ADC2, &mut Delay);
    diode_adc.set_sample_time(SampleTime::Cycles24_5);
    loop {
        let mf_diode = diode_adc.read(&mut p.PC3);
        let lf_diode = diode_adc.read(&mut p.PC0);
        let l1_diode = diode_adc.read(&mut p.PC1);
        let l2_diode = diode_adc.read(&mut p.PC2);
        let rf_diode = diode_adc.read(&mut p.PA2);
        let r1_diode = diode_adc.read(&mut p.PA1);
        let r2_diode = diode_adc.read(&mut p.PA0);
        info!("
            mf diode: {}
            lf diode: {} l1_diode: {} l2_diode: {}
            rf_diode: {} r1_diode: {} r2_diode: {}",
            mf_diode, lf_diode, l1_diode,
            l2_diode, rf_diode, r1_diode, r2_diode);
        Timer::after_millis(200).await;
    }

    /*
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
    */
}
