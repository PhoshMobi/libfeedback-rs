use libfeedback as lfb;

fn main() {
    lfb::init("org.sigxcpu.Lfb-Rs").unwrap();

    let event = lfb::Event::new("phone-incoming-call");

    event.connect_feedback_ended(|_| {
        glib::g_message!("example", "Feedback ended");
    });

    match event.trigger_feedback() {
	Ok(_) => {
            glib::g_message!("example", "Triggered event")
        }
        Err(e) => {
            glib::g_critical!("example", "Error triggerieng feedback: {e:#?}");
        }
    }

    let l = glib::MainLoop::new(None, false);
    l.run();

    lfb::functions::uninit();
}
