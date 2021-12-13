use std::process::Command;
use std::{thread, time};
use std::time::{Duration};

static N: i32 = 10;

fn normal_execution() {
    Command::new("cargo").current_dir("../monitor").arg("run").arg(N.to_string()).spawn().expect("Failed to initialize monitor");

    for i in 0..N {
        Command::new("cargo").current_dir("../server").arg("run").arg(i.to_string()).arg(N.to_string()).spawn().expect("Failed to initialize server");
    }

    Command::new("cargo").current_dir("../client").arg("run").spawn().expect("Failed to initialize client");
}

fn failed_execution() {
    let mut pids: Vec<u32> = Vec::new();

    Command::new("cargo").current_dir("../monitor").arg("run").arg(N.to_string()).spawn().expect("Failed to initialize monitor");

    for i in 0..N {
        let process = Command::new("cargo").current_dir("../server").arg("run").arg(i.to_string()).arg(N.to_string()).spawn().expect("Failed to initialize server");
        pids.push(process.id());
    }

    Command::new("cargo").current_dir("../client").arg("run").spawn().expect("Failed to initialize client");

    // Kill server after 10 seconds
    thread::sleep(time::Duration::from_millis(10000));
    println!("Killing server 6");
    Command::new("kill").arg("-9").arg(pids[6].to_string()).spawn().expect("Failed to kill server");
}

fn main() {
    // normal_execution();
    failed_execution();
}
