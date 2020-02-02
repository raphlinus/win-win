use std::mem;
use std::ptr::null_mut;

use winapi::shared::minwindef::BOOL;
use winapi::shared::windef::HACCEL;
use winapi::um::winuser::{DispatchMessageW, GetMessageW, TranslateAcceleratorW, TranslateMessage};

/// A basic winapi runloop.
///
/// This runloop blocks on receiving messages and dispatches them to windows. It exits
/// on [`WM_QUIT`].
///
/// It is tempting to try to get fancier with runloops, for example waiting on semaphores
/// or other events, but these strategies are risky. In particular, the main runloop is not
/// always in control; when the window is being resized, or a modal dialog is open, then
/// that runloop takes precedence. For waking the UI thread from another thread,
/// [`SendMessage`] is probably the best bet.
///
/// # Safety
///
/// The `accel` argument must be a valid HACCEL handle (though `null_mut()` is valid).
///
/// [`WM_QUIT`]: https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-quit
/// [`SendMessage`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessage
pub unsafe fn runloop(accel: HACCEL) -> BOOL {
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
