use fj_viewer::{Screen, ScreenSize};
use winit::{event_loop::EventLoop, window::WindowBuilder};

/// A window that can be used with `fj-viewer`
pub struct Window(winit::window::Window);

impl Window {
    /// Create an instance of `Window` from the given `EventLoop`
    pub fn new<T>(event_loop: &EventLoop<T>) -> Result<Self, WindowError> {
        let window = WindowBuilder::new()
            .with_title("Fornjot")
            .with_maximized(true)
            // When the window decorations are enabled, I'm seeing the following
            // error on Gnome/Wayland, in response to a `ScaleFactorChange`
            // event:
            // ```
            // wl_surface@24: error 2: Buffer size (1940x45) must be an integer multiple of the buffer_scale (2).
            // ```
            //
            // This is happening most of the time. Very rarely, the window will
            // open as expected.
            //
            // I believe that there is a race condition somewhere low in the
            // stack, that will cause the buffer size for the window decorations
            // to not be updated before the check that produces the above error.
            // I failed to track down where this is happening, so I decided to
            // deploy this workaround instead of spending more time.
            //
            // Window decorations should be re-enabled once possible. This is
            // being tracked in this issue:
            // https://github.com/hannobraun/fornjot/issues/1848
            .with_decorations(false)
            .with_transparent(false)
            .build(event_loop)?;

        Ok(Self(window))
    }
}

impl Screen for Window {
    type Window = winit::window::Window;

    fn size(&self) -> ScreenSize {
        let size = self.0.inner_size();

        ScreenSize {
            width: size.width,
            height: size.height,
        }
    }

    fn window(&self) -> &winit::window::Window {
        &self.0
    }
}

/// Error initializing window
#[derive(Debug, thiserror::Error)]
#[error("Error initializing window")]
pub struct WindowError(#[from] pub winit::error::OsError);
