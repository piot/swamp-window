/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-window
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use crate::dpi::PhysicalSize;
use log::info;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi;
use winit::dpi::PhysicalPosition;
use winit::error::EventLoopError;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, InnerSizeWriter, MouseButton, MouseScrollDelta, Touch,
    TouchPhase, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

/// `AppHandler` - Handle window, cursor, mouse and keyboard events, designed for games and graphical applications.
///
/// Think of `AppHandler` as your app’s backstage crew, handling everything
/// from window setup to keyboard and mouse inputs, and making sure each frame
/// redraws smoothly.
pub trait AppHandler {
    // Query functions

    /// Returns the minimum window size (width, height) in pixels that the application requires.
    ///
    /// This can be used to enforce a minimum size on the window, preventing it from
    /// being resized below this dimension.
    fn min_size(&self) -> (u16, u16);

    /// Returns the starting window size (width, height) in pixels when the application launches.
    ///
    /// This size will be used to set the initial window dimensions on startup.
    fn start_size(&self) -> (u16, u16);

    fn cursor_should_be_visible(&self) -> bool;

    // Window Events

    /// Called to trigger a redraw of the application’s content.
    ///
    /// This method is generally called when the window needs to refresh its
    /// contents, such as after a resize or focus change.
    /// Return false if application should close
    fn redraw(&mut self) -> bool;

    /// Called when the application window gains focus.
    ///
    /// This can be used to resume or activate specific behaviors when the window
    /// becomes active.
    fn got_focus(&mut self);

    /// Called when the application window loses focus.
    ///
    /// Useful for suspending actions or input handling when the application
    /// window is not in the foreground.
    fn lost_focus(&mut self);

    /// Called after the application window has been created and is ready to use.
    ///
    /// Use this method to perform any initialization that requires access to the window,
    /// such as setting up rendering contexts.
    ///
    /// # Parameters
    /// - `window`: A reference-counted pointer to the application window.
    fn window_created(&mut self, window: Arc<Window>);

    /// Called whenever the window is resized, providing the new physical size.
    ///
    /// This method should handle adjustments to the application’s layout and content
    /// based on the window’s new dimensions.
    ///
    /// # Parameters
    /// - `size`: The new size of the window in physical pixels.
    fn resized(&mut self, size: PhysicalSize<u32>);

    // Keyboard Events

    /// Processes keyboard input events, such as key presses and releases.
    ///
    /// # Parameters
    /// - `element_state`: Indicates whether the key is pressed or released.
    /// - `physical_key`: The physical key that was pressed or released.
    fn keyboard_input(&mut self, element_state: ElementState, physical_key: PhysicalKey);

    // Cursor (Pointer) Events

    /// Called when the cursor enters the window.
    ///
    /// This can trigger visual changes or status updates when the cursor moves
    /// into the application window area.
    fn cursor_entered(&mut self);

    /// Called when the cursor leaves the window.
    ///
    /// This can be used to revert visual changes or trigger actions when the
    /// cursor exits the application window.
    fn cursor_left(&mut self);

    /// Handles cursor movement within the window, providing the new position.
    ///
    /// # Parameters
    /// - `physical_position`: The current position of the cursor in physical
    ///   screen coordinates.
    fn cursor_moved(&mut self, physical_position: PhysicalPosition<f64>);

    // Mouse Events

    /// Handles mouse button input events, such as presses and releases.
    ///
    /// # Parameters
    /// - `element_state`: Indicates whether the mouse button is pressed or released.
    /// - `button`: The mouse button that was pressed or released.
    fn mouse_input(&mut self, element_state: ElementState, button: MouseButton);

    /// Processes mouse wheel events, which indicate scrolling actions.
    ///
    /// # Parameters
    /// - `delta`: The amount of scroll, which may be specified in lines or pixels.
    /// - `touch_phase`: The phase of the scroll gesture, which can indicate
    ///   the start, movement, or end of the gesture.
    fn mouse_wheel(&mut self, delta: MouseScrollDelta, touch_phase: TouchPhase);

    /// Handles mouse motion. the delta follows no standard, so it is up to the game to apply
    /// a factor as it sees fit.
    fn mouse_motion(&mut self, delta: (f64, f64));

    // Touch Events

    /// Handles touch input events, such as screen touches and gestures.
    ///
    /// # Parameters
    /// - `touch`: Describes the touch event, including position, phase, and other
    ///   touch-specific information.
    fn touch(&mut self, touch: Touch);

    // Environment or Screen Events

    /// Handles changes to the display scale factor, usually due to monitor DPI changes.
    ///
    /// This method receives the new scale factor and a writer to update the inner
    /// size of the application.
    ///
    /// # Parameters
    /// - `scale_factor`: The new scale factor, which may be applied to adjust
    ///   rendering.
    /// - `inner_size_writer`: A writer to update the inner size.
    fn scale_factor_changed(&mut self, scale_factor: f64, inner_size_writer: InnerSizeWriter);
}

pub struct App<'a> {
    window: Option<Arc<Window>>,
    handler: &'a mut (dyn AppHandler),
    window_attributes: WindowAttributes,
    is_focused: bool,
    cursor_is_visible: bool,
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
            is_focused: false,
            cursor_is_visible: true,
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

            self.handler.window_created(window);
            info!("created the window");
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.is_focused {
                self.handler.mouse_motion(delta);
            }
        }
        /*
        match event {
            // DeviceEvent::MouseWheel { .. } => {},
            //DeviceEvent::Button { .. } => { }
            //DeviceEvent::Added => {}
            //DeviceEvent::Removed => {}
            //DeviceEvent::Motion { .. } => { }
            //DeviceEvent::Key(_) => {}
            _ => {}
        }
         */
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if id != self.window.as_ref().unwrap().id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                self.handler.resized(physical_size);

                // This tells winit that we want another frame after this one
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                // This tells winit that we want another frame after this one
                self.window.as_ref().unwrap().request_redraw();

                let window = self.window.as_mut().unwrap();
                let cursor_visible_request = self.handler.cursor_should_be_visible();
                if cursor_visible_request != self.cursor_is_visible {
                    window.set_cursor_visible(cursor_visible_request);
                    self.cursor_is_visible = cursor_visible_request;
                }

                if self.window.is_some() {
                    let wants_to_keep_going = self.handler.redraw();
                    if !wants_to_keep_going {
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::Focused(is_focus) => {
                self.is_focused = is_focus;
                if is_focus {
                    self.handler.got_focus();
                } else {
                    // usually you might want to stop or lower audio, maybe lower rendering frequency, etc
                    self.handler.lost_focus();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handler.keyboard_input(event.state, event.physical_key)
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.cursor_is_visible {
                    // Position makes no sense to user if the cursor is not visible
                    self.handler.cursor_moved(position)
                }
            }

            WindowEvent::CursorEntered { .. } => self.handler.cursor_entered(),

            WindowEvent::CursorLeft { .. } => self.handler.cursor_left(),

            WindowEvent::MouseWheel { delta, phase, .. } => self.handler.mouse_wheel(delta, phase),

            WindowEvent::MouseInput { state, button, .. } => {
                self.handler.mouse_input(state, button)
            }

            WindowEvent::Touch(touch_data) => self.handler.touch(touch_data),

            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer,
            } =>
            // Changing the display’s resolution.
            // Changing the display’s scale factor (e.g. in Control Panel on Windows).
            // Moving the window to a display with a different scale factor.
            {
                self.handler
                    .scale_factor_changed(scale_factor, inner_size_writer)
            }

            // --------------------------------------------

            // WindowEvent::ModifiersChanged(_) => {} // modifiers comes in as KeyboardInput anyway, so we can ignore this one.
            // WindowEvent::Ime(_) => {} // IME is outside the scope of events, and not supported on all platforms, e.g. Web.

            // Gestures could be relevant, but we leave them for future versions
            //WindowEvent::PinchGesture { .. } => {}
            //WindowEvent::PanGesture { .. } => {}
            //WindowEvent::DoubleTapGesture { .. } => {}
            //WindowEvent::RotationGesture { .. } => {}
            // WindowEvent::TouchpadPressure { .. } => {} // only on some macbooks and similar, not relevant for multiplatform games.
            //WindowEvent::AxisMotion { .. } => {} // intentionally not supported, since we want to use platform-specific api:s for gamepad input
            // WindowEvent::ThemeChanged(_) => {} // mostly unsupported and not really related to games.
            // WindowEvent::Occluded(_) => {} not available on most platforms anyway
            // WindowEvent::ActivationTokenDone { .. } => {} winit handles this normally, so no need to implement it.
            // WindowEvent::Moved(_) => {} // since this is not supported on all platforms, it should not be exposed in this library
            // WindowEvent::Destroyed => {} // this is handled internally
            // since this crate is mostly for games, this file operations are outside the scope.
            //WindowEvent::DroppedFile(_) => {}
            //WindowEvent::HoveredFile(_) => {}
            //WindowEvent::HoveredFileCancelled => {}
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
