use std::intrinsics::transmute;

use bindings::Windows::Win32::{
    Graphics::{Direct3D11::*, Direct3D12::*, Dxgi::*, Hlsl::*},
    System::{
        SystemServices::*,
        Threading::{CreateEventA, WaitForSingleObject},
        WindowsProgramming::INFINITE,
    },
    UI::{DisplayDevices::RECT, WindowsAndMessaging::*},
};

use utf16_literal::utf16;
use windows::*;

pub struct Resources {
    hwnd: HWND,
}

pub struct BareBoneGame {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resources: Option<Resources>,
}

impl BareBoneGame {
    pub fn new(title: String, width: u32, height: u32) -> Result<BareBoneGame> {
        let mut ret = BareBoneGame {
            title,
            width,
            height,
            resources: None,
        };

        let hwnd = ret.create_window()?;

        unsafe { ShowWindow(hwnd, SW_SHOW) };

        loop {
            let mut msg = MSG::default();

            if unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                if msg.message == WM_QUIT {
                    break;
                }
            }
        }

        Ok(ret)
    }

    fn create_window(&self) -> Result<HWND> {
        let hinstance = unsafe { GetModuleHandleW(None) };
        debug_assert!(!hinstance.is_null());

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc),
            hInstance: hinstance,
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) },
            lpszClassName: PWSTR(utf16!("Barebone-Game\0").as_ptr() as _),
            ..Default::default()
        };

        unsafe { RegisterClassExW(&wc) };

        let hwnd = unsafe {
            CreateWindowExW(
                Default::default(),
                "Barebone-Game",
                self.title.as_str(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                self.width as _,
                self.height as _,
                None,
                None,
                hinstance,
                std::ptr::null_mut(),
            )
        };

        debug_assert!(!hwnd.is_null());

        Ok(hwnd)
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => (),
        }

        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
