use paris::{error, info, success};
use std::io::{BufReader, Read, Write};
use std::sync::{Arc, Mutex};

// performance optimization for bit counting
const BIT_COUNTS: [u8; 256] = [
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
];

pub struct Disk {
    device: String,
    pub bit_flips: u64,
    previous_bit_flips: u64,
    size_bytes: u64,
    buffer: [u8; 512],
    read_count: u64,
}

impl Disk {
    pub fn new(device: String) -> Disk {
        let mut disk = Disk {
            device,
            bit_flips: 0,
            size_bytes: 0,
            buffer: [0; 512],
            read_count: 0,
        };
        disk.get_size();
        info!("Disk size: {} bytes", disk.size_bytes);
        info!("Starting to get bit flips on disk {}...", disk.device);
        disk.get_bit_flips();
        disk
    }

    fn get_size(&mut self) {
        // use blockdev to get the total size of the block device and then store it as a u32
        self.size_bytes = std::process::Command::new("blockdev")
            .arg("--getsize64")
            .arg(&self.device)
            .output()
            .unwrap_or_else(|_| {
                error!("Could not run blockdev command!");
                error!("Exiting...");
                std::process::exit(1);
            })
            .stdout
            .iter()
            .map(|&x| x as char)
            .collect::<String>()
            .trim()
            .parse()
            .unwrap_or_else(|err| {
                error!("Could not parse blockdev output!");
                error!("Error: {}", err);
                std::process::exit(1);
            });
    }

    pub fn get_bit_flips(&mut self) {
        loop {
            let mut file = std::fs::File::open(&self.device).unwrap();
            let mut buffer = [0; 512];

            while let Ok(bytes_read) = file.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }

                // Process the buffer and update the bit_flips count
                for byte in buffer.iter() {
                    self.bit_flips += u64::from(BIT_COUNTS[*byte as usize]);
                    if (self.bit_flips != self.previous_bit_flips) {
                        info!("Bit flip detected")
                        // reset the byte read on the device back to 0
                        file.seek(std::io::SeekFrom::Start(0)).unwrap();
                        
                    }
                }

                self.previous_bit_flips = self.bit_flips;
            }
        }
    }

    // pub fn write_bit_flip_data(&mut self) {
    //     // write the bit flip data to a file
    //     let mut file = std::fs::File::create(format!("{}-bit-flips.txt", self.device)).unwrap();
    //     // write the time and bit flips to the file
    //     file.write_all(
    //         format!(
    //             "{}: {} bit flips\n",
    //             chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    //             self.bit_flips
    //         )
    //         .as_bytes(),
    //     )
    //     .unwrap();
    // }
}
