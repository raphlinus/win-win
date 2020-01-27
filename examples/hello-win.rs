use std::rc::Rc;
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HACCEL, HMENU, HWND};
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::winnt::LPCWSTR;
use winapi::um::winuser::{
    PostQuitMessage, LoadCursorW, LoadIconW, ShowWindow, CW_USEDEFAULT, IDC_ARROW, IDI_APPLICATION, SW_SHOWNORMAL, WM_DESTROY,
    WS_OVERLAPPEDWINDOW,
};

use win_win::{create_window, WindowClass, WindowProc, WindowState};

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
            unsafe { PostQuitMessage(0); }
        }
        None
    }
}

fn main() {
    unsafe {
        let icon = LoadIconW(0 as HINSTANCE, IDI_APPLICATION);
        let cursor = LoadCursorW(0 as HINSTANCE, IDC_ARROW);
        let brush = CreateSolidBrush(0xff_ff_ff);
        let win_class = WindowClass::new(
            0,
            0 as HINSTANCE,
            icon,
            cursor,
            brush,
            0 as LPCWSTR,
            "rust",
        )
        .unwrap();
        let window_state = WindowState::new(MyWindowProc);
        let hwnd = create_window(
            0,
            &win_class,
            "win-win example",
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            640,
            480,
            0 as HWND,
            0 as HMENU,
            0 as HINSTANCE,
            Rc::new(window_state),
        );
        ShowWindow(hwnd, SW_SHOWNORMAL);
        win_win::runloop(0 as HACCEL);
    }
}
