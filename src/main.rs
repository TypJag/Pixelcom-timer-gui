use iced::widget::{button, column, container, row, text};
use iced::{Background, Color, Element, Subscription, Task, Theme};
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Production PixelCom device address — change as needed
const DEFAULT_HOST: &str = "192.168.0.85";
const DEFAULT_PORT: u16 = 24;

fn main() -> iced::Result {
    iced::application(TimerApp::new, update, view)
        .title(|_state: &TimerApp| "PixelCom Timer GUI".to_string())
        .subscription(subscription)
        .run()
}

// ---------------------------------------------------------------------------
// Shared TCP stream: cloning increments the Arc reference count only
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct SharedStream(Arc<Mutex<TcpStream>>);

impl std::fmt::Debug for SharedStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedStream")
    }
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Flag {
    No,
    Green,
    Yellow,
    Red,
    Finish,
    ShowTime,
}

#[derive(Debug, Clone)]
enum Message {
    /// Fired every second by the subscription
    Tick,
    /// Add or subtract seconds from the current time
    AddTime(i32),
    /// Add or subtract seconds from the default (reset) time
    AddDefaultTime(i32),
    /// Force the timer to zero immediately
    EndNow,
    /// Freeze the countdown
    Halt,
    /// Resume the countdown
    UnHalt,
    /// Reset the timer to the default time
    Reset,
    /// Start an async TCP connection to PixelCom
    ConnectPixel,
    /// Drop the TCP connection
    DisconnectPixel,
    /// Send a flag command to PixelCom (or enable show-time mode)
    SetFlag(Flag),
    /// Terminate the process
    ExitApp,
    /// Delivered when the async connect attempt finishes
    ConnectResult(Result<SharedStream, String>),
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct TimerApp {
    time_left: u32,
    default_time: u32,
    halted: bool,
    is_finished: bool,
    connected: bool,
    /// When true the current time is sent to PixelCom every tick
    show_time: bool,
    error_msg: String,
    tcp_stream: Option<SharedStream>,
}

impl TimerApp {
    fn new() -> Self {
        Self {
            time_left: 600,
            default_time: 600,
            halted: false,
            is_finished: false,
            connected: false,
            show_time: false,
            error_msg: String::new(),
            tcp_stream: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_time(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

/// Decode a lowercase hex string into bytes without external crate dependency.
fn hex_bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// Build and send the time-update message.
/// Message format: 69 08 08 69 00 01 0e [min] 00 [sec] 00 00 [checksum] 16
/// Checksum = 1 + 14 + minutes + seconds
fn send_time_to_pixelcom(stream: &SharedStream, time: u32) -> Result<(), String> {
    if time == 0 {
        return Ok(());
    }
    let minutes = time / 60;
    let secs = time % 60;
    let checksum = ((1u32 + 14 + minutes + secs) & 0xFF) as u8;
    let hex_string = format!(
        "6908086900010e{:02x}00{:02x}0000{:02x}16",
        minutes, secs, checksum
    );
    let bytes = hex_bytes(&hex_string);
    let mut guard = stream.0.lock().map_err(|_| "Mutex poisoned".to_string())?;
    guard.write_all(&bytes).map_err(|e| e.to_string())
}

/// Send a flag command to PixelCom.
fn send_flag_to_pixelcom(stream: &SharedStream, flag: &Flag) -> Result<(), String> {
    let hex_string = match flag {
        Flag::No => "6908086900010000000000000116",
        Flag::Yellow => "6908086900011c00000000001d16",
        Flag::Finish => "6908086900010600000000646b16",
        Flag::Green => "6908086900011f00000000002016",
        Flag::Red => "6908086900011b00000000001c16",
        Flag::ShowTime => return Ok(()), // ShowTime is a mode, not a direct command
    };
    let bytes = hex_bytes(hex_string);
    let mut guard = stream.0.lock().map_err(|_| "Mutex poisoned".to_string())?;
    guard.write_all(&bytes).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

fn update(state: &mut TimerApp, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            // Send current time to PixelCom when show-time mode is active
            if state.show_time {
                if let Some(ref stream) = state.tcp_stream {
                    if let Err(e) = send_time_to_pixelcom(stream, state.time_left) {
                        state.error_msg = format!("Send error: {e}");
                    }
                }
            }

            // Decrement the counter when running
            if !state.halted && state.time_left > 0 {
                state.time_left -= 1;
            }

            // Send finish flag once when the timer reaches zero
            if state.time_left == 0 && !state.is_finished {
                state.is_finished = true;
                if let Some(ref stream) = state.tcp_stream {
                    if let Err(e) = send_flag_to_pixelcom(stream, &Flag::Finish) {
                        state.error_msg = format!("Send error: {e}");
                    }
                }
            }

            Task::none()
        }

        Message::AddTime(delta) => {
            if delta >= 0 {
                state.time_left = state.time_left.saturating_add(delta as u32);
            } else {
                state.time_left = state.time_left.saturating_sub((-delta) as u32);
            }
            Task::none()
        }

        Message::AddDefaultTime(delta) => {
            if delta >= 0 {
                state.default_time = state.default_time.saturating_add(delta as u32);
            } else {
                state.default_time = state.default_time.saturating_sub((-delta) as u32);
            }
            Task::none()
        }

        Message::EndNow => {
            state.time_left = 0;
            Task::none()
        }

        Message::Halt => {
            state.halted = true;
            Task::none()
        }

        Message::UnHalt => {
            state.halted = false;
            Task::none()
        }

        Message::Reset => {
            state.time_left = state.default_time;
            state.is_finished = false;
            Task::none()
        }

        // Perform the TCP connect asynchronously so the UI stays responsive
        Message::ConnectPixel => {
            let host = DEFAULT_HOST.to_string();
            let port = DEFAULT_PORT;
            Task::perform(
                async move {
                    let stream = tokio::net::TcpStream::connect((host.as_str(), port))
                        .await
                        .map_err(|e| e.to_string())?;
                    // Convert to std stream so it can be stored and used synchronously
                    let std_stream = stream.into_std().map_err(|e| e.to_string())?;
                    std_stream
                        .set_nonblocking(false)
                        .map_err(|e| e.to_string())?;
                    // Short write timeout so a slow/stalled device does not freeze the UI tick
                    std_stream
                        .set_write_timeout(Some(Duration::from_millis(500)))
                        .map_err(|e| e.to_string())?;
                    Ok::<SharedStream, String>(SharedStream(Arc::new(Mutex::new(std_stream))))
                },
                Message::ConnectResult,
            )
        }

        Message::DisconnectPixel => {
            state.tcp_stream = None; // dropping the Arc closes the stream
            state.connected = false;
            state.error_msg.clear();
            Task::none()
        }

        Message::ConnectResult(result) => {
            match result {
                Ok(stream) => {
                    state.tcp_stream = Some(stream);
                    state.connected = true;
                    state.error_msg.clear();
                }
                Err(e) => {
                    state.connected = false;
                    state.error_msg = format!("Error connecting to pixelcom: {e}");
                }
            }
            Task::none()
        }

        Message::SetFlag(flag) => {
            if flag == Flag::ShowTime {
                // Enable continuous time-sending; the actual command is sent each Tick
                state.show_time = true;
            } else {
                state.show_time = false;
                if let Some(ref stream) = state.tcp_stream {
                    if let Err(e) = send_flag_to_pixelcom(stream, &flag) {
                        state.error_msg = format!("Send error: {e}");
                    }
                }
            }
            Task::none()
        }

        Message::ExitApp => {
            std::process::exit(0);
        }
    }
}

// ---------------------------------------------------------------------------
// Subscription — fires Message::Tick every second
// ---------------------------------------------------------------------------

fn subscription(_state: &TimerApp) -> Subscription<Message> {
    iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
}

// ---------------------------------------------------------------------------
// View helpers
// ---------------------------------------------------------------------------

fn flag_button_style(
    bg: Color,
    text_color: Color,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |_theme, _status| button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        ..button::Style::default()
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

fn view(state: &TimerApp) -> Element<'_, Message> {
    let halted_label = if state.halted { "  [HALTED]" } else { "" };

    // ----- Left column -----

    let connected_status = text(format!(
        "Connected to PixelCom: {}",
        if state.connected { "Yes" } else { "No" }
    ))
    .size(20);

    let time_display =
        text(format!("{}{}", format_time(state.time_left), halted_label)).size(72);

    let default_time_display =
        text(format!("Default Time: {}", format_time(state.default_time))).size(20);

    let connection_row = row![
        button("Connect to PixelCom").on_press(Message::ConnectPixel),
        button("Disconnect from PixelCom").on_press(Message::DisconnectPixel),
        button("Exit").on_press(Message::ExitApp),
    ]
    .spacing(10);

    let time_modifier_row = row![
        button("Default time +60s").on_press(Message::AddDefaultTime(60)),
        button("Default time -60s").on_press(Message::AddDefaultTime(-60)),
        button("+60s").on_press(Message::AddTime(60)),
        button("-60s").on_press(Message::AddTime(-60)),
    ]
    .spacing(10);

    let control_row = row![
        button("End time now!").on_press(Message::EndNow),
        button("Halt timer").on_press(Message::Halt),
        button("Unhalt timer").on_press(Message::UnHalt),
        button("Reset timer").on_press(Message::Reset),
    ]
    .spacing(10);

    let left_column = column![
        text("PixelCom-Timer_GUI").size(28),
        connected_status,
        time_display,
        default_time_display,
        connection_row,
        time_modifier_row,
        control_row,
    ]
    .spacing(15)
    .padding(20);

    // ----- Right column (flag buttons) -----

    let gray = Color::from_rgb8(201, 201, 201);
    let black = Color::BLACK;
    let white = Color::WHITE;
    let btn_width = iced::Length::Fixed(200.0);
    let btn_pad = 20;

    let right_column = column![
        button("No flag")
            .style(flag_button_style(gray, black))
            .on_press(Message::SetFlag(Flag::No))
            .padding(btn_pad)
            .width(btn_width),
        button("Green flag")
            .style(flag_button_style(Color::from_rgb8(0, 233, 0), black))
            .on_press(Message::SetFlag(Flag::Green))
            .padding(btn_pad)
            .width(btn_width),
        button("Yellow flag")
            .style(flag_button_style(Color::from_rgb8(255, 255, 0), black))
            .on_press(Message::SetFlag(Flag::Yellow))
            .padding(btn_pad)
            .width(btn_width),
        button("Red flag")
            .style(flag_button_style(Color::from_rgb8(255, 0, 0), black))
            .on_press(Message::SetFlag(Flag::Red))
            .padding(btn_pad)
            .width(btn_width),
        button("Finish flag")
            .style(flag_button_style(black, white))
            .on_press(Message::SetFlag(Flag::Finish))
            .padding(btn_pad)
            .width(btn_width),
        button("Show Time Left")
            .style(flag_button_style(gray, black))
            .on_press(Message::SetFlag(Flag::ShowTime))
            .padding(btn_pad)
            .width(btn_width),
    ]
    .spacing(10)
    .padding(20);

    // ----- Error strip -----

    let error_row = if state.error_msg.is_empty() {
        row![text("")]
    } else {
        row![text(format!("Error: {}", state.error_msg)).size(14)]
    };

    // ----- Root layout -----

    container(
        column![row![left_column, right_column].spacing(20), error_row]
            .spacing(10)
            .padding(10),
    )
    .into()
}
