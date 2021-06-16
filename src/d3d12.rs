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
        let command_queue = ret.create_command_queue(&device)?;
        let command_allocator = ret.create_command_allocator(&device)?;
        let (swap_chain, frame_index) =
            ret.create_swap_chain(2, &hwnd, &factory, &command_queue)?;
        let (rtv_heap, rtv_desc_size) = ret.create_rtv_heap(2, &device)?;
        let render_targets =
            ret.create_render_target(2, rtv_desc_size, &device, &swap_chain, &rtv_heap);

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

    // 커맨드 큐를 생성한다
    fn create_command_queue(&self, device: &ID3D12Device) -> Result<ID3D12CommandQueue> {
        let command_queue = unsafe {
            device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                ..Default::default()
            })
        }?;

        Ok(command_queue)
    }

    // 커맨드 할당자를 생성한다
    fn create_command_allocator(&self, device: &ID3D12Device) -> Result<ID3D12CommandAllocator> {
        let command_allocator =
            unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }?;

        Ok(command_allocator)
    }

    // 스왑 체인을 생성한다
    fn create_swap_chain(
        &self,
        buffer_count: u32,
        hwnd: &HWND,
        factory: &IDXGIFactory,
        command_queue: &ID3D12CommandQueue,
    ) -> Result<(IDXGISwapChain, u32)> {
        // IDXGIFactorty2::CreateSwapChainForHwnd 함수를 호출하기 위한 캐스팅
        let factory: IDXGIFactory2 = factory.cast()?;

        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            BufferCount: buffer_count,
            Width: self.width,
            Height: self.height,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut swap_chain = None;
        let swap_chain: IDXGISwapChain3 = unsafe {
            factory.CreateSwapChainForHwnd(
                command_queue,
                hwnd,
                &swap_chain_desc,
                std::ptr::null(),
                None,
                &mut swap_chain,
            )
        }
        .and_some(swap_chain)?
        // IDXGISwapChain3::GetCurrentBackBufferIndex를 호출하기 위한 캐스팅
        .cast()?;

        let frame_index = unsafe { swap_chain.GetCurrentBackBufferIndex() };

        Ok((swap_chain.cast()?, frame_index))
    }

    // RTV Heap을 생성합니다.
    fn create_rtv_heap(
        &self,
        buffer_count: u32,
        device: &ID3D12Device,
    ) -> Result<(ID3D12DescriptorHeap, u32)> {
        let rtv_heap: ID3D12DescriptorHeap = unsafe {
            device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                NumDescriptors: buffer_count,
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                ..Default::default()
            })
        }?;

        let rtv_desc_size =
            unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) };

        Ok((rtv_heap, rtv_desc_size))
    }

    fn create_render_target(
        &self,
        buffer_count: u32,
        rtv_desc_size: u32,
        device: &ID3D12Device,
        swap_chain: &IDXGISwapChain,
        rtv_heap: &ID3D12DescriptorHeap,
    ) -> Result<Vec<ID3D12Resource>> {
        let rtv_handle = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };

        let mut render_targets = Vec::new();

        for i in 0..buffer_count {
            let render_target: ID3D12Resource = unsafe { swap_chain.GetBuffer(i) }?;

            unsafe {
                device.CreateRenderTargetView(
                    &render_target,
                    std::ptr::null(),
                    &D3D12_CPU_DESCRIPTOR_HANDLE {
                        ptr: rtv_handle.ptr + (rtv_desc_size * i) as usize,
                    },
                )
            }

            render_targets.push(render_target);
        }

        Ok(render_targets)
    }
}
