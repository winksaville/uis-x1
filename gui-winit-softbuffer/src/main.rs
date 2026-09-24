//! Display "Hello, World" in a window until Return is pressed or the window is closed, then exit.
//!
//! The window twin of the terminal program: winit opens the window and delivers its events, and
//! softbuffer presents a pixel buffer the library draws the text into.

use std::error::Error;
use std::num::NonZeroU32;
use std::rc::Rc;

use gui_winit_softbuffer::{Canvas, render, scale_for};
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

/// A boxed error from any of the three crates, so one type carries them out of the event loop.
type BoxError = Box<dyn Error>;

/// The window and the surface that presents into it, created once the event loop resumes.
struct Shown {
    /// The window, shared with the surface, which presents into it.
    window: Rc<Window>,
    /// The softbuffer surface over the window.
    surface: Surface<Rc<Window>, Rc<Window>>,
}

/// The application: the window once shown, and the first error, kept for `main` to return.
#[derive(Default)]
struct App {
    /// The window and its surface, `None` until `resumed`.
    shown: Option<Shown>,
    /// The first error, which also ends the event loop.
    error: Option<BoxError>,
}

impl App {
    /// Record the error and end the event loop, since a handler cannot return one.
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: BoxError) {
        self.error.get_or_insert(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for App {
    /// Open the window and its surface the first time the event loop resumes.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.shown.is_some() {
            return;
        }
        match show(event_loop) {
            Ok(shown) => self.shown = Some(shown),
            Err(error) => self.fail(event_loop, error),
        }
    }

    /// Redraw when asked, and exit on Return or when the window is closed.
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Enter),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(shown) = &self.shown {
                    shown.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(shown) = &mut self.shown
                    && let Err(error) = draw(shown)
                {
                    self.fail(event_loop, error);
                }
            }
            _ => {}
        }
    }
}

/// Create the window, titled with the package name, and the softbuffer surface over it.
fn show(event_loop: &ActiveEventLoop) -> Result<Shown, BoxError> {
    let attributes = Window::default_attributes().with_title(env!("CARGO_PKG_NAME"));
    let window = Rc::new(event_loop.create_window(attributes)?);
    let context = Context::new(window.clone())?;
    let surface = Surface::new(&context, window.clone())?;
    Ok(Shown { window, surface })
}

/// Size the buffer to the window, draw the frame into it at the window's scale, and present it.
///
/// A window with no area, a minimized one for instance, has nothing to draw, so it is skipped.
fn draw(shown: &mut Shown) -> Result<(), BoxError> {
    let size = shown.window.inner_size();
    let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
    else {
        return Ok(());
    };
    shown.surface.resize(width, height)?;
    let mut buffer = shown.surface.buffer_mut()?;
    let scale = scale_for(shown.window.scale_factor());
    render(&mut Canvas::new(
        &mut buffer,
        size.width,
        size.height,
        scale,
    ));
    buffer.present()?;
    Ok(())
}

/// Run the event loop until Return or a close, then return the first error, if any.
fn main() -> Result<(), BoxError> {
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    match app.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
