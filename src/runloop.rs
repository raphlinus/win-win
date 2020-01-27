use std::mem;
use std::ptr::null_mut;

use winapi::shared::minwindef::BOOL;
use winapi::shared::windef::HACCEL;
use winapi::um::winuser::{DispatchMessageW, GetMessageW, TranslateAcceleratorW, TranslateMessage};

pub fn runloop(accel: HACCEL) -> BOOL {
    unsafe {
        loop {
            let mut msg = mem::MaybeUninit::uninit();
            let res = GetMessageW(msg.as_mut_ptr(), null_mut(), 0, 0);
            if res <= 0 {
                return res;
            }
            let mut msg = msg.assume_init();
            if accel.is_null() || TranslateAcceleratorW(msg.hwnd, accel, &mut msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
