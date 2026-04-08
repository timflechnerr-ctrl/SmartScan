// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ptr;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HWND};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, IsIconic, SW_RESTORE,
};

fn main() {
    // Try to create a system-wide named mutex
    let mutex_name: Vec<u16> = "Global\\SmartScan_SingleInstance_Mutex\0"
        .encode_utf16()
        .collect();

    unsafe {
        let handle = CreateMutexW(ptr::null(), 0, mutex_name.as_ptr());
        if handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            // Another instance is already running — try to bring it to front
            let class_name = ptr::null();
            let window_title: Vec<u16> = "SmartScan\0".encode_utf16().collect();
            let hwnd: HWND = FindWindowW(class_name, window_title.as_ptr());
            if !hwnd.is_null() {
                if IsIconic(hwnd) != 0 {
                    ShowWindow(hwnd, SW_RESTORE);
                }
                SetForegroundWindow(hwnd);
            }
            return;
        }
    }

    smartscan_lib::run()
}
