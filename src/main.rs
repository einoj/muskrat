#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_time::{Delay, Timer};
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, OutputType, Pin, Pull, Speed};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::peripherals::{ADC2, PC0, PC1, PC2, PC3, PA0, PA1, PA2, TIM3, TIM4};
use embassy_stm32::rcc::{AdcClockSource, ClockSrc, Pll, PllMul, PllPreDiv, PllRDiv, PllSource};

use {defmt_rtt as _, panic_probe as _};

struct Detectors {
   mf_diode: PC3,
   lf_diode: PC0,
   l1_diode: PC1,
   l2_diode: PC2,
   rf_diode: PA2,
   r1_diode: PA1,
   r2_diode: PA0,
}

pub struct MotorDriver {
    pwm_r: SimplePwm<'static, TIM4>,
    pwm_l: SimplePwm<'static, TIM3>,
    speed_r: u16,
    speed_l: u16,
}

impl MotorDriver {
    pub fn new(pwm_r: SimplePwm<'static, TIM4>, pwm_l: SimplePwm<'static, TIM3>) -> Self {
        Self {
            pwm_r,
            pwm_l,
            speed_r: 0,
            speed_l: 0,
        }
    }

    pub fn set_right_speed(&mut self, speed: u16) {
        self.speed_r = speed;
        self.pwm_r.set_duty(Channel::Ch1, 0);
        self.pwm_r.set_duty(Channel::Ch2, self.speed_r);
    }

    pub fn set_left_speed(&mut self, speed: u16) {
        self.speed_l = speed;
        self.pwm_l.set_duty(Channel::Ch1, self.speed_l);
        self.pwm_l.set_duty(Channel::Ch2, 0);
    }

}

#[embassy_executor::task]
async fn motor_driver(mut pwm_r: SimplePwm<'static, TIM4>,
                      mut pwm_l: SimplePwm<'static, TIM3>,
                      speed_r: u16,
                      speed_l: u16) {

    loop {
        pwm_r.set_duty(Channel::Ch1, 0);
        pwm_r.set_duty(Channel::Ch2, speed_r);
        pwm_l.set_duty(Channel::Ch1, speed_l);
        pwm_l.set_duty(Channel::Ch2, 0);
    }
}

#[embassy_executor::task]
async fn diode_transducer(mut emitters: [Output<'static, AnyPin>; 6], mut detectors: Detectors, mut diode_adc: Adc<'static, ADC2>) {
    diode_adc.set_sample_time(SampleTime::Cycles24_5);
    loop {
        for e in &mut emitters {
            e.set_high();
        }
        let mf_diode = diode_adc.read(&mut detectors.mf_diode);
        let lf_diode = diode_adc.read(&mut detectors.lf_diode);
        let l1_diode = diode_adc.read(&mut detectors.l1_diode);
        let l2_diode = diode_adc.read(&mut detectors.l2_diode);
        let rf_diode = diode_adc.read(&mut detectors.rf_diode);
        let r1_diode = diode_adc.read(&mut detectors.r1_diode);
        let r2_diode = diode_adc.read(&mut detectors.r2_diode);
        info!("
            mf diode: {}
            lf diode: {} l1_diode: {} l2_diode: {}
            rf_diode: {} r1_diode: {} r2_diode: {}",
            mf_diode, lf_diode, l1_diode,
            l2_diode, rf_diode, r1_diode, r2_diode);
        Timer::after_millis(100).await;
        for e in &mut emitters {
            e.set_low();
        }
        Timer::after_millis(100).await;
    }

}

#[embassy_executor::main]
async fn main(spawner: Spawner) {

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
    let p = embassy_stm32::init(config);
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

    let emitters = [
        Output::new(p.PD4.degrade(), Level::High, Speed::High), //mf_emitter_1
        Output::new(p.PD5.degrade(), Level::High, Speed::High), //mf_emitter_2
        Output::new(p.PD7.degrade(), Level::High, Speed::High), //lf_emitter
        Output::new(p.PD6.degrade(), Level::High, Speed::High), //l_emitter
        Output::new(p.PD2.degrade(), Level::High, Speed::High), //rf_emitter
        Output::new(p.PD3.degrade(), Level::High, Speed::High), //r_emitter
    ];

    let detectors = Detectors{
       mf_diode: p.PC3,
       lf_diode: p.PC0,
       l1_diode: p.PC1,
       l2_diode: p.PC2,
       rf_diode: p.PA2,
       r1_diode: p.PA1,
       r2_diode: p.PA0,
    };

    let mut diode_adc = Adc::new(p.ADC2, &mut Delay);
    diode_adc.set_sample_time(SampleTime::Cycles24_5);

    spawner.spawn(diode_transducer(emitters, detectors, diode_adc)).ok();
    button.wait_for_rising_edge().await;
    let mut motor_driver = MotorDriver::new(pwm_r, pwm_l);
    loop {
        button.wait_for_rising_edge().await;
        motor_driver.set_left_speed(4500);
        motor_driver.set_right_speed(4500);
        Timer::after_millis(500).await;
        motor_driver.set_right_speed(0);
        motor_driver.set_left_speed(0);
    }

}
