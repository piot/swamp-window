use async_trait::async_trait;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// A trait for handling application-specific window creation and management.
///
/// The `AppHandler` trait defines the behavior required for handling
/// the creation of application windows. Implementing this trait allows
/// for customized window management tailored to the needs of your application.
#[async_trait]
pub trait AppHandler {
    /// Creates a new window for the application.
    ///
    /// This asynchronous method is called when a new window needs to be created.
    /// It provides a reference to the newly created `Window` wrapped in an `Arc`,
    /// allowing for shared ownership and safe concurrent access.
    ///
    /// # Parameters
    ///
    /// - `window`: An `Arc<Window>` representing the newly created window.
    ///
    /// # Example
    ///
    /// ```rust
    /// use async_trait::async_trait;
    /// use swamp_window::AppHandler;
    /// use std::sync::Arc;
    /// use winit::window::Window;
    ///
    /// struct MyApp;
    ///
    /// #[async_trait]
    /// impl AppHandler for MyApp {
    ///     async fn create_window(&mut self, window: Arc<Window>) {
    ///         // Custom window initialization code here
    ///     }
    /// }
    /// ```
    async fn create_window(&mut self, window: Arc<Window>);
}

pub struct App<'a> {
    window: Option<Arc<Window>>,
    handler: &'a mut (dyn AppHandler),
}

impl<'a> App<'a> {
    pub fn new(handler: &'a mut dyn AppHandler) -> Self {
        Self {
            handler,
            window: None,
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            self.window = Some(window.clone());

            pollster::block_on(self.handler.create_window(window));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if id != self.window.as_ref().unwrap().id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                // TODO: Add callback to AppHandler
            }
            WindowEvent::RedrawRequested => {
                // TODO: Add callback to AppHandler
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {}

    fn exiting(&mut self, _: &ActiveEventLoop) {}
}

/// A struct responsible for managing the application window lifecycle.
///
/// The `WindowRunner` struct provides functionality to run an application
/// that utilizes an event loop for window management. It abstracts the details
/// of creating and running the event loop, making it easier to integrate window
/// handling into your game application.
pub struct WindowRunner;

impl WindowRunner {
    /// Runs the application with the provided handler.
    ///
    /// This method initializes an event loop and starts the application by
    /// executing the provided `AppHandler`. The event loop runs in a polling
    /// mode, allowing for responsive event handling. It is not guaranteed to ever return.
    ///
    /// # Parameters
    ///
    /// - `handler`: A mutable reference to an object implementing the `AppHandler`
    ///   trait, which defines the behavior of the application in response to events.
    ///
    /// # Returns
    ///
    /// This method returns a `Result<(), EventLoopError>`.
    /// If an error occurs during event loop creation, it returns an `EventLoopError`.
    ///
    /// # Note
    ///
    /// It is not guaranteed to ever return, as the event loop will run indefinitely
    /// until the application is terminated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use swamp_window::WindowRunner;
    /// use async_trait::async_trait;
    /// use swamp_window::AppHandler;
    /// use std::sync::Arc;
    /// use winit::window::Window;
    ///
    /// struct MyApp;
    ///
    /// #[async_trait]
    /// impl AppHandler for MyApp {
    ///     async fn create_window(&mut self, window: Arc<Window>) {
    ///         // Custom window initialization code here
    ///     }
    /// }
    ///
    /// let mut my_app = MyApp;
    ///
    /// #[cfg(test)]
    /// fn test() {
    ///    if let Err(e) = WindowRunner::run_app(&mut my_app) {
    ///       eprintln!("Error running the application: {:?}", e);
    ///    }
    /// }
    /// ```
    pub fn run_app(handler: &mut dyn AppHandler) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App::new(handler);
        let _ = event_loop.run_app(&mut app);
        Ok(())
    }
}