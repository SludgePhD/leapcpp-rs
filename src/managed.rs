use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Condvar, Mutex},
};

use crate::{Controller, ControllerRef, Listener};

/// A [`Controller`] that adds a few convenience methods to perform blocking waits for events.
pub struct ManagedController {
    inner: Controller,
    shared: Arc<Shared>,
}

impl ManagedController {
    /// Creates a [`ManagedController`].
    ///
    /// This will register an internal [`Listener`] to make the additional functionality provided by
    /// [`ManagedController`] work.
    pub fn new() -> Self {
        let shared = Arc::new(Shared {
            mutex: Mutex::new(()),

            mutex_frame: Mutex::new(0),
            mutex_images: Mutex::new(0),
            mutex_device_change: Mutex::new(0),

            device_connect: Condvar::new(),
            device_disconnect: Condvar::new(),
            service_connect: Condvar::new(),
            service_disconnect: Condvar::new(),
            frame: Condvar::new(),
            focus_gained: Condvar::new(),
            focus_lost: Condvar::new(),
            device_change: Condvar::new(),
            images: Condvar::new(),
        });

        let mut inner = Controller::new();
        inner.add_listener(ManagedListener {
            shared: shared.clone(),
        });

        Self { inner, shared }
    }

    /// Blocks the calling thread until [`ControllerRef::is_service_connected`] is `true`.
    pub fn wait_until_service_connected(&self) {
        self.wait_until(&self.shared.service_connect, || self.is_service_connected());
    }

    /// Blocks the calling thread until [`ControllerRef::is_service_connected`] is `false`.
    pub fn wait_until_service_disconnected(&self) {
        self.wait_until(&self.shared.service_disconnect, || {
            !self.is_service_connected()
        });
    }

    /// Blocks the calling thread until [`ControllerRef::is_connected`] is `true`.
    pub fn wait_until_device_connected(&self) {
        self.wait_until(&self.shared.device_connect, || self.is_connected());
    }

    /// Blocks the calling thread until [`ControllerRef::is_connected`] is `false`.
    pub fn wait_until_device_disconnected(&self) {
        self.wait_until(&self.shared.device_disconnect, || !self.is_connected());
    }

    /// Blocks the calling thread until [`ControllerRef::has_focus`] is `true`.
    pub fn wait_until_focus_gained(&self) {
        self.wait_until(&self.shared.focus_gained, || self.has_focus());
    }

    /// Blocks the calling thread until [`ControllerRef::has_focus`] is `false`.
    pub fn wait_until_focus_lost(&self) {
        self.wait_until(&self.shared.focus_lost, || !self.has_focus());
    }

    /// Blocks the calling thread until the device configuration changes.
    ///
    /// Device configuration changes include:
    ///
    /// - A Leap Motion device is plugged in or removed.
    /// - "Robust" mode is enabled or disabled.
    /// - The image capture rate is changed.
    pub fn wait_until_device_change(&self) {
        self.wait_until_counter(&self.shared.device_change, &self.shared.mutex_device_change);
    }

    /// Blocks the calling thread until new tracking data is available.
    pub fn wait_until_frame(&self) {
        self.wait_until_counter(&self.shared.frame, &self.shared.mutex_frame);
    }

    /// Blocks the calling thread until a new set of camera images is available.
    pub fn wait_until_images(&self) {
        self.wait_until_counter(&self.shared.images, &self.shared.mutex_images);
    }

    fn wait_until(&self, var: &Condvar, mut predicate: impl FnMut() -> bool) {
        let guard = self.shared.mutex.lock().unwrap();
        drop(var.wait_while(guard, |_| !predicate()).unwrap());
    }

    fn wait_until_counter(&self, var: &Condvar, mutex: &Mutex<u64>) {
        log::trace!("wait_until_counter(var = {:?}, mutex = {:?})", var, mutex);
        let guard = mutex.lock().unwrap();
        let old = *guard;

        drop(var.wait_while(guard, |val| *val == old).unwrap());
    }
}

impl Deref for ManagedController {
    type Target = Controller;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ManagedController {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

struct Shared {
    // Mutex for condvars whose state is stored in the `Connection`.
    mutex: Mutex<()>,

    mutex_frame: Mutex<u64>,
    mutex_images: Mutex<u64>,
    mutex_device_change: Mutex<u64>,

    device_connect: Condvar,
    device_disconnect: Condvar,
    service_connect: Condvar,
    service_disconnect: Condvar,
    frame: Condvar,
    focus_gained: Condvar,
    focus_lost: Condvar,
    device_change: Condvar,
    images: Condvar,
}

struct ManagedListener {
    shared: Arc<Shared>,
}

#[allow(unused_variables)]
impl Listener for ManagedListener {
    fn on_init(&mut self, controller: &ControllerRef) {}
    fn on_exit(&mut self, controller: &ControllerRef) {}

    fn on_connect(&mut self, controller: &ControllerRef) {
        self.shared.device_connect.notify_all();
    }

    fn on_disconnect(&mut self, controller: &ControllerRef) {
        self.shared.device_disconnect.notify_all();
    }

    fn on_frame(&mut self, controller: &ControllerRef) {
        *self.shared.mutex_frame.lock().unwrap() += 1;
        self.shared.frame.notify_all();
    }

    fn on_focus_gained(&mut self, controller: &ControllerRef) {
        self.shared.focus_gained.notify_all();
    }

    fn on_focus_lost(&mut self, controller: &ControllerRef) {
        self.shared.focus_lost.notify_all();
    }

    fn on_service_connect(&mut self, controller: &ControllerRef) {
        self.shared.service_connect.notify_all();
    }

    fn on_service_disconnect(&mut self, controller: &ControllerRef) {
        self.shared.service_disconnect.notify_all();
    }

    fn on_device_change(&mut self, controller: &ControllerRef) {
        *self.shared.mutex_device_change.lock().unwrap() += 1;
        self.shared.device_change.notify_all();
    }

    fn on_images(&mut self, controller: &ControllerRef) {
        *self.shared.mutex_images.lock().unwrap() += 1;
        self.shared.images.notify_all();
    }
}
