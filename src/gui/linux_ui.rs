pub fn init_gtk() {
    // must be initialized in main thread (because of gtk requirements)
    let _result = gtk::init().expect("Could not initialize gtk");
}
