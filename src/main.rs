use std::{
    fs, io,
    sync::{Arc, Mutex},
    thread,
};
use image::{ImageBuffer, Rgb};
use num_cpus;
use png::{self, BitDepth, ColorType, Compression, Encoder, FilterType};
use rand::{Rng};

fn main() {
    let n_generated = Arc::new(Mutex::new(0));
    let n_threads = num_cpus::get();
    println!("Number of threads: {}", n_threads);

    let mut handles = vec![];
    let n = get_number_of_images();
    let start = std::time::Instant::now();
    let output_directory = "images/";
    fs::create_dir_all(output_directory).expect("Could not create directory");

    for _ in 0..n_threads {
        let n_generated = n_generated.clone();
        let handle = thread::spawn(move || {
            loop {
                let mut n_generated = n_generated.lock().unwrap();
                let id = *n_generated;
                if *n_generated >= n {
                    break;
                }
                *n_generated += 1;
                drop(n_generated);
                generate_image(output_directory, id);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("Time elapsed {:?}", duration);
    println!(
        "Speed: {} images per second",
        n as f32 / duration.as_secs_f32()
    );

    println!("Press enter to exit");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn get_number_of_images() -> i32 {
    println!("Number of images to generate: ");
    let mut input_line = String::new();
    io::stdin()
        .read_line(&mut input_line)
        .expect("Failed to read line");
    let n = input_line.trim().parse().expect("Input not an integer");
    if n < 100 {
        println!("Number of images must be greater than 100");
        get_number_of_images()
    } else {
        n
    }
}

fn generate_image(output_directory: &str, id: i32) {
    let width = 256;
    let grid_size = 8;
    let ratio = width / grid_size as u32;
    let mut rng = rand::thread_rng();
    let random_pixel = [rng.gen(), rng.gen(), rng.gen()];
    let white_pixel = [255u8, 255u8, 255u8];
    let mut img = ImageBuffer::from_fn(width, width, |_, _| Rgb(white_pixel));
    let mut grid = vec![vec![0u8; grid_size]; grid_size];

    for i in 1..grid_size - 1 {
        for j in 1..grid_size - 1 {
            grid[i][j] = rng.gen::<u8>() % 2;
        }
    }

    for i in 0..grid_size / 2 {
        for j in 0..grid_size {
            grid[grid_size - i - 1][j] = grid[i][j];
        }
    }

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let i = x / ratio;
        let j = y / ratio;
        if grid[i as usize][j as usize] == 1 {
            *pixel = Rgb(random_pixel);
        }
    }

    let path = format!("{}{}.png", output_directory, id);
    let file = fs::File::create(path).expect("Could not create file");
    let ref mut w = io::BufWriter::new(file);

    let mut encoder = Encoder::new(w, width, width);
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(Compression::Fast);
    encoder.set_filter(FilterType::NoFilter);
    let mut writer = encoder.write_header().expect("Could not write header");

    let data: Vec<u8> = img.into_raw();
    writer.write_image_data(&data).expect("Could not write image data");
}

