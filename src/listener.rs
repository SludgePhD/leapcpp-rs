use std::{
    ffi::c_void,
    mem::MaybeUninit,
    panic::{catch_unwind, AssertUnwindSafe},
    process,
};

use crate::{sys, ControllerRef};

/// An event listener.
#[allow(unused_variables)]
pub trait Listener: Send + 'static {
    /// Called when the listener is added to a [`Controller`][crate::Controller].
    fn on_init(&mut self, controller: &ControllerRef) {}
    fn on_connect(&mut self, controller: &ControllerRef) {}
    fn on_disconnect(&mut self, controller: &ControllerRef) {}

    /// Invoked when the [`Controller`][crate::Controller] owning this listener is destroyed, or
    /// when the listener is removed from the controller.
    fn on_exit(&mut self, controller: &ControllerRef) {}

    fn on_frame(&mut self, controller: &ControllerRef) {}
    fn on_focus_gained(&mut self, controller: &ControllerRef) {}
    fn on_focus_lost(&mut self, controller: &ControllerRef) {}
    fn on_service_connect(&mut self, controller: &ControllerRef) {}
    fn on_service_disconnect(&mut self, controller: &ControllerRef) {}
    fn on_device_change(&mut self, controller: &ControllerRef) {}
    fn on_images(&mut self, controller: &ControllerRef) {}
}

pub(crate) struct BoxedListener {
    #[allow(dead_code)] // needed for drop side-effect
    rust: Box<dyn Listener>,
    pub(crate) sys: sys::Leap_RustListener,
}

pub(crate) fn create_rust_listener<L: Listener>(listener: L) -> Box<BoxedListener> {
    let boxed = Box::new(listener);
    let callbacks = sys::Leap_RustListenerCallbacks {
        onInit: Some(cb_on_init::<L>),
        onConnect: Some(cb_on_connect::<L>),
        onDisconnect: Some(cb_on_disconnect::<L>),
        onExit: Some(cb_on_exit::<L>),
        onFrame: Some(cb_on_frame::<L>),
        onFocusGained: Some(cb_on_focus_gained::<L>),
        onFocusLost: Some(cb_on_focus_lost::<L>),
        onServiceConnect: Some(cb_on_service_connect::<L>),
        onServiceDisconnect: Some(cb_on_service_disconnect::<L>),
        onDeviceChange: Some(cb_on_device_change::<L>),
        onImages: Some(cb_on_images::<L>),
        userdata: (&*boxed) as *const _ as _,
    };

    unsafe {
        let mut sys = MaybeUninit::uninit();
        sys::Leap_RustListener_RustListener(sys.as_mut_ptr(), callbacks);

        Box::new(BoxedListener {
            rust: boxed,
            sys: sys.assume_init(),
        })
    }
}

macro_rules! wrap_callbacks {
    (
        $(
            $wrapper_name:ident -> $method_name:ident,
        )+
    ) => {
        $(
            unsafe extern "C" fn $wrapper_name<L: Listener>(
                userdata: *mut c_void,
                controller: *const sys::Leap_Controller,
            ) {
                let listener = &mut *(userdata as *mut L);
                let controller = ControllerRef::from_raw(controller);

                let res = catch_unwind(AssertUnwindSafe(|| {
                    listener.$method_name(controller);
                }));

                if res.is_err() {
                    process::abort();
                }
            }
        )+
    };
}

wrap_callbacks! {
    cb_on_init -> on_init,
    cb_on_connect -> on_connect,
    cb_on_disconnect -> on_disconnect,
    cb_on_exit -> on_exit,
    cb_on_frame -> on_frame,
    cb_on_focus_gained -> on_focus_gained,
    cb_on_focus_lost -> on_focus_lost,
    cb_on_service_connect -> on_service_connect,
    cb_on_service_disconnect -> on_service_disconnect,
    cb_on_device_change -> on_device_change,
    cb_on_images -> on_images,
}
