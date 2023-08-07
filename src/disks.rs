use simplelog::{error, info};
use std::{
    io::{BufReader, Read, Seek},
    simd::u8x64,
};
// Import the lazy_static macro
use lazy_static::lazy_static;

// Create a lookup table for counting bits quickly
lazy_static! {
    static ref BIT_COUNTS: [u8; 65536] = {
        let mut table = [0; 65536];
        for num in 0..65536 {
            table[num] = num.count_ones() as u8;
        }
        table
    };
}

pub struct Disk {
    pub device: String,
    pub bit_flips: u64,
    size_bytes: u64,
}

impl Disk {
    pub fn new(device: String) -> Disk {
        let mut disk = Disk {
            device,
            bit_flips: 0,
            size_bytes: 0,
        };
        disk.get_size();
        info!("Disk size: {} bytes", disk.size_bytes);
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
        info!("Starting bit flip counter for {}", self.device);

        let mut file = BufReader::new(std::fs::File::open(&self.device).unwrap());
        let mut buffer = [0; 8192];
        let mut bit_flips = 0u64;
        let mut current_position = 0u64;
        let mut bitflip_locations = Vec::new();

        loop {
            // Read a chunk of data from the file
            let bytes_read = file.read(&mut buffer).unwrap();

            // first break the u8 array into 64 byte chunks since the array is 8192 bytes
            // and we know the size of the disk will always be a multiple of 8192 bytes we can safely assume that
            // the last chunk will always be 8192 bytes so edge cases are not a problem

            // lets break our buffer up into 64 byte arrays of u8
            let mut buffer_64 = [u8x64::splat(0); 128];
            for i in 0..128 {
                let mut temp = [0u8; 64];
                temp.copy_from_slice(&buffer[i * 64..(i + 1) * 64]);
                buffer_64[i] = u8x64::from_array(temp);
            }

            // now we have our 128 u8x64 arrays we can sum them to one u8x64 array
            let mut buffer_64_sum = u8x64::splat(0);
            for i in 0..128 {
                buffer_64_sum += buffer_64[i];
            }

            // now we have our final u8x64 array we can loop through and count the bits
            let mut sum = 0;
            for i in 0..64 {
                sum += BIT_COUNTS[buffer_64_sum[i] as usize] as u64;
            }

            if sum != 0 {
                // now if the sum is not 0 we know that there has been a bit flip somewhere in the 8192 bytes
                // so we can now loop through the 128 u8x64 arrays and find the exact location of the bit flip
                for i in 0..128 {
                    for j in 0..64 {
                        if BIT_COUNTS[buffer_64[i][j] as usize] != 0 {
                            // we have found the exact location of the bit flip so we can now increment the bit flip counter
                            bit_flips += BIT_COUNTS[buffer_64[i][j] as usize] as u64;
                            // we also want to store the exact location of the bit flip so we can print it later we will store this location as a hex string
                            let hex_string =
                                format!("{:x}", current_position + (i * 64) as u64 + j as u64);
                            info!(
                                "<b><green>Bit flip found at {} on {}</></b>",
                                hex_string, self.device
                            );
                            bitflip_locations.push(hex_string);
                        }
                    }
                }
            }

            // Print the progress every 100000 reads to avoid spamming the console
            if current_position % 10000000 == 0 {
                info!(
                    "Progress: {}% BitFlips: {} Device: {}",
                    (current_position as f64 * 100.0) / self.size_bytes as f64,
                    bit_flips,
                    self.device
                );
            }

            current_position += bytes_read as u64;

            // Reset the cursor position to the beginning of the disk if the end of the file is reached
            if bytes_read < buffer.len() {
                file.seek(std::io::SeekFrom::Start(0)).unwrap();
                info!("End of file reached, resetting cursor position");
            }
        }
    }
}
