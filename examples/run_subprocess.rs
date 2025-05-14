use std::process::Command;
use utf8_supported::CommandUtf8Ext;

pub fn main() {
    let mut cmd = Command::new(std::env::args().nth(1).expect("No command provided"));
    let support = utf8_supported::utf8_supported();
    if support == utf8_supported::Utf8Support::UTF8 {
        println!("run_subprocess: UTF-8 supported ({support:?})");
    } else {
        println!("run_subprocess: Setting C locale ({support:?})");
        cmd.set_c_locale();
    }
    let output = cmd.output().expect("failed to run command");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
