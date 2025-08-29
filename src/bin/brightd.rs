use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;

const BR_PATH: &str = "/sys/devices/platform/mtk_leds/leds/lcd-backlight/brightness";

// System max brightness (from Android UI)
const SYS_MAX: i32 = 418;

// Hardware range (panel real values)
const HW_MIN: i32 = 10;
const HW_MAX: i32 = 4095;

fn map_curve(percent: i32) -> i32 {
    match percent {
        0..=5   => HW_MIN,
        6..=10  => 200,
        11..=20 => 600,
        21..=40 => 1800,
        41..=60 => 2800,
        61..=80 => 3500,
        _       => HW_MAX,
    }
}

fn read_brightness() -> Option<i32> {
    let mut buf = String::new();
    if let Ok(mut file) = File::open(BR_PATH) {
        if file.read_to_string(&mut buf).is_ok() {
            return buf.trim().parse::<i32>().ok();
        }
    }
    None
}

fn write_brightness(value: i32) {
    if let Ok(mut file) = OpenOptions::new().write(true).open(BR_PATH) {
        let _ = file.write_all(value.to_string().as_bytes());
    }
}

fn smooth_set(target: i32, current: i32) {
    let diff = target - current;
    let steps = 10;
    let mut step = diff / steps;

    if step == 0 {
        step = if diff > 0 { 1 } else { -1 };
    }

    let mut value = current;
    for _ in 0..steps {
        value += step;
        write_brightness(value);
        sleep(Duration::from_millis(10));
    }
    write_brightness(target);
}

fn main() {
    let mut last_ui_value = -1;
    let mut last_mapped = -1;

    loop {
        if let Some(cur_ui) = read_brightness() {
            // Ignore out-of-range or no-change
            if cur_ui > SYS_MAX || cur_ui == last_ui_value {
                sleep(Duration::from_millis(50));
                continue;
            }

            // Convert UI brightness → percent
            let percent = cur_ui * 100 / SYS_MAX;
            let target = map_curve(percent);

            // Write only if mapped value changed
            if target != last_mapped {
                if let Some(current_hw) = read_brightness() {
                    smooth_set(target, current_hw);
                }
                last_mapped = target;
            }

            last_ui_value = cur_ui;
        }

        sleep(Duration::from_millis(50));
    }
}
