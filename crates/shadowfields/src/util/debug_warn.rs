/// Print huge warning if debugging is enabled.
pub fn print_debug_warning() {
	#[cfg(debug_assertions)]
	println!("{DEBUG_WARNING}");
}

pub const DEBUG_WARNING: &str = r"
*************************************************
  WARNING: debug build, performance will suffer!
  Instead, build with:
  
    cargo build --release
  
  or run with:
  
    cargo run --release 
  
*************************************************
";
