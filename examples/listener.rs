use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use leapcpp::{ControllerRef, Listener, ManagedController, Policy};

struct MyListener {
    exit: Arc<AtomicBool>,
}

#[allow(unused_variables)]
impl Listener for MyListener {
    fn on_init(&mut self, controller: &ControllerRef) {
        println!("on_init");
    }

    fn on_connect(&mut self, controller: &ControllerRef) {
        println!("on_connect");
    }

    fn on_disconnect(&mut self, controller: &ControllerRef) {
        println!("on_disconnect");
    }

    fn on_exit(&mut self, controller: &ControllerRef) {
        println!("on_exit");
    }

    fn on_frame(&mut self, controller: &ControllerRef) {
        println!("on_frame");
        let id = controller.frame().id();
        println!("- frame ID: {}", id);
    }

    fn on_focus_gained(&mut self, controller: &ControllerRef) {
        println!("on_focus_gained");
    }

    fn on_focus_lost(&mut self, controller: &ControllerRef) {
        println!("on_focus_lost");
    }

    fn on_service_connect(&mut self, controller: &ControllerRef) {
        println!("on_service_connect");
    }

    fn on_service_disconnect(&mut self, controller: &ControllerRef) {
        println!("on_service_disconnect");
    }

    fn on_device_change(&mut self, controller: &ControllerRef) {
        println!("on_device_change");
    }

    fn on_images(&mut self, controller: &ControllerRef) {
        println!("on_images");

        let images = controller.images();
        println!("- {} images", images.len());

        for image in images.iter() {
            println!(" - {} {:?}:", image.sequence_id(), image.camera());
            println!("   - timestamp {:?}", image.timestamp());
        }

        self.exit.store(true, Ordering::Relaxed);
    }
}

impl Drop for MyListener {
    fn drop(&mut self) {
        println!("dropping `MyListener`");
    }
}

fn main() {
    let exit = Arc::new(AtomicBool::new(false));

    let mut controller = ManagedController::new();
    controller.wait_until_device_connected();
    controller.set_policy(Policy::Images);

    controller.add_listener(MyListener { exit: exit.clone() });

    println!("waiting for `on_image` event");
    while !exit.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }
}
