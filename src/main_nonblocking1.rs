#![no_main]
#![no_std]
use cortex_m_rt::entry;
use panic_halt as _;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use microbit::{
    board::Board,
    display::nonblocking::{Display, GreyscaleImage},
    pac::{self, interrupt, TIMER1},
};

static DISPLAY: Mutex<RefCell<Option<Display<TIMER1>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let Some(mut board) = Board::take() {
        // ディスプレイを初期化する
        let mut display = Display::new(board.TIMER1, board.display_pins);
        display.show(&create_image(9,7,1));
        cortex_m::interrupt::free(move |cs| {
            *DISPLAY.borrow(cs).borrow_mut() = Some(display);
        });
        // 優先度の設定をした後割り込みを許可する。
        unsafe {
            board.NVIC.set_priority(pac::Interrupt::TIMER1, 64);
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