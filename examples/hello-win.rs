#[allow(unused)]
use std::cell::RefCell;
use std::ptr::null_mut;

use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::winuser::{
    LoadCursorW, LoadIconW, PostQuitMessage, ShowWindow, IDC_ARROW, IDI_APPLICATION, SW_SHOWNORMAL,
    WM_CHAR, WM_DESTROY, WM_INPUTLANGCHANGE, WM_KEYDOWN, WM_KEYUP, WM_SYSCHAR, WM_SYSKEYDOWN,
    WM_SYSKEYUP, WS_OVERLAPPEDWINDOW,
};

#[cfg(feature = "kb")]
use win_win::KeyboardState;

use win_win::{WindowBuilder, WindowClass, WindowProc};

struct MyWindowProc {
    #[cfg(feature = "kb")]
    kb_state: RefCell<KeyboardState>,
}

impl WindowProc for MyWindowProc {
    #[allow(unused)]
    fn window_proc(
        &self,
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        match msg {
            WM_DESTROY => unsafe {
                PostQuitMessage(0);
            },
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP | WM_CHAR | WM_SYSCHAR
            | WM_INPUTLANGCHANGE => {
                #[cfg(feature = "kb")]
                if let Some(event) = unsafe {
                    self.kb_state
                        .borrow_mut()
                        .process_message(hwnd, msg, wparam, lparam)
                } {
                    println!("event: {:?}", event);
                    return Some(0);
                }
            }
            _ => (),
        }
        None
    }
}

fn main() {
    unsafe {
        let icon = LoadIconW(0 as HINSTANCE, IDI_APPLICATION);
        let cursor = LoadCursorW(0 as HINSTANCE, IDC_ARROW);
        let brush = CreateSolidBrush(0xff_ff_ff);
        let win_class = WindowClass::builder("rust")
            .icon(icon)
            .cursor(cursor)
            .background(brush)
            .build()
            .unwrap();
        let window_proc = MyWindowProc {
            #[cfg(feature = "kb")]
            kb_state: RefCell::new(KeyboardState::new()),
        };
        let hwnd = WindowBuilder::new(window_proc, &win_class)
            .name("win-win example")
            .style(WS_OVERLAPPEDWINDOW)
            .build();
        ShowWindow(hwnd, SW_SHOWNORMAL);
        win_win::runloop(null_mut());
    }
}
