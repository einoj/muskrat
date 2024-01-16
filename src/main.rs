#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_time::Delay;
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, OutputType, Pin, Pull, Speed};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::peripherals::{ADC2, PC0, PC1, PC2, PC3, PA0, PA1, PA2, TIM3, TIM4};
use embassy_stm32::rcc::{AdcClockSource, ClockSrc, Pll, PllMul, PllPreDiv, PllRDiv, PllSource};

use {defmt_rtt as _, panic_probe as _};

struct Detectors {
   mf_diode: PC3,
   _lf_diode: PC0,
   l1_diode: PC1,
   l2_diode: PC2,
   _rf_diode: PA2,
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

    pub fn go_forward(&mut self) {
        self.pwm_l.set_duty(Channel::Ch1, self.speed_l);
        self.pwm_l.set_duty(Channel::Ch2, 0);

        self.pwm_r.set_duty(Channel::Ch1, 0);
        self.pwm_r.set_duty(Channel::Ch2, self.speed_r);
    }

    pub fn turn_left(&mut self) {
        self.pwm_l.set_duty(Channel::Ch1, 0);
        self.pwm_l.set_duty(Channel::Ch2, self.speed_l);

        self.pwm_r.set_duty(Channel::Ch1, 0);
        self.pwm_r.set_duty(Channel::Ch2, self.speed_r);
    }

    pub fn turn_right(&mut self) {
        self.pwm_r.set_duty(Channel::Ch1, self.speed_l);
        self.pwm_r.set_duty(Channel::Ch2, 0);

        self.pwm_l.set_duty(Channel::Ch1, self.speed_r);
        self.pwm_l.set_duty(Channel::Ch2, 0);
    }

}

pub struct SensorReadings{
    front: u16,
    left: u16,
    right: u16,
}
pub struct Diodes {
    emitters: [Output<'static, AnyPin>; 6],
    detectors: Detectors,
    diode_adc: Adc<'static, ADC2>,
}

impl Diodes {
    fn new(emitters: [Output<'static, AnyPin>; 6], detectors: Detectors, diode_adc: Adc<'static, ADC2>) -> Self {
        Self {
            emitters,
            detectors,
            diode_adc,
        }
    }
    fn get_sensor_readings(&mut self) -> SensorReadings {
        self.diode_adc.set_sample_time(SampleTime::Cycles24_5);
        for e in &mut self.emitters {
            e.set_high();
        }

        let mut d = SensorReadings {front: 0, left: 0, right: 0};
        // ignore left and right front facing diodes for now
        d.front = self.diode_adc.read(&mut self.detectors.mf_diode);
        let l1_diode = self.diode_adc.read(&mut self.detectors.l1_diode);
        let l2_diode = self.diode_adc.read(&mut self.detectors.l2_diode);
        d.left = (l1_diode+l2_diode)/2;
        let r1_diode = self.diode_adc.read(&mut self.detectors.r1_diode);
        let r2_diode = self.diode_adc.read(&mut self.detectors.r2_diode);
        d.right = (r1_diode+r2_diode)/2;
        for e in &mut self.emitters {
            e.set_low();
        }
        return d;
    }
}

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
       _lf_diode: p.PC0,
       l1_diode: p.PC1,
       l2_diode: p.PC2,
       _rf_diode: p.PA2,
       r1_diode: p.PA1,
       r2_diode: p.PA0,
    };

    let mut diode_adc = Adc::new(p.ADC2, &mut Delay);
    diode_adc.set_sample_time(SampleTime::Cycles24_5);
    let mut diodes = Diodes::new(emitters, detectors, diode_adc);

    button.wait_for_rising_edge().await;
    let mut motor_driver = MotorDriver::new(pwm_r, pwm_l);
    motor_driver.set_left_speed(3250);
    motor_driver.set_right_speed(3000);
    loop {
        let sensor_data = diodes.get_sensor_readings();
        if sensor_data.left > 0 {
            motor_driver.turn_right();
        } else if sensor_data.right > 0 {
            motor_driver.turn_left();
        } else if sensor_data.front > 0 {
            motor_driver.turn_right();
        } else {
            motor_driver.go_forward();
        }
    }
}
