fn nanoid() {
    use e_utils::random;
    println!("nanoid -> {}", random!(nanoid));
    println!("nanoid 16bytes -> {}", random!(nanoid 16));
    println!("nanoid 16bytes -> {}", random!(nanoid 16));
    println!(
        "nanoid 10bytes [alphabet:expr] -> {}",
        random!(nanoid 16, &['1', 'b', 'c', '7'])
    );
    println!("nanoid unsafe 10bytes -> {}", random!(nanoid unsafe 10));
    println!(
        "nanoid unsafe 10bytes [alphabet:expr]-> {}",
        random!(nanoid unsafe 10, &['1','0'])
    );
}

fn std() {
    use e_utils::random;
    println!("random bool -> {}", random!());
    println!("random type -> {}", random!(#u32));
    println!("random type[] -> {:?}", random!([u32; 10]));
    println!("random range 0-13 -> {}", random!(13i64));
    println!("random range -> {}", random!(0i32..13));
    println!("random rgb range -> {:?}", random!(rgb 100,255));
}
fn main() {
    nanoid();
    std();
}
