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
    factory: IDXGIFactory,
    adapter: IDXGIAdapter,
    device: ID3D12Device,
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
        let factory = ret.create_factory()?;
        let adapter = ret.create_adapter(&factory)?;
        let device = ret.create_device(&adapter)?;

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

        ret.resources = Some(Resources {
            hwnd,
            factory,
            adapter,
            device,
        });

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

    // factory를 생성한다
    fn create_factory(&self) -> Result<IDXGIFactory> {
        let factory = unsafe { CreateDXGIFactory() }?;

        Ok(factory)
    }

    // DirectX12를 지원하는 처음 adapter를 반환한다.
    fn create_adapter(&self, factory: &IDXGIFactory) -> Result<IDXGIAdapter> {
        // 루프를 돌면서 D3D_FEATURE_LEVEL_12_1을 지원하는
        // 어댑터가 있는지 확인한다.
        for i in 0.. {
            let mut adapter = None;
            let adapter = unsafe { factory.EnumAdapters(i, &mut adapter) }
                .and_some(adapter)
                .unwrap();

            // windows 크레이트에서 제공하는 D3D12CreateDevice 함수는
            // adapter의 지원여부만을 확인하는 방법을 제공하지 않기 때문에
            // d3d12.lib에서 직접 D3D12CreateDevice 함수를 가져온다.
            #[link(name = "d3d12")]
            #[allow(dead_code)]
            extern "system" {
                pub fn D3D12CreateDevice(
                    pdapter: windows::RawPtr,
                    minimutfeaturelevel: D3D_FEATURE_LEVEL,
                    riid: *const windows::Guid,
                    ppdevice: *mut *mut std::ffi::c_void,
                ) -> windows::HRESULT;
            }

            if unsafe {
                D3D12CreateDevice(
                    adapter.abi(),
                    D3D_FEATURE_LEVEL_12_1,
                    &ID3D12Device::IID,
                    std::ptr::null_mut(),
                )
            }
            .is_ok()
            {
                return Ok(adapter);
            }
        }

        unreachable!()
    }

    // device를 생성한다.
    fn create_device(&self, adapter: &IDXGIAdapter) -> Result<ID3D12Device> {
        let device = unsafe { D3D12CreateDevice(adapter, D3D_FEATURE_LEVEL_12_1) }?;

        Ok(device)
    }
}
