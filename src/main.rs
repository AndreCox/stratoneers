#![feature(portable_simd)]
// import macros from paris
use simplelog::*;
use simplelog::{info, warn};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
    time::Duration,
};

// import modules
mod disks;

// include the assets/audio folder in the binary
// do this with include_bytes! macro
// this will include the audio files in the binary

// links to audio files
const AUDIO_FILES: [&str; 2] = [
    "/home/andre/stratoneers/assets/audio/hello.wav",
    "/home/andre/stratoneers/assets/audio/isAnyoneThere.wav",
];
const BLOCK_DEVICES: [&str; 4] = ["/dev/sda", "/dev/sdb", "/dev/sdc", "/dev/sdd"];

#[tokio::main]
async fn main() {
    // ascii art banner saying stratonlseers
    let banner = r#"
                ____  _             _                                    ____          _      
               / ___|| |_ _ __ __ _| |_ ___  _ __   ___  ___ _ __ ___   / ___|___   __| | ___ 
               \___ \| __| '__/ _` | __/ _ \| '_ \ / _ \/ _ \ '__/ __| | |   / _ \ / _` |/ _ \
                ___) | |_| | | (_| | || (_) | | | |  __/  __/ |  \__ \ | |__| (_) | (_| |  __/
               |____/ \__|_|  \__,_|\__\___/|_| |_|\___|\___|_|  |___/  \____\___/ \__,_|\___|
     _____               __                       __      ______             _______           __             
    |     \.-----.-----.|__|.-----.-----.-----.--|  |    |   __ \.--.--.    |   _   |.-----.--|  |.----.-----.
    |  --  |  -__|__ --||  ||  _  |     |  -__|  _  |    |   __ <|  |  |    |       ||     |  _  ||   _|  -__|
    |_____/|_____|_____||__||___  |__|__|_____|_____|    |______/|___  |    |___|___||__|__|_____||__| |_____|
                            |_____|                              |_____|                                      
    "#;
    println!("{}", banner);

    // scan for an existing log file
    // if it exists we check the number on the end and increment it
    // if it doesn't exist we create it
    let mut log_file_number = 0;
    loop {
        if std::path::Path::new(&format!("stratoneers_code_{}.log", log_file_number)).exists() {
            log_file_number += 1;
        } else {
            break;
        }
    }

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(format!("./stratoneers_code_{}.log", log_file_number)).unwrap(),
        ),
    ])
    .unwrap();

    info!("Starting up...");
    // play audio files
    info!("Hello?");
    play_audio(0);
    info!("Is anyone there?");
    play_audio(1);

    // check for simd support
    if is_x86_feature_detected!("sse3") {
        info!("<b><green>SIMD detected!</></b>");
    } else {
        warn!("<b>SIMD not detected!</b>");
        warn!("This program will run very slowly!");
    }

    // first check if the block devices are mounted, they shouldn't be
    // if they are, exit as they may be used by the os
    info!("Checking if block devices are mounted...");
    // read the mount file if it fails, exit with error
    let mount_file = std::fs::read_to_string("/proc/mounts").unwrap_or_else(|_| {
        error!("Could not read mount file!");
        error!("Probably Permission error");
        error!("Exiting...");
        std::process::exit(1);
    });
    // see if the block devices appear in the mount file
    for device in BLOCK_DEVICES.iter() {
        if mount_file.contains(device) {
            error!("Block device {} is mounted!", device);
            error!("Exiting...");
            std::process::exit(1);
        }
    }

    // Create a vector of disks in an Arc<Mutex>
    let mut disks = Vec::new();
    for device in BLOCK_DEVICES.iter() {
        info!("Creating disk for {}", device);
        disks.push(Arc::new(Mutex::new(disks::Disk::new(device.to_string()))));
    }

    // create tokio tasks for each disk to get_bit_flips
    let mut tasks = Vec::new();
    for disk in disks.iter() {
        let disk = disk.clone();
        tasks.push(tokio::spawn(async move {
            // get the bit flips
            let _bit_flips = disk.lock().unwrap().get_bit_flips();
        }));
    }

    info!("<b><green>Started all threads!</></b>");

    loop {
        // sleep here to avoid busy-waiting
        // since we are using tokio, this won't block other tasks
        tokio::time::sleep(Duration::from_secs(100)).await;
    }

    // Add a delay to avoid busy-waiting (adjust the duration as needed)
}

fn play_audio(index: usize) {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let file = std::fs::File::open(AUDIO_FILES[index]).unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
    sink.set_volume(1.0);
    sink.sleep_until_end();
}
