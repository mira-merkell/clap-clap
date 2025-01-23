use clap::Plugin;

#[derive(Default)]
pub struct MyPlug;

impl Plugin for MyPlug {
    const ID: &'static str = "com.plugggs.my_plug";
    const NAME: &'static str = "MyPlug";
}

#[derive(Default)]
pub struct MyPlug2;

impl Plugin for MyPlug2 {
    const ID: &'static str = "com.plugggs.my_plug2";
    const NAME: &'static str = "MyPlug2";
}
