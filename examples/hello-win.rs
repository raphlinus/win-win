use std::ptr::null_mut;

use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::winuser::{
    LoadCursorW, LoadIconW, PostQuitMessage, ShowWindow, IDC_ARROW, IDI_APPLICATION, SW_SHOWNORMAL,
    WM_DESTROY, WS_OVERLAPPEDWINDOW,
};

use win_win::{WindowBuilder, WindowClass, WindowProc};

struct MyWindowProc;

impl WindowProc for MyWindowProc {
    fn window_proc(
        &self,
        _hwnd: HWND,
        msg: UINT,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Option<LRESULT> {
        println!("msg {}", msg);
        if msg == WM_DESTROY {
            unsafe {
                PostQuitMessage(0);
            }
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
        let hwnd = WindowBuilder::new(MyWindowProc, &win_class)
            .name("win-win example")
            .style(WS_OVERLAPPEDWINDOW)
            .build();
        ShowWindow(hwnd, SW_SHOWNORMAL);
        win_win::runloop(null_mut());
    }
}
