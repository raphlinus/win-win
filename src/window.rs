#![allow(non_snake_case)]

use std::ffi::OsStr;
use std::mem;
use std::ptr::{null, null_mut};
use std::rc::Rc;

use winapi::ctypes::c_int;
use winapi::shared::minwindef::{ATOM, DWORD, HINSTANCE, LPARAM, LPVOID, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HCURSOR, HICON, HMENU, HWND};
use winapi::um::winnt::LPCWSTR;
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, RegisterClassExW, SetWindowLongPtrW,
    CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, WM_CREATE, WM_NCDESTROY, WNDCLASSEXW,
};

use wio::wide::ToWide;

use crate::error::Error;

/// A Rust wrapper for the winapi "window procedure".
///
/// See the Microsoft documentation on [Window Procedures] for more information. The details of
/// the window procedure are up to the application, though this wrapper does a bit of lifetime
/// management for the trait object, dropping it on [`WM_NCDESTROY`].
///
/// The window procedure will only be called from the message loop of the thread on which it was
/// created, which is why there is no `Sync` or `Send` bound on the trait object. However, it is
/// definitely possible for it to be called [reentrantly], which is a primary reason the method is
/// `&self`. Common ways to observe reentrant calls include:
///
/// * Calling [`DestroyWindow`].
///
/// * Calling [`SendMessage`].
///
/// * Calling a synchronous dialog, including a file dialog.
///
/// [Window Procedures]: https://docs.microsoft.com/en-us/windows/win32/winmsg/window-procedures
/// [`DestroyWindow`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow
/// [`SendMessage`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessage
/// [reentrantly]: https://www-user.tu-chemnitz.de/~heha/viewchm.php/hs/petzold.chm/petzoldi/ch03c.htm
/// [`WM_NCDESTROY`]: https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-ncdestroy
pub trait WindowProc {
    /// The Rust-side implementation of the window procedure.
    ///
    /// When return value is `None`, [`DefWindowProc`] is called.
    ///
    /// [`DefWindowProc`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-defwindowprocw
    fn window_proc(&self, hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM)
        -> Option<LRESULT>;
}

/// A window class.
pub enum WindowClass {
    Atom(ATOM),
    Name(Vec<u16>),
}

/// A builder for registering new window classes.
pub struct WindowClassBuilder {
    style: UINT,
    cbWndExtra: c_int,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    menu_name: Vec<u16>,
    class_name: Vec<u16>,
    hIconSm: HICON,
}

/// A builder for creating new windows.
pub struct WindowBuilder<'a> {
    window_proc: Rc<Box<dyn WindowProc>>,
    dwExStyle: DWORD,
    window_class: &'a WindowClass,
    window_name: Vec<u16>,
    dwStyle: DWORD,
    x: c_int,
    y: c_int,
    nWidth: c_int,
    nHeight: c_int,
    hWndParent: HWND,
    hMenu: HMENU,
    hInstance: HINSTANCE,
}

impl<'a> WindowBuilder<'a> {
    /// Create a new window builder.
    ///
    /// The window procedure and window class are set here.
    ///
    /// Discussion question: would it ever make sense to create a window
    /// without a window procedure?
    pub fn new(
        window_proc: impl WindowProc + 'static,
        window_class: &WindowClass,
    ) -> WindowBuilder {
        WindowBuilder {
            window_proc: Rc::new(Box::new(window_proc)),
            dwExStyle: 0,
            window_class,
            window_name: Vec::new(),
            dwStyle: 0,
            x: CW_USEDEFAULT,
            y: CW_USEDEFAULT,
            nWidth: CW_USEDEFAULT,
            nHeight: CW_USEDEFAULT,
            hWndParent: null_mut(),
            hMenu: null_mut(),
            hInstance: null_mut(),
        }
    }

    /// Build a window.
    ///
    /// The return value is the HWND for the window, or 0 on error.
    ///
    /// The lifetime of the window is until `WM_NCDESTROY` is called,
    /// at which point the window procedure is dropped.
    ///
    /// [`WM_NCDESTROY`]: https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-ncdestroy
    pub fn build(self) -> HWND {
        unsafe {
            let wnd_proc_ptr = Rc::into_raw(self.window_proc) as LPVOID;
            let hwnd = CreateWindowExW(
                self.dwExStyle,
                self.window_class.as_lpcwstr(),
                pointer_or_null(&self.window_name),
                self.dwStyle,
                self.x,
                self.y,
                self.nWidth,
                self.nHeight,
                self.hWndParent,
                self.hMenu,
                self.hInstance,
                wnd_proc_ptr,
            );
            if hwnd.is_null() {
                std::mem::drop(Rc::from_raw(wnd_proc_ptr));
            }
            hwnd
        }
    }

    /// Set the window name.
    ///
    /// This becomes the `lpWindowName` parameter to [`CreateWindowEx`].
    ///
    /// [`CreateWindowEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    pub fn name(mut self, name: impl AsRef<OsStr>) -> Self {
        self.window_name = name.to_wide_null();
        self
    }

    /// Set the window style.
    ///
    /// The argument is the bitwise OR of a number of `WS_` values from the [Window Styles] enumeration.
    /// It becomes the `dwStyle` parameter to [`CreateWindowEx`].
    ///
    /// [`CreateWindowEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    /// [Window Styles]: https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles
    pub fn style(mut self, style: DWORD) -> Self {
        self.dwStyle = style;
        self
    }

    /// Set the extended window style.
    ///
    /// The argument is the bitwise OR of a number of `WS_EX` values from the [Extended Window Styles] enumeration.
    /// It becomes the `dwExStyle` parameter to [`CreateWindowEx`].
    ///
    /// An interesting parameter is `WS_EX_NOREDIRECTIONBITMAP`, which disables the redirection bitmap.
    /// It is useful to set when the window will contain a swapchain and no GDI content (in particular, no
    /// menus). There is a particular source of artifacting on window resize that is reduced when the
    /// redirection bitmap is disabled. It should almost always be set when using DirectComposition,
    /// see this [article by Kenny Kerr].
    ///
    /// [`CreateWindowEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    /// [Extended Window Styles]: https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
    /// [article by Kenny Kerr]: https://docs.microsoft.com/en-us/archive/msdn-magazine/2014/june/windows-with-c-high-performance-window-layering-using-the-windows-composition-engine
    pub fn ex_style(mut self, style: DWORD) -> Self {
        self.dwExStyle = style;
        self
    }

    /// Set the window position.
    ///
    /// The arguments become the `x` and `y` parameters to [`CreateWindowEx`]. To set one but not the other,
    /// use `CW_USEDEFAULT`. These are in raw pixel values.
    ///
    /// The position is relative to the top left corner of the primary monitor. See [`EnumDisplayMonitors`]
    /// for more information about multiple monitors.
    ///
    /// [`CreateWindowEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    /// [`EnumDisplayMonitors`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors
    pub fn position(mut self, x: c_int, y: c_int) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the window size.
    ///
    /// The arguments become the `nWidth` and `nHeight` parameters to [`CreateWindowEx`]. To set one but not
    /// the other, use `CW_USEDEFAULT`. These are in raw pixel values.
    ///
    /// [`CreateWindowEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    pub fn size(mut self, width: c_int, height: c_int) -> Self {
        self.nWidth = width;
        self.nHeight = height;
        self
    }

    /// Set the parent window.
    ///
    /// The argument becomes the `hWndParent` parameter to [`CreateWindowEx`].
    ///
    /// # Safety
    ///
    /// The argument must be a valid HWND reference.
    ///
    /// [`CreateWindowEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    pub unsafe fn parent_hwnd(mut self, parent: HWND) -> Self {
        self.hWndParent = parent;
        self
    }

    /// Set the menu.
    ///
    /// The argument becomes the `hMenu` parameter to [`CreateWindowEx`].
    ///
    /// # Safety
    ///
    /// The argument must be a valid HMENU reference.
    ///
    /// [`CreateWindowEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    pub unsafe fn menu(mut self, menu: HMENU) -> Self {
        self.hMenu = menu;
        self
    }

    /// Set the instance handle.
    ///
    /// The argument becomes the `hInstance` parameter to [`CreateWindowEx`].
    ///
    /// Instance handles are a namespace mechanism, so that components (in a DLL, for example) don't
    /// interfere with each other. For a top-level application, it is safe to leave this unset.
    ///
    /// [Raymond Chen's blog](https://devblogs.microsoft.com/oldnewthing/20040614-00/?p=38903) has
    /// a bit of information about HINSTANCE, including its historical distinction from HMODULE
    /// (they are now the same).
    ///
    /// # Safety
    ///
    /// The argument must be a valid HINSTANCE reference.
    ///
    /// [`CreateWindowEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw
    pub unsafe fn instance(mut self, instance: HINSTANCE) -> Self {
        self.hInstance = instance;
        self
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
    let window_proc_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Box<dyn WindowProc>;
    let result = {
        if window_proc_ptr.is_null() {
            None
        } else {
            // Hold a reference for the duration of the call, in case there's a
            // reentrant call to WM_NCDESTROY (as would happen if the window
            // procedure called DestroyWindow).
            let reference = Rc::from_raw(window_proc_ptr);
            mem::forget(reference.clone());
            (*window_proc_ptr).window_proc(hwnd, msg, wparam, lparam)
        }
    };

    if msg == WM_NCDESTROY && !window_proc_ptr.is_null() {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        mem::drop(Rc::from_raw(window_proc_ptr));
    }
    result.unwrap_or_else(|| DefWindowProcW(hwnd, msg, wparam, lparam))
}

impl WindowClass {
    /// A builder for creating a new window class.
    ///
    /// The class name should be unique, otherwise creation will fail.
    pub fn builder(class_name: impl AsRef<OsStr>) -> WindowClassBuilder {
        WindowClassBuilder {
            class_name: class_name.to_wide_null(),
            style: 0,
            cbWndExtra: 0,
            hInstance: null_mut(),
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            menu_name: Vec::new(),
            hIconSm: null_mut(),
        }
    }

    /// Create a window class reference from a name.
    /// 
    /// This function is useful if the window class has already been registered, either
    /// through a successful builder or some other means.
    pub fn from_name(class_name: impl AsRef<OsStr>) -> WindowClass {
        WindowClass::Name(class_name.to_wide_null())
    }

    fn as_lpcwstr(&self) -> LPCWSTR {
        match self {
            WindowClass::Atom(atom) => *atom as LPCWSTR,
            WindowClass::Name(name) => name.as_ptr(),
        }
    }
}

impl WindowClassBuilder {
    /// Create the window class.
    ///
    /// Note: the window class is leaked, as its lifetime is most commonly that of
    /// the application. Somebody who really wants to reclaim that memory can call
    /// [`UnregisterClass`] manually and deal with the soundness consequences.
    ///
    /// [`UnregisterClass`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unregisterclassw
    pub fn build(self) -> Result<WindowClass, Error> {
        unsafe {
            let wnd = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: self.style,
                lpfnWndProc: Some(raw_window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: self.hInstance,
                hIcon: self.hIcon,
                hCursor: self.hCursor,
                hbrBackground: self.hbrBackground,
                lpszMenuName: pointer_or_null(&self.menu_name),
                lpszClassName: self.class_name.as_ptr(),
                hIconSm: self.hIconSm,
            };
            // TODO: probably should be RegisterClassExW so we can set small icon
            let class_atom = RegisterClassExW(&wnd);
            if class_atom == 0 {
                // This should probably be GetLastError.
                Err(Error::RegisterClassFailed)
            } else {
                Ok(WindowClass::Atom(class_atom))
            }
        }
    }

    /// Set the window class style.
    ///
    /// The argument is the bitwise OR of a number of `CS_` values from the [Window Class Styles] enumeration.
    /// It becomes `style` field in the [`WNDCLASSEX`] passed to [`RegisterClassEx`]. See [Class Styles] for
    /// more explanation.
    ///
    /// [`RegisterClassEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexw
    /// [`WNDCLASSEX`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw
    /// [Class Styles]: https://docs.microsoft.com/en-us/windows/win32/winmsg/about-window-classes#class-styles
    /// [Window Class Styles]: https://docs.microsoft.com/en-us/windows/win32/winmsg/window-class-styles
    pub fn class_style(mut self, style: DWORD) -> Self {
        self.style = style;
        self
    }

    /// Allocate extra bytes in window instances.
    ///
    /// The argument becomes the `cbWndExtra` field in the [`WNDCLASSEX`] passed to [`RegisterClassEx`].
    ///
    /// Generally this isn't that useful unless creating a dialog, in which case it should be
    /// [`DLGWINDOWEXTRA`](#associatedconstant.DLGWINDOWEXTRA).
    ///
    /// Note: there is no corresponding method to set `cbClsExtra`, as I can't think of a good reason
    /// why it would ever be needed.
    ///
    /// # Safety
    ///
    /// The argument must be a reasonable size (no idea what happens if negative, for example).
    ///
    /// [`RegisterClassEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexw
    /// [`WNDCLASSEX`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw
    pub unsafe fn wnd_extra_bytes(mut self, extra_bytes: c_int) -> Self {
        self.cbWndExtra = extra_bytes;
        self
    }

    /// Set the instance handle.
    ///
    /// The argument becomes the `hInstance` field in the [`WNDCLASSEX`] passed to [`RegisterClassEx`].
    ///
    /// See the [`instance`](struct.WindowBuilder.html#method.instance) method on `WindowBuilder` for
    /// more details.
    ///
    /// # Safety
    ///
    /// The argument must be a valid HINSTANCE reference.
    ///
    /// [`RegisterClassEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexw
    /// [`WNDCLASSEX`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw
    pub unsafe fn instance(mut self, instance: HINSTANCE) -> Self {
        self.hInstance = instance;
        self
    }

    /// Set the icon.
    ///
    /// The argument becomes the `hIcon` field in the [`WNDCLASSEX`] passed to [`RegisterClassEx`].
    ///
    /// # Safety
    ///
    /// The argument must be a valid HICON reference.
    ///
    /// [`RegisterClassEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexw
    /// [`WNDCLASSEX`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw
    pub unsafe fn icon(mut self, icon: HICON) -> Self {
        self.hIcon = icon;
        self
    }

    /// Set the small icon.
    ///
    /// The argument becomes the `hIconSm` field in the [`WNDCLASSEX`] passed to [`RegisterClassEx`].
    ///
    /// # Safety
    ///
    /// The argument must be a valid HICON reference.
    ///
    /// [`RegisterClassEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexw
    /// [`WNDCLASSEX`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw
    pub unsafe fn small_icon(mut self, icon: HICON) -> Self {
        self.hIconSm = icon;
        self
    }

    /// Set the cursor.
    ///
    /// The argument becomes the `hCursor` field in the [`WNDCLASSEX`] passed to [`RegisterClassEx`].
    ///
    /// The default implementation of [`WM_SETCURSOR`] applies this cursor. In the old-school approach
    /// where each control has its own HWND, it's reasonable to use this to set the cursor, then
    /// everything should just work (even without explicit handling of `WM_SETCURSOR`). However, in
    /// the modern approach where there's a single window for the application, probably a more useful
    /// strategy is to set the cursor on `WM_MOUSEMOVE`, which reports the cursor position (rather than
    /// relying on hit testing with the HWND bounds). In that case, setting a default cursor on the
    /// window will likely result in flashing, as the two window message handlers will compete.
    ///
    /// This [Stack overflow question](https://stackoverflow.com/questions/19257237/reset-cursor-in-wm-setcursor-handler-properly)
    /// contains more details.
    ///
    /// Of course, if the entire window is to have a single cursor, setting it here is quite reasonable.
    ///
    /// # Safety
    ///
    /// The argument must be a valid HCURSOR reference.
    ///
    /// [`RegisterClassEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexw
    /// [`WNDCLASSEX`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw
    /// [`WM_SETCURSOR`]: https://docs.microsoft.com/en-us/windows/win32/menurc/wm-setcursor
    /// [`WM_MOUSEMOVE`]: https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-mousemove
    pub unsafe fn cursor(mut self, cursor: HCURSOR) -> Self {
        self.hCursor = cursor;
        self
    }

    /// Set the background brush.
    ///
    /// The argument becomes the `hBrBackground` field in the [`WNDCLASSEX`] passed to [`RegisterClassEx`].
    ///
    /// # Safety
    ///
    /// The argument must be a valid HBRUSH reference.
    ///
    /// [`RegisterClassEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexw
    /// [`WNDCLASSEX`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw
    pub unsafe fn background(mut self, brush: HBRUSH) -> Self {
        self.hbrBackground = brush;
        self
    }

    /// Set the default menu.
    ///
    /// The argument becomes the `lpszClassName` field in the [`WNDCLASSEX`] passed to [`RegisterClassEx`].
    ///
    /// The string references the resource name of the class menu. There is no mechanism to support
    /// the MAKEINTRESOURCE macro.
    ///
    /// [`RegisterClassEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexw
    /// [`WNDCLASSEX`]: https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw
    pub fn menu_name(mut self, menu_name: impl AsRef<OsStr>) -> Self {
        self.menu_name = menu_name.to_wide_null();
        self
    }

    /// The number of extra bytes needed for dialogs.
    ///
    /// See [`wnd_extra_bytes`](#method.wnd_extra_bytes).

    // Note: this arguably should be defined in the winapi crate. In any case,
    // probably not that important.
    pub const DLGWINDOWEXTRA: c_int = 30;
}

/// A convenience function for an optional string, on which an empty slice
/// returns a null pointer.
fn pointer_or_null(slice: &[u16]) -> *const u16 {
    if slice.is_empty() {
        null()
    } else {
        slice.as_ptr()
    }
}
