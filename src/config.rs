//mod json_config;
use nix::unistd::Uid;
use std::io;

// netoe1-mod: config for hard exiting
// fn error_handler() -> Result<(), Box<dyn std::error::Error>> {

//     let err = true;

//     if err {
//         return Err("error".into());
//     }

//     Ok(())
// }

// netoe1-mod: linux things. Will be documented later!
// Uses functions from nix, and verify if the program is running by sudo.
// If isn't, exit without cleaning memory! (i'll make it safe later.)

pub fn check_if_has_root_access() {

    if !Uid::effective().is_root(){
        println!("mx002-config-err: You must run a sudo!");
        std::process::exit(-1);
    } 
}


// netoe-mod:
// Pen pressure sensitivity configuration
//
// PEN_THRESHOLD:
//   Controls how close the pen must be to the tablet surface before
//   pressure processing starts.
//
//   This value defines the proximity cutoff. When the pen is hovering
//   close to the surface but not actually touching it, the raw pressure
//   value remains within this threshold range and is treated as zero.
//
//   Lower values:
//     - Pen activates earlier (more sensitive proximity detection)
//     - May cause unintended touches when hovering
//
//   Higher values:
//     - Pen must be closer / slightly pressing to activate
//     - Reduces false touch events
//
//   Typical values:
//     - 300–500 : Very sensitive, light hover detection
//     - 600–800 : Balanced (default range)
//     - 900+    : Requires firm contact to activate
//
//
// PEN_STRENGTH_SCALING:
//   Controls how much physical force is required for the touch/pressure
//   to be considered strong.
//
//   This value scales the normalized pressure after the proximity
//   threshold has been passed. It directly affects how fast the pressure
//   value increases once the pen is touching the surface.
//
//   Lower values:
//     - Requires more physical force for strong pressure
//     - Produces smoother, softer pressure curves
//
//   Higher values:
//     - Strong pressure is reached with less physical force
//     - Makes strokes feel heavier and more aggressive
//
//   Typical values:
//     - 1       : Very soft pressure response
//     - 2–3     : Balanced pressure curve (recommended)
//     - 4–5     : Very sensitive, strong pressure with light force
//
// These parameters work together to define the pen feel.
// PEN_THRESHOLD determines *when* the pen starts touching,
// PEN_STRENGTH_SCALING determines *how hard* the pen presses.

// Putting defaults in a file, that you can change later, based in a pattern.

use std::sync::atomic::{AtomicI32, Ordering};

// Default values
pub const PEN_THRESHOLD_STD: i32 = 600;
pub const PEN_STRENGTH_STD: i32 = 2;

// Global mutable configuration (thread-safe)
pub static PEN_THRESHOLD: AtomicI32 = AtomicI32::new(PEN_THRESHOLD_STD);
pub static PEN_STRENGTH_SCALING: AtomicI32 = AtomicI32::new(PEN_STRENGTH_STD);

// Helper setters/getters (clean API)
pub fn set_pen_threshold(value: i32) {
    PEN_THRESHOLD.store(value, Ordering::Relaxed);
}

pub fn get_pen_threshold() -> i32 {
    PEN_THRESHOLD.load(Ordering::Relaxed)
}

pub fn set_pen_strength(value: i32) {
    PEN_STRENGTH_SCALING.store(value, Ordering::Relaxed);
}


pub fn get_pen_strength() -> i32 {
    PEN_STRENGTH_SCALING.load(Ordering::Relaxed)
}

pub fn output_pen_config(){
    println!("mx002-info: Pen Strength value = {}",get_pen_strength());
    println!("mx002-info: Pen Threshold strength = {}",get_pen_threshold());
}
// #endregion DESKPEN_CONFIG


// #region CONFIG_PROGRAM_INPUTS
// netoe-mod: Config Program INPUTs
// The program inputs will determine how actually the driver behavior.
// First of all, in this version, I'm trying to use only CLI or Input while execution, to make some tests.

// In older codes, There is some raw code, so, I'll encapsulate some code over here, to make easier understanding for people.
// You can call this abstraction or whatever you want to call.

pub fn while_executing_input(){
    // netoe1-mod: Adding config interface
    // These values control pen proximity and pressure sensitivity

    println!("mx002-driver-info: Using input while executing way");

    let mut get_threshold_proximity = String::new();
    let mut get_strength_scaling = String::new();

    // --- Threshold ---
    println!("Set threshold proximity [0-1700], recommended 600:");
    io::stdin()
        .read_line(&mut get_threshold_proximity)
        .expect("mx002-driver-err: Failed to read threshold");

    let threshold: i32 = get_threshold_proximity
        .trim()
        .parse()
        .expect("mx002-driver-err: Threshold must be an integer");

    // Optional safety clamp
    let threshold = threshold.clamp(0, 1700);

    // --- Strength ---
    println!("Set strength scaling [1-10], recommended 5:");
    io::stdin()
        .read_line(&mut get_strength_scaling)
        .expect("mx002-driver-err: Failed to read strength");

    let strength: i32 = get_strength_scaling
        .trim()
        .parse()
        .expect("mx002-driver-err: Strength must be an integer");

    let strength = strength.clamp(1, 10);

    // println!("mx002-driver-log: Values typed below!");
    // println!("mx002-driver-log: Configured sens.");
    // println!("mx002-driver-log: Threshold proximity = {}", threshold);
    // println!("mx002-driver-log: Strength scaling = {}", strength);

    // Setting values.
    set_pen_strength(strength);
    set_pen_threshold(threshold);
    get_pen_strength();
    get_pen_threshold();

}

// netoe1-mod: Here, this is it the json mode, that will be function.

pub fn json_input(raw_data: String){
    //json_config::read_cfg_file("");
}

// #endregion CONFIG_PROGRAM_INPUTS

