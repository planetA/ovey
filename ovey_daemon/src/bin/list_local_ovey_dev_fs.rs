use ovey_daemon::util::get_all_local_ovey_devices;

fn main() {
    let foo = get_all_local_ovey_devices();
    println!("{:#?}", foo);
}