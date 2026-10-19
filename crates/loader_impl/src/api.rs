use macros::generate_vtable;

generate_vtable!(MainExtension, {
    fn ext_main();
});
