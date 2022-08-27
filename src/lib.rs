// Note: (some?) `Leap.h` types appear to be location-sensitive, so they must be constructed on the
// heap.

mod listener;

pub mod image;
mod managed;
mod timestamp;

use image::ImageList;
pub use managed::ManagedController;
pub use timestamp::Timestamp;

use std::{mem::MaybeUninit, ops::Deref};

use leapcpp_sys as sys;

use listener::BoxedListener;
pub use listener::Listener;

/// A connection to a leapd instance.
///
/// This is the main entry point of this library. There is also [`ManagedController`], which
/// provides additional utilities that are missing from [`Controller`].
pub struct Controller {
    sys: Box<sys::Leap_Controller>,
    listeners: Vec<Box<BoxedListener>>,
}

impl Controller {
    /// Creates a new [`Controller`], connecting to leapd in the background.
    pub fn new() -> Self {
        unsafe {
            let mut controller = Box::new(MaybeUninit::uninit());
            sys::Leap_Controller_Controller1(controller.as_mut_ptr());
            Self {
                sys: init_box(controller),
                listeners: Vec::new(),
            }
        }
    }

    /// Adds a new [`Listener`] to the controller, which will be notified of any events.
    ///
    /// The [`Listener`]'s methods will be invoked from another thread, so it has to be thread-safe.
    pub fn add_listener<L: Listener>(&mut self, listener: L) {
        let mut listener = listener::create_rust_listener(listener);
        let success = unsafe {
            sys::Leap_Controller_addListener(&mut *self.sys, &mut listener.sys as *mut _ as _)
        };

        if success {
            self.listeners.push(listener);
        }

        // FIXME: should do something when this fails
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        unsafe {
            sys::Leap_Controller_Controller_destructor(&mut *self.sys);
        }

        // Drop glue will free the listeners.
    }
}

impl Deref for Controller {
    type Target = ControllerRef;

    fn deref(&self) -> &Self::Target {
        unsafe { ControllerRef::from_raw(&*self.sys) }
    }
}

/// A reference to a controller.
///
/// This exposes most of the controller interface, except access to [`Listener`]s, which requires
/// extra book-keeping provided by [`Controller`].
#[repr(transparent)]
pub struct ControllerRef {
    sys: sys::Leap_Controller,
}

impl ControllerRef {
    unsafe fn from_raw<'a>(sys: *const sys::Leap_Controller) -> &'a Self {
        &*(sys as *const _ as *const Self)
    }

    /// Returns whether the connection to leapd is established.
    pub fn is_service_connected(&self) -> bool {
        unsafe { sys::Leap_Controller_isServiceConnected(&self.sys) }
    }

    /// Returns whether a Leap Motion Controller is connected.
    ///
    /// When a controller is connected or disconnected, [`Listener::on_connect`] or
    /// [`Listener::on_disconnect`] will be invoked.
    pub fn is_connected(&self) -> bool {
        unsafe { sys::Leap_Controller_isConnected(&self.sys) }
    }

    /// Returns whether this application currently has device focus.
    ///
    /// When this property changes, [`Listener::on_focus_lost`] or [`Listener::on_focus_gained`]
    /// will be invoked.
    pub fn has_focus(&self) -> bool {
        unsafe { sys::Leap_Controller_hasFocus(&self.sys) }
    }

    /// Sets a leapd or device policy.
    ///
    /// Make sure to only set policies *after* a device is connected, otherwise the policy might not
    /// come into effect (eg. your app might not receive any image data even though
    /// [`Policy::Images`] was set).
    ///
    /// Note that policies are not enabled immediately, so [`ControllerRef::is_policy_set`] might
    /// still return `false` for a while after a policy is enabled.
    pub fn set_policy(&self, policy: Policy) {
        unsafe {
            sys::Leap_Controller_setPolicy(&self.sys, policy as u32);
        }
    }

    /// Unsets a leapd or device policy.
    pub fn clear_policy(&self, policy: Policy) {
        unsafe {
            sys::Leap_Controller_clearPolicy(&self.sys, policy as u32);
        }
    }

    /// Returns whether a leapd or device policy is currently enabled.
    pub fn is_policy_set(&self, policy: Policy) -> bool {
        unsafe { sys::Leap_Controller_isPolicySet(&self.sys, policy as u32) }
    }

    /// Returns the current timestamp.
    pub fn now(&self) -> Timestamp {
        let raw = unsafe { sys::Leap_Controller_now(&self.sys) };
        Timestamp::from_raw(raw)
    }

    /// Returns the most recent frame of tracking data.
    pub fn frame(&self) -> Frame {
        self.frame_at(0)
    }

    /// Returns a frame of tracking data, of the specified age.
    ///
    /// `history` specified how old the returned frame should be. 0 selects the most recent frame, 1
    /// the frame before that, and so on.
    ///
    /// The maximum value of `history` is 59. Using a value greater than that will return an invalid
    /// [`Frame`] object.
    pub fn frame_at(&self, history: u8) -> Frame {
        unsafe {
            let mut frame = Box::new(MaybeUninit::uninit());
            sys::Leap_Controller_frame(frame.as_mut_ptr(), &self.sys, history.into());
            Frame {
                inner: init_box(frame),
            }
        }
    }

    /// Returns the most recent set of captured images.
    pub fn images(&self) -> ImageList {
        unsafe {
            let mut images = Box::new(MaybeUninit::uninit());
            sys::Leap_Controller_images(images.as_mut_ptr(), &self.sys);
            ImageList::from_raw(init_box(images))
        }
    }

    /// Enables detection and reporting of a gesture.
    ///
    /// Enabled gestures will be included in the [`Frame`] objects.
    pub fn enable_gesture(&self, gesture: GestureType) {
        unsafe {
            sys::Leap_Controller_enableGesture(&self.sys, gesture as i32, true);
        }
    }

    /// Disables detection and reporting of a gesture.
    pub fn disable_gesture(&self, gesture: GestureType) {
        unsafe {
            sys::Leap_Controller_enableGesture(&self.sys, gesture as i32, false);
        }
    }

    /// Returns whether detection and reporting of a gesture is enabled.
    pub fn is_gesture_enabled(&self, gesture: GestureType) -> bool {
        unsafe { sys::Leap_Controller_isGestureEnabled(&self.sys, gesture as i32) }
    }
}

/// A device or leapd policy.
///
/// These can be enabled or disabled via [`ControllerRef::set_policy`] and
/// [`ControllerRef::clear_policy`].
#[repr(u32)]
#[non_exhaustive]
pub enum Policy {
    //Default = sys::Leap_Controller_PolicyFlag_POLICY_DEFAULT,
    /// Receive [`Frame`]s even when the application does not have focus
    /// ([`ControllerRef::has_focus`]).
    BackgroundFrames = sys::Leap_Controller_PolicyFlag_POLICY_BACKGROUND_FRAMES,

    /// Receive raw camera [`Image`][image::Image]s.
    Images = sys::Leap_Controller_PolicyFlag_POLICY_IMAGES,

    /// Optimize tracking for head-mounted device (when turned off, tracking is optimized for the
    /// device sitting flat on a surface).
    OptimizeHmd = sys::Leap_Controller_PolicyFlag_POLICY_OPTIMIZE_HMD,
}

/// A frame of tracking data.
pub struct Frame {
    inner: Box<sys::Leap_Frame>,
}

impl Frame {
    /// Returns the frame's unique ID.
    ///
    /// The ID is incremented for every reported frame.
    pub fn id(&self) -> i64 {
        unsafe { sys::Leap_Frame_id(&*self.inner) }
    }

    /// The timestamp at which this frame was captured.
    pub fn timestamp(&self) -> Timestamp {
        let raw = unsafe { sys::Leap_Frame_timestamp(&*self.inner) };
        Timestamp::from_raw(raw)
    }

    /// Returns the instantaneous frame rate at which this frame was captured.
    pub fn frames_per_second(&self) -> f32 {
        unsafe { sys::Leap_Frame_currentFramesPerSecond(&*self.inner) }
    }

    /// Returns whether this frame contains valid data.
    pub fn is_valid(&self) -> bool {
        unsafe { sys::Leap_Frame_isValid(&*self.inner) }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        // No `Frame` destructor, call superclass dtor instead.
        unsafe {
            sys::Leap_Interface_Interface_destructor((&mut *self.inner) as *mut _ as _);
        }
    }
}

unsafe fn init_box<T>(bx: Box<MaybeUninit<T>>) -> Box<T> {
    // FIXME: use `Box::assume_init` when stable
    Box::from_raw(Box::into_raw(bx) as *mut T)
}

/// Types of gestures the Leap Motion service can detect and report to the application.
#[repr(i32)]
#[non_exhaustive]
pub enum GestureType {
    //Invalid = sys::Leap_Gesture_Type_TYPE_INVALID,
    /// Horizontal swiping movement of a hand, with fingers extended.
    Swipe = sys::Leap_Gesture_Type_TYPE_SWIPE,

    /// Movement of a single finger in a circle.
    Circle = sys::Leap_Gesture_Type_TYPE_CIRCLE,

    /// A tapping movement parallel to the Leap Motion Controller.
    ScreenTap = sys::Leap_Gesture_Type_TYPE_SCREEN_TAP,

    /// A tapping movement towards the Leap Motion Controller.
    KeyTap = sys::Leap_Gesture_Type_TYPE_KEY_TAP,
}

/// States describing the progression of a gesture.
#[repr(i32)]
#[non_exhaustive]
pub enum GestureState {
    //Invalid = sys::Leap_Gesture_State_STATE_INVALID,
    /// Gesture is starting just now.
    Start = sys::Leap_Gesture_State_STATE_START,

    /// Gesture that was started earlier is continuing.
    Update = sys::Leap_Gesture_State_STATE_UPDATE,

    /// Gesture that was started earlier is ending.
    Stop = sys::Leap_Gesture_State_STATE_STOP,
}
