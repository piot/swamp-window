/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-window
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use crate::dpi::PhysicalSize;
use log::info;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// A trait for handling application-specific window creation and management.
///
/// The `AppHandler` trait defines the behavior required for handling
/// the creation of application windows. Implementing this trait allows
/// for customized window management tailored to the needs of your application.
pub trait AppHandler {
    fn create_window(&mut self, window: Arc<Window>);

    fn resized(&mut self, size: dpi::PhysicalSize<u32>);

    fn min_size(&self) -> (u16, u16);

    fn start_size(&self) -> (u16, u16);

    fn redraw(&mut self);
}

pub struct App<'a> {
    window: Option<Arc<Window>>,
    handler: &'a mut (dyn AppHandler),
    window_attributes: WindowAttributes,
}

impl<'a> App<'a> {
    pub fn new(
        handler: &'a mut dyn AppHandler,
        title: &str,
        min_size: (u16, u16),
        start_size: (u16, u16),
    ) -> Self {
        let min_logical_size = PhysicalSize::new(min_size.0 as f64, min_size.1 as f64);
        let start_logical_size = PhysicalSize::new(start_size.0 as f64, start_size.1 as f64);

        let window_attributes = WindowAttributes::default()
            .with_title(title)
            .with_resizable(true)
            .with_inner_size(start_logical_size)
            .with_min_inner_size(min_logical_size);

        Self {
            handler,
            window_attributes,
            window: None,
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            info!("creating new window");

            let window = Arc::new(
                event_loop
                    .create_window(self.window_attributes.clone())
                    .unwrap(),
            );
            self.window = Some(window.clone());

            self.handler.create_window(window);
            info!("created the window");
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
            WindowEvent::Resized(physical_size) => {
                self.handler.resized(physical_size);

                // This tells winit that we want another frame after this one
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                // This tells winit that we want another frame after this one
                self.window.as_ref().unwrap().request_redraw();

                if self.window.is_some() {
                    self.handler.redraw();
                }
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
    /// use winit::dpi;
    ///
    /// struct MyApp;
    ///
    /// #[async_trait]
    /// impl AppHandler for MyApp {
    ///     async fn create_window(&mut self, window: Arc<Window>) {
    ///         // Custom window initialization code here
    ///     }
    ///
    /// fn redraw(&mut self) { todo!() }
    /// fn resized(&mut self, size: dpi::PhysicalSize<u32>) { todo!() }
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
    pub fn run_app(handler: &mut dyn AppHandler, title: &str) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let min_size = handler.min_size();
        let start_size = handler.start_size();
        let mut app = App::new(handler, title, min_size, start_size);
        let _ = event_loop.run_app(&mut app);
        Ok(())
    }
}
