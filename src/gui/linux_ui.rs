pub fn init_gtk() {
    // must be initialized in main thread (because of gtk requirements)
    //gtk::init().expect("Could not initialize gtk");
}
