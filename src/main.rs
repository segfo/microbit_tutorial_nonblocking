// #![deny(unsafe_code)]
#![no_main]
#![no_std]
use cortex_m_rt::entry;
// use defmt_rtt as _;
// use panic_rtt_target as _;
// use rtt_target::rtt_init_print;
use panic_halt as _;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::Peripherals;

use microbit::{
    board::Board,
    display::nonblocking::{Display, GreyscaleImage},
    hal::{
        clocks::Clocks,
        rtc::{Rtc, RtcInterrupt},
    },
    pac::{self, interrupt, RTC0, TIMER1},
};

// We use TIMER1 to drive the display, and RTC0 to update the animation.
// We set the TIMER1 interrupt to a higher priority than RTC0.

static DISPLAY: Mutex<RefCell<Option<Display<TIMER1>>>> = Mutex::new(RefCell::new(None));
static ANIM_TIMER: Mutex<RefCell<Option<Rtc<RTC0>>>> = Mutex::new(RefCell::new(None));




#[entry]
fn main() -> ! {
    if let Some(mut board) = Board::take() {
        // RTCに必要な低周波ハードウェアクロックを起動する
        Clocks::new(board.CLOCK).start_lfclk();

        // RTC を 16Hz (32_768 / (2047 + 1))で設定する
        // 62.5ms 毎に割り込みが発生するようになる
        let mut rtc0 = Rtc::new(board.RTC0, 2047).unwrap();
        rtc0.enable_event(RtcInterrupt::Tick);
        rtc0.enable_interrupt(RtcInterrupt::Tick, None);
        rtc0.enable_counter();

        // ディスプレイを初期化する
        let mut display = Display::new(board.TIMER1, board.display_pins);
        // display.show(&create_image(9,7,1));
        cortex_m::interrupt::free(move |cs| {
            *DISPLAY.borrow(cs).borrow_mut() = Some(display);
            *ANIM_TIMER.borrow(cs).borrow_mut() = Some(rtc0);
        });
        // 優先度の設定をした後割り込みを許可する。
        unsafe {
            board.NVIC.set_priority(pac::Interrupt::RTC0, 128);
            board.NVIC.set_priority(pac::Interrupt::TIMER1, 64);
            pac::NVIC::unmask(pac::Interrupt::RTC0);
            pac::NVIC::unmask(pac::Interrupt::TIMER1);
        }
    }

    loop {}
}

#[interrupt]
fn TIMER1() {
    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.handle_display_event();
        }
    });
}

fn create_image(b: u8,b1: u8,b2: u8) -> GreyscaleImage {
    GreyscaleImage::new(&[
        [b, b, b, b, b],
        [b, b1, b1, b1, b],
        [b, b1, b2, b1, b],
        [b, b1, b1, b1, b],
        [b, b, b, b, b],
    ])
}

fn square_image(b: u8) -> GreyscaleImage {
    let b1=b;
    let b2=b;
    GreyscaleImage::new(&[
        [b, b, b, b, b],
        [b, b1, b1, b1, b],
        [b, b1, b2, b1, b],
        [b, b1, b1, b1, b],
        [b, b, b, b, b],
    ])
}

#[interrupt]
unsafe fn RTC0() {
    static mut STEP: u8 = 0;

    cortex_m::interrupt::free(|cs| {
        if let Some(rtc) = ANIM_TIMER.borrow(cs).borrow_mut().as_mut() {
            rtc.reset_event(RtcInterrupt::Tick);
        }
    });

    let inner_brightness = match *STEP {
        0..=8 => 9 - *STEP,
        9..=17=> *STEP-8,
        _ => 0,
    };

    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.show(&square_image(inner_brightness));
        }
    });

    *STEP += 1;
    if *STEP == 18 {
        *STEP = 0
    };
}
