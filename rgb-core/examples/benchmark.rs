use std::process::exit;


fn main(){
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <rom>", args[0]);
        exit(-1);
    }

    let cart = rgb_core::cart::Cart::load(&args[1], None).unwrap();
    let mut dmg = rgb_core::Dmg::new(cart);


    let cycles = 40_000_000;
    let start = std::time::Instant::now();
    let mut frames = 0;

    for _ in 0..cycles {
        if dmg.step() {
            frames += 1;
        }
    }

    let runtime = std::time::Instant::now() - start;

    println!("Runtime: {:?}, fps: {}", runtime, (frames as f64) / runtime.as_secs_f64() );
}