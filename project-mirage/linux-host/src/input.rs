use anyhow::{Context, Result};
use evdev::{Device, EventType, InputEventKind, Key};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, debug, warn, error};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    pub buttons: MouseButtons,
    pub screen_width: u32,
    pub screen_height: u32,
}

#[derive(Debug, Clone, Default)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    MouseMove { delta_x: f32, delta_y: f32 },
    MouseButton { button: MouseButton, pressed: bool },
    MouseWheel { delta: f32, horizontal: bool },
    KeyPress { key_code: u32, pressed: bool },
    EdgeCrossed { edge: ScreenEdge, position: (f32, f32) },
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenEdge {
    Left,
    Right,
    Top,
    Bottom,
}

pub struct InputManager {
    config: Config,
    mouse_state: Arc<RwLock<MouseState>>,
    event_tx: mpsc::Sender<InputEvent>,
    event_rx: Option<mpsc::Receiver<InputEvent>>,
    mouse_device: Option<Device>,
}

impl InputManager {
    pub fn new(config: Config) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(1000);

        // Find mouse device
        let mouse_device = Self::find_mouse_device()?;
        
        if let Some(ref device) = mouse_device {
            info!("✓ Found mouse device: {}", device.name().unwrap_or("unknown"));
        } else {
            warn!("⚠ No mouse device found - input capture disabled");
        }

        let mouse_state = Arc::new(RwLock::new(MouseState {
            x: 0.0,
            y: 0.0,
            buttons: MouseButtons::default(),
            screen_width: 1920, // TODO: Get actual screen dimensions
            screen_height: 1080,
        }));

        Ok(Self {
            config,
            mouse_state,
            event_tx,
            event_rx: Some(event_rx),
            mouse_device,
        })
    }

    fn find_mouse_device() -> Result<Option<Device>> {
        // Try to find a mouse or pointer device
        let devices = evdev::enumerate().collect::<Vec<_>>();
        
        for (path, device) in devices {
            // Check if device supports relative movement (mouse/touchpad)
            if device.supported_events().contains(EventType::RELATIVE) {
                debug!("Found input device: {} at {:?}", 
                    device.name().unwrap_or("unknown"), path);
                
                // Check if it's a mouse (has button events)
                if device.supported_keys().map(|keys| {
                    keys.contains(Key::BTN_LEFT) || keys.contains(Key::BTN_MOUSE)
                }).unwrap_or(false) {
                    info!("Selected mouse device: {}", device.name().unwrap_or("unknown"));
                    return Ok(Some(device));
                }
            }
        }

        warn!("No suitable mouse device found");
        Ok(None)
    }

    pub async fn run(mut self) -> Result<()> {
        if self.mouse_device.is_none() {
            error!("Cannot run input manager: no mouse device available");
            return Ok(());
        }

        info!("Starting input event monitoring...");

        let mut device = self.mouse_device.take().unwrap();
        let event_tx = self.event_tx.clone();
        let mouse_state = Arc::clone(&self.mouse_state);
        let edge_threshold = self.config.host.display_edge_threshold as f32;

        // Spawn input monitoring task
        tokio::task::spawn_blocking(move || {
            loop {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            let rt = tokio::runtime::Handle::current();
                            
                            rt.block_on(async {
                                if let Err(e) = Self::process_event(
                                    event,
                                    &event_tx,
                                    &mouse_state,
                                    edge_threshold,
                                ).await {
                                    warn!("Error processing event: {}", e);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            error!("Error fetching events: {}", e);
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            }
        });

        Ok(())
    }

    async fn process_event(
        event: evdev::InputEvent,
        event_tx: &mpsc::Sender<InputEvent>,
        mouse_state: &Arc<RwLock<MouseState>>,
        edge_threshold: f32,
    ) -> Result<()> {
        match event.kind() {
            InputEventKind::RelAxis(axis) => {
                match axis {
                    evdev::RelativeAxisType::REL_X => {
                        let delta_x = event.value() as f32;
                        let mut state = mouse_state.write().await;
                        
                        let old_x = state.x;
                        state.x = (state.x + delta_x).clamp(0.0, state.screen_width as f32);
                        
                        // Check for edge crossing
                        if old_x >= edge_threshold && state.x < edge_threshold {
                            // Crossed left edge
                            drop(state);
                            let _ = event_tx.send(InputEvent::EdgeCrossed {
                                edge: ScreenEdge::Left,
                                position: (0.0, state.y),
                            }).await;
                        } else if old_x <= (state.screen_width as f32 - edge_threshold) 
                            && state.x > (state.screen_width as f32 - edge_threshold) {
                            // Crossed right edge
                            let y = state.y;
                            drop(state);
                            let _ = event_tx.send(InputEvent::EdgeCrossed {
                                edge: ScreenEdge::Right,
                                position: (state.screen_width as f32, y),
                            }).await;
                        } else {
                            drop(state);
                            let _ = event_tx.send(InputEvent::MouseMove {
                                delta_x,
                                delta_y: 0.0,
                            }).await;
                        }
                    }
                    evdev::RelativeAxisType::REL_Y => {
                        let delta_y = event.value() as f32;
                        let mut state = mouse_state.write().await;
                        
                        let old_y = state.y;
                        state.y = (state.y + delta_y).clamp(0.0, state.screen_height as f32);
                        
                        // Check for edge crossing
                        if old_y >= edge_threshold && state.y < edge_threshold {
                            // Crossed top edge
                            drop(state);
                            let _ = event_tx.send(InputEvent::EdgeCrossed {
                                edge: ScreenEdge::Top,
                                position: (state.x, 0.0),
                            }).await;
                        } else if old_y <= (state.screen_height as f32 - edge_threshold)
                            && state.y > (state.screen_height as f32 - edge_threshold) {
                            // Crossed bottom edge
                            let x = state.x;
                            drop(state);
                            let _ = event_tx.send(InputEvent::EdgeCrossed {
                                edge: ScreenEdge::Bottom,
                                position: (x, state.screen_height as f32),
                            }).await;
                        } else {
                            drop(state);
                            let _ = event_tx.send(InputEvent::MouseMove {
                                delta_x: 0.0,
                                delta_y,
                            }).await;
                        }
                    }
                    evdev::RelativeAxisType::REL_WHEEL => {
                        let delta = event.value() as f32;
                        let _ = event_tx.send(InputEvent::MouseWheel {
                            delta,
                            horizontal: false,
                        }).await;
                    }
                    evdev::RelativeAxisType::REL_HWHEEL => {
                        let delta = event.value() as f32;
                        let _ = event_tx.send(InputEvent::MouseWheel {
                            delta,
                            horizontal: true,
                        }).await;
                    }
                    _ => {}
                }
            }
            InputEventKind::Key(key) => {
                let pressed = event.value() != 0;
                
                let button = match key {
                    Key::BTN_LEFT | Key::BTN_MOUSE => Some(MouseButton::Left),
                    Key::BTN_RIGHT => Some(MouseButton::Right),
                    Key::BTN_MIDDLE => Some(MouseButton::Middle),
                    Key::BTN_SIDE => Some(MouseButton::Back),
                    Key::BTN_EXTRA => Some(MouseButton::Forward),
                    _ => None,
                };

                if let Some(button) = button {
                    let mut state = mouse_state.write().await;
                    match button {
                        MouseButton::Left => state.buttons.left = pressed,
                        MouseButton::Right => state.buttons.right = pressed,
                        MouseButton::Middle => state.buttons.middle = pressed,
                        _ => {}
                    }
                    drop(state);

                    let _ = event_tx.send(InputEvent::MouseButton {
                        button,
                        pressed,
                    }).await;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn subscribe(&mut self) -> mpsc::Receiver<InputEvent> {
        self.event_rx.take().unwrap()
    }

    pub async fn get_mouse_state(&self) -> MouseState {
        self.mouse_state.read().await.clone()
    }
}
