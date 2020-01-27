#![allow(non_snake_case)]

use std::ffi::OsStr;
use std::mem;
use std::rc::Rc;

use winapi::ctypes::c_int;
use winapi::shared::minwindef::{ATOM, DWORD, HINSTANCE, LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HCURSOR, HICON, HMENU, HWND};
use winapi::um::winnt::LPCWSTR;
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, RegisterClassW, SetWindowLongPtrW,
    CREATESTRUCTW, GWLP_USERDATA, WM_CREATE, WM_NCDESTROY, WNDCLASSW,
};

use wio::wide::ToWide;

use crate::error::Error;

pub trait WindowProc {
    // TODO: figure out signature of connect function

    /// The Rust-side implementation of the window procedure.
    fn window_proc(&self, hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM)
        -> Option<LRESULT>;
}

pub struct WindowClass {
    class_atom: ATOM,
}

pub struct WindowState {
    window_proc: Box<dyn WindowProc>,
}

impl WindowState {
    pub fn new(window_proc: impl WindowProc + 'static) -> WindowState {
        WindowState {
            window_proc: Box::new(window_proc),
        }
    }
}

#[cfg(target_arch = "x86_64")]
type WindowLongPtr = winapi::shared::basetsd::LONG_PTR;
#[cfg(target_arch = "x86")]
type WindowLongPtr = winapi::shared::ntdef::LONG;

unsafe extern "system" fn raw_window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        let create_struct = &*(lparam as *const CREATESTRUCTW);
        let window_state_ptr = create_struct.lpCreateParams;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_state_ptr as WindowLongPtr);
    }
    let window_state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const WindowState;
    let result = {
        if window_state_ptr.is_null() {
            None
        } else {
            (*window_state_ptr)
                .window_proc
                .window_proc(hwnd, msg, wparam, lparam)
        }
    };

    if msg == WM_NCDESTROY && !window_state_ptr.is_null() {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        mem::drop(Rc::from_raw(window_state_ptr));
    }
    result.unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
}

impl WindowClass {
    pub unsafe fn new(
        style: UINT,
        hInstance: HINSTANCE,
        hIcon: HICON,
        hCursor: HCURSOR,
        hbrBackground: HBRUSH,
        lpszMenuName: LPCWSTR,
        class_name: impl AsRef<OsStr>,
    ) -> Result<WindowClass, Error> {
        let class_name = class_name.to_wide_null();
        let wnd = WNDCLASSW {
            style,
            lpfnWndProc: Some(raw_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance,
            hIcon,
            hCursor,
            hbrBackground,
            lpszMenuName,
            lpszClassName: class_name.as_ptr(),
        };
        // TODO: probably should be RegisterClassExW so we can set small icon
        let class_atom = RegisterClassW(&wnd);
        if class_atom == 0 {
            // This should probably be GetLastError.
            Err(Error::RegisterClassFailed)
        } else {
            Ok(WindowClass { class_atom })
        }
    }
}

pub unsafe fn create_window(
    dwExStyle: DWORD,
    window_class: &WindowClass,
    window_name: impl AsRef<OsStr>,
    dwStyle: DWORD,
    x: c_int,
    y: c_int,
    nWidth: c_int,
    nHeight: c_int,
    hWndParent: HWND,
    hMenu: HMENU,
    hInstance: HINSTANCE,
    window_state: Rc<WindowState>,
) -> HWND {
    let window_name = window_name.to_wide_null();
    CreateWindowExW(
        dwExStyle,
        window_class.class_atom as LPCWSTR,
        window_name.as_ptr(),
        dwStyle,
        x,
        y,
        nWidth,
        nHeight,
        hWndParent,
        hMenu,
        hInstance,
        Rc::into_raw(window_state) as LPVOID,
    )
}
