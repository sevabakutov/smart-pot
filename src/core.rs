mod private {}

crate::mod_interface! {
    layer client;
    layer error;
    layer telemetry;
    layer wifi;
}
