use std::io::Write;
use std::fs::File;
use std::process::Command;
use std::fs;
use std::path::Path;
use std::fs::OpenOptions;

struct DataPoint {
    wattage: f64,
}

const DO_IO: bool = false;

fn run() {
    //let mut running = std::collections::VecDeque::new();
    //let mut sot_secs = load();
    let mut current_counter = std::time::Instant::now();
    let mut write_mod = 0;
    let (mut sot_secs, mut last_capacity) = load();

    let write_period = 1;

    loop {
        let w = watts();
        //running.push_back(DataPoint { wattage: amps() * volts() });

        let sleep_time = 60.0;
        //let sleep_time = 1.0;
        let sleep_hysterisis = 1.0;
        let pre_sleep_duration = current_counter.elapsed();

        let pre_sleep = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_secs_f64(sleep_time));
        //let post_sleep = std::time::Instant::now();
        

        if pre_sleep.elapsed().as_secs_f64() > sleep_time as f64 + sleep_hysterisis {
            // computer has likely slept
            sot_secs += pre_sleep_duration.as_secs_f64();
            current_counter = std::time::Instant::now();
            //println!("hit first path");
        } else {
            // everything continuing normally
            //println!("hit second path");
        }
        //check_reset(&mut sot_secs);
        if check_reset() {
            sot_secs = 0.0;
            current_counter = std::time::Instant::now();
            write_sot("Charging...\n".to_owned());
            continue;
        }
        //println!("\rsot secs: {}                      ", sot_secs + current_counter.elapsed().as_secs_f64());
        std::io::stdout().flush().unwrap();

        println!("Drawing {} watts", w);

        if DO_IO {
            if write_mod == 0 {
                write_mod = write_period;
                let time = (sot_secs + current_counter.elapsed().as_secs_f64()) as u64;
                let hours = time / (60 * 60);
                let minutes = (time / 60) % 60;
                //let seconds = time - minutes * 60 - hours * 60 * 60;
                let seconds = time % 60;
                let contents = format!("Current SOT secs is {} which comes to {}:{}:{} | {}\n",
                                       time, hours, minutes, seconds, w);
                write_sot(contents);

                if DO_IO {
                    let mut file = File::create("/home/sawyer/oss/sot/save.txt").unwrap();
                    let contents = format!("{} {}", time, last_capacity);
                    file.write_all(contents.as_bytes()).unwrap();
                }
            }
            write_mod -= 1;
        }

    }
}

fn write_sot(line: String) {
        println!("L: {}", line);
        //let mut file = File::create("/home/sawyer/oss/sot/sot.txt").unwrap();
        if DO_IO {
            let mut file = OpenOptions::new().write(true).append(true).open("/home/sawyer/oss/sot/sot.txt").expect("expected sot file");
            file.write_all(line.as_bytes()).unwrap();
        }
}

fn load() -> (f64, u64) {
    let mut time: f64 = 0.0;
    let mut charge: u64 = 0;
    //
    //let mut file = File::open("save.txt").unwrap();
    let contents = fs::read_to_string("/home/sawyer/oss/sot/save.txt");
    if let Ok(contents) = contents {
        for line in contents.lines() {
            let mut toks = line.split_whitespace();
            let time_amount = toks.next();
            if let Ok(val) = time_amount.unwrap_or("").parse() {
                time = val;
            }
            let charge_tok = toks.next();
            if let Ok(val) = charge_tok.unwrap_or("").parse() {
                charge = val;
            }
            //v = line.parse().unwrap_or(v);
        }
    }

    (time, charge)
}

fn check_reset() -> bool {
    if DO_IO {
        let path = Path::new("/home/sawyer/oss/sot/reset");
        if path.exists() {
            let _ = fs::remove_file(path);
            true
        } else if !discharging() {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn amps() -> f64 {
    let mut v = 0.0;
    if let Ok(contents) = fs::read_to_string("/sys/class/power_supply/BAT0/current_now") {
        //println!("contents: {}", contents);
        for word in contents.split_whitespace() {
            if let Ok(u) = word.parse() {
                let s: u64 = u;
                v = s as f64;
                //println!("amps is {}", s);
            }
        }
    }

    v = v / 1000000.0;

    //println!("amps returns {}", v);

    v
}

fn volts() -> f64 {
    let mut v = 0.0;
    if let Ok(contents) = fs::read_to_string("/sys/class/power_supply/BAT0/voltage_now") {
        //println!("contents: {}", contents);
        for word in contents.split_whitespace() {
            if let Ok(u) = word.parse() {
                let s: u64 = u;
                v = s as f64;
                //println!("volts is {}", s);
            }
        }
    }

    v = v / 1000000.0;

    //println!("volts returns {}", v);

    v
}

fn charge() -> u64 {
    let mut v = 0;
    if DO_IO {
        if let Ok(contents) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
            //println!("contents: {}", contents);
            for word in contents.split_whitespace() {
                if let Ok(u) = word.parse() {
                    v = u;
                    //println!("volts is {}", s);
                }
            }
        }
    }

    v
}

fn discharging() -> bool {
    if let Ok(contents) = fs::read_to_string("/sys/class/power_supply/BAT0/status") {
        if contents.contains("Discharging") { true } else { false }
    } else { false }
}

fn watts() -> String {
    let a = amps();
    let v = volts();
    let wattage = a * v;
    //println!("watts is {:.3}", wattage);

    format!("using {:.3} watts", wattage)
}

fn main() {
    run();
}
