mod app;
mod buffer;
mod color;
mod command;
mod descriptor;
mod device;
mod image;
mod image_view;
mod instance;
mod memory;
mod pipeline;
mod queue;
mod render_pass;
mod swapchain;
mod texture;
mod validation;
mod vertex;

use app::{App, AppError};
use thiserror::Error;
use vulkanalia::{vk, Version};
use winit::{
    dpi::LogicalSize,
    error::{EventLoopError, OsError},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const IS_MACOS: bool = cfg!(target_os = "macos");
pub const PORTABILITY_MACOS_VERSION: Version =
    Version::new(1, 3, 216);
pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] =
    &[vk::KHR_SWAPCHAIN_EXTENSION.name];

type Result<T> = std::result::Result<T, MainError>;
#[derive(Error, Debug)]
enum MainError {
    #[error(transparent)]
    WinitEventLoopError(#[from] EventLoopError),
    #[error(transparent)]
    WinitOsError(#[from] OsError),
    #[error(transparent)]
    AppError(#[from] AppError),
}

fn main() -> Result<()> {
    // env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    match main_f() {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("{}", e);
            Err(e)
        }
    }
}

fn main_f() -> Result<()> {
    // Window

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Broth")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;
    // Root
    let mut app = unsafe { App::create(&window) }?;
    let mut destroying = false;
    let mut minimized = false;
    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::AboutToWait if !destroying && !minimized => {
                unsafe { app.render(&window) }.unwrap();
            }
            // Destroy our Vulkan app.
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                destroying = true;
                unsafe {
                    app.destroy();
                }
                target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width == 0 || size.height == 0 {
                    minimized = true;
                } else {
                    minimized = false;
                    app.resized = true;
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}
