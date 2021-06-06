fn main() {
    windows::build!(
        // Windows
        Windows::Win32::UI::WindowsAndMessaging::{HWND, WPARAM, LPARAM, WNDCLASSW, WNDCLASSEXW, MSG, CREATESTRUCTW},
        Windows::Win32::UI::WindowsAndMessaging::{RegisterClassW, CreateWindowExW, ShowWindow, PeekMessageW, TranslateMessage, DispatchMessageW, PostQuitMessage, DefWindowProcW, LoadCursorW, GetWindowLongPtrW, SetWindowLongPtrW,
        RegisterClassExW},
        Windows::Win32::UI::WindowsAndMessaging::{WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, SW_SHOW, WM_DESTROY, PM_REMOVE, WM_QUIT, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, WM_PAINT, GWLP_USERDATA, WM_CREATE},
        Windows::Win32::System::SystemServices::{LRESULT, PSTR, PWSTR},
        Windows::Win32::System::SystemServices::{GetModuleHandleW},
        Windows::Win32::UI::DisplayDevices::{RECT},
        // Threading
        Windows::Win32::System::Threading::{CreateEventA, WaitForSingleObject},
        Windows::Win32::System::WindowsProgramming::{INFINITE},
        //Graphics
        Windows::Win32::Graphics::Direct3D11::{
            D3D_FEATURE_LEVEL_12_1,
            D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST
        },
        // D3D12
        Windows::Win32::Graphics::Direct3D12::*,
        Windows::Win32::Graphics::Dxgi::*,
        Windows::Win32::Graphics::Hlsl::*,
    );
}
