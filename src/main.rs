// import macros from paris
use paris::{error, info, success, warn};
use rayon::prelude::*;
use std::{
    io::{BufReader, Read},
    sync::{Arc, Mutex},
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
const BLOCK_DEVICES: [&str; 3] = ["/dev/sda", "/dev/sdb", "/dev/sdc"];

fn main() {
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
    info!("Starting up...");
    // play audio files
    info!("Hello?");
    play_audio(0);
    info!("Is anyone there?");
    play_audio(1);

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

    // create a vector of disks in a arc mutex
    // this will be used to store the disks
    let mut disks = Vec::new();
    for device in BLOCK_DEVICES.iter() {
        disks.push(Arc::new(Mutex::new(disks::Disk::new(device.to_string()))));
    }

    // create a thread that will print the number of bit flips every 5 seconds
    // this thread will run in the background
    let mut total_bit_flip_count = 0;

    let mut bit_flip_count_thread = std::thread::spawn(move || loop {
        // get the bit flips from each disk which is stored in a public variable on the disk struct

        let mut bit_flip_count = 0;
        for disk in disks.iter() {
            bit_flip_count += disk.lock().unwrap().bit_flips;
        }

        info!("Bit flips: {}", total_bit_flip_count);

        std::thread::sleep(std::time::Duration::from_secs(5));
    });

    success!("Started bit flip count thread!");

    loop {
        info!("Doing other things");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn play_audio(index: usize) {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let file = std::fs::File::open(AUDIO_FILES[index]).unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
    sink.set_volume(1.0);
    sink.sleep_until_end();
}
