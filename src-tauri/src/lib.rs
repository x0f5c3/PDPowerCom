// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
use serialport;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, ipc::Channel};
use serde::Serialize;
use std::thread;

const DEVICE_PID: u16 = 65036;
const DEVICE_VID: u16 = 6790;
const DEVICE_SERIAL_NUMBER: &str = "AC";
const DEVICE_SERIAL_NUMBER2: &str = "B3";
static SERIAL_PORT: Mutex<Option<Box<dyn serialport::SerialPort>>> = Mutex::new(None);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
enum SerialEvent {
    Opened {
        port_name: String,
    },
    _Closed {
        port_name: String,
    },
    Error {
        port_name: String,
        error: String,
    },
    Read {
        data: Vec<u8>,
    }
}
#[tauri::command]
fn serial_get_available_ports() -> (Vec<String>,Vec<String>) {
    let ports = serialport::available_ports().expect("无法获取串口列表");
    let mut port_names = Vec::new();
    let mut port_id = Vec::new();

    for port in ports {
        println!("{:?}", port);
        // 只添加符合设备PID、VID和序列号的USB端口
        if let serialport::SerialPortType::UsbPort(info) = port.port_type {
            if info.pid == DEVICE_PID && info.vid == DEVICE_VID {
                // 安全匹配序列号，消除unwrap
                if let Some(sn) = &info.serial_number {
                    let sn_upper = sn.to_uppercase();
                    if sn_upper.starts_with(DEVICE_SERIAL_NUMBER) || sn_upper.starts_with(DEVICE_SERIAL_NUMBER2) {
                        port_names.push(port.port_name);
                        port_id.push(sn.clone());
                    }
                }
            }
        }
    }
    (port_names,port_id)
}

#[tauri::command]
fn serial_open_port(app: AppHandle, port_name: &str, channel: Channel<SerialEvent>) -> bool {
    let port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(port) => {
            let mut guard = SERIAL_PORT.lock().unwrap();
            *guard = Some(port);
            println!("open port: {:?}", port_name);
            // 发送打开成功事件
            if let Err(e) = channel.send(SerialEvent::Opened { port_name: port_name.to_string() }) {
                println!("发送打开事件失败: {}", e);
                return false;
            }
            
            // 启动数据读取线程
            let app_clone = app.clone();
            let channel_clone = channel.clone();
            thread::spawn(move || {
                serial_loop(app_clone, channel_clone);
            });
            true
        }
        Err(_) => {
            let mut guard = SERIAL_PORT.lock().unwrap();
            *guard = None;
            println!("open port failed");
            false
        }
    }
}

#[tauri::command]
fn serial_close_port() -> bool {
    let mut guard = SERIAL_PORT.lock().unwrap();
    if let Some(port) = guard.take() {
        std::mem::drop(port);
        println!("close port");
        true
    } else {
        println!("close port already");
        true  // 如果串口本来就是关闭的，也返回true
    }
}

fn serial_loop(_app_handle: AppHandle, channel: Channel<SerialEvent>) {
    let mut guard = SERIAL_PORT.lock().unwrap();
    if let Some(port) = guard.as_mut() {
        // 刷新接收缓冲区
        match port.clear(serialport::ClearBuffer::Input) {
            Ok(_) => println!("接收缓冲区已刷新"),
            Err(e) => println!("刷新接收缓冲区失败: {:?}", e),
        }
    }
    drop(guard); // 释放锁
    loop {
        thread::sleep(Duration::from_millis(10)); // 每 10 毫秒检查一次

        let mut guard = SERIAL_PORT.lock().unwrap();
        if let Some(port) = guard.as_mut() {
            let port_name = port.name().unwrap_or_else(|| "Unknown Port".to_string());
            let mut buffer = [0; 1024];
            match port.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let data = &buffer[0..bytes_read];
                        // 发送数据到前端
                        if let Err(e) = channel.send(SerialEvent::Read {data: data.to_vec() }) {
                            println!("发送数据事件失败: {}", e);
                        }
                        // app_handle.emit_all("serial_data", data.to_vec()).unwrap();
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::TimedOut {
                        println!("读取串口数据出错: {:?}", e);
                        // 发送错误事件
                        channel.send(SerialEvent::Error { port_name: port_name, error: e.to_string() }).unwrap();
                        *guard = None;
                        break;
                    }
                }
            }
        } else {
            println!("serial disconnected");
            break;
        }
    }
}

#[tauri::command]
fn serial_write(data: Vec<u8>) -> bool {
    let mut guard = SERIAL_PORT.lock().unwrap();
    if let Some(port) = guard.as_mut() {
        match port.write(&data) {
            Ok(_) => true,
            Err(e) => {
                println!("写入串口数据出错: {:?}", e);
                false
            }
        }
    } else {
        println!("write port failed");
        false
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                // Note: DevTools are not available on mobile
                use tauri::Manager;
                let window = _app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            serial_get_available_ports,
            serial_open_port,
            serial_close_port,
            serial_write,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
