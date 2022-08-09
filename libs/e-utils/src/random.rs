use rand::{
    rngs::{SmallRng, StdRng},
    Rng, SeedableRng,
};
use std::fmt::Write;

#[macro_export]
#[doc(hidden)]
macro_rules! rgb_format {
    () => (panic!("internal error: entered unreachable code"));
    // 自定义RGB
    (rgb[$rgb_f:expr, $rgb_b:expr] $fmt:expr) => ($crate::random::rgb_format($rgb_f, $rgb_b, format_args!("{}\x1B[0m",$fmt)));
    // 返回干净变色代码
    (pure $fmt:expr) => ($crate::rgb_format!(rgb[None, None] $fmt));
    // 常规值
    ($fmt:expr) => ($crate::rgb_format!(random_f format_args!("{}\x1B[0m", $fmt)));
    // 格式化颜色
    (random $fmt:expr) => {$crate::random::rgb_format(Some($crate::random::Rand::Safe.rgb_range(100, 255)), Some($crate::random::Rand::Safe.rgb_range(100,255)), $fmt)};
    (random_f $fmt:expr) => {$crate::random::rgb_format(Some($crate::random::Rand::Safe.rgb_range(100, 255)), None, $fmt)};
    (random_b $fmt:expr) => {$crate::random::rgb_format(None, Some($crate::random::Rand::Safe.rgb_range(100,255)), $fmt)};
    // 干净的颜色结果
    (pure $($arg:tt)*) => {{$crate::rgb_format!(rgb[None, None] format_args!($($arg)*))}};
    // 携带标识
    ($($arg:tt)*) => {{$crate::rgb_format!(random_f format_args!("{}\x1B[0m", format_args!($($arg)*)))}};
}

/// # Random 工具宏
///
/// # Exmaple `Nanoid`
/// ```no_run
/// fn main() {
///     use e_utils::random;
///     println!("nanoid -> {}", random!(nanoid));
///     println!("nanoid 16bytes -> {}", random!(nanoid 16));
///     println!("nanoid 16bytes -> {}", random!(nanoid 16));
///     println!("nanoid 10bytes [alphabet:expr] -> {}", random!(nanoid 16, &['1', 'b', 'c', '7']));
///     println!("nanoid unsafe 10bytes -> {}", random!(nanoid unsafe 10));
///     println!("nanoid unsafe 10bytes [alphabet:expr]-> {}", random!(nanoid unsafe 10, &['1','0']));
/// }
/// ```
///
/// # Exmaple `random`
/// ```no_run
/// fn main() {
///    use e_utils::random;
///    println!("random bool -> {}", random!());
///    println!("random type -> {}", random!(#u32));
///    println!("random type[] -> {:?}", random!([u32; 10]));
///    println!("random range 0-13 -> {}", random!(13));
///    println!("random range -> {}", random!(0i32..13));
///    println!("random rgb range -> {:?}", random!(rgb 100,255));
/// }
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! random {
    () => {
        $crate::random::Rand::Safe.random_bool()
    };
    (#$t:ty) => {
        $crate::random::Rand::Safe.random_type::<$t>()
    };
    ([$t:ty; $size:expr]) => {
        $crate::random::Rand::Safe.random_type::<[$t; $size]>()
    };
    (rgb $min:tt, $max:tt) => {
        $crate::random::Rand::Safe.rgb_range($min, $max)
    };
    (nanoid) => {
        $crate::random::Rand::Safe.nanoid_format(&$crate::random::NID_SAFE, 21)
    };
    // generate
    (nanoid $size:tt) => {
        $crate::random::Rand::Safe.nanoid_format(&$crate::random::NID_SAFE, $size)
    };
    // custom
    (nanoid $size:tt, $alphabet:expr) => {
        $crate::random::Rand::Safe.nanoid_format($alphabet, $size)
    };
    // unsafe
    (nanoid unsafe $size:tt) => {
        $crate::random::Rand::UnSafe.nanoid_format(&$crate::random::NID_SAFE, $size)
    };
    // unsafe custom
    (nanoid unsafe $size:tt, $alphabet:expr) => {
        $crate::random::Rand::UnSafe.nanoid_format($alphabet, $size)
    };
    ($size:tt) => {
        $crate::random::Rand::Safe.random_range(0..$size)
    };
    ($min:tt..$max:tt) => {
        $crate::random::Rand::Safe.random_range($min..$max)
    };
}
/// URL safe symbols.
///
/// An array of characters which can be safely used in urls.
/// Alphabet default for Rand. Is alphabet by default for Rand
///
/// # Example
///
/// ```
/// let id = Rand::Rand!(10, &Rand::alphabet::NID_SAFE);
/// ```
pub const NID_SAFE: [char; 64] = [
    '_', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub fn rgb_format(
    rgb_f: Option<(u8, u8, u8)>,
    rgb_b: Option<(u8, u8, u8)>,
    args: std::fmt::Arguments<'_>,
) -> String {
    let rf = rgb_f
        .and_then(|x| Some(format!("38;2;{};{};{}", x.0, x.1, x.2)))
        .unwrap_or("".to_owned());
    let rb = rgb_b
        .and_then(|x| Some(format!("48;2;{};{};{};", x.0, x.1, x.2)))
        .unwrap_or("".to_owned());
    let mut s = String::new();
    let _ = s.write_fmt(format_args!("\x1B[{}{}m", rb, rf));
    let _ = s.write_fmt(args);
    s
}

pub enum Rand {
    Safe,
    UnSafe,
}
impl Rand {
    pub fn random_type<T>(&self) -> T
    where
        rand::distributions::Standard: rand::prelude::Distribution<T>,
    {
        match &self {
            Rand::Safe => StdRng::from_entropy().gen::<T>(),
            Rand::UnSafe => SmallRng::from_entropy().gen::<T>(),
        }
    }
    pub fn random_bool(&self) -> bool {
        match &self {
            Rand::Safe => StdRng::from_entropy().gen_range(1u8..=2) == 1,
            Rand::UnSafe => SmallRng::from_entropy().gen_range(1u8..=2) == 1,
        }
    }
    pub fn rgb_range(&self, min: u8, max: u8) -> (u8, u8, u8) {
        match &self {
            Rand::Safe => {
                let mut rng = StdRng::from_entropy();
                (
                    rng.gen_range(min..max),
                    rng.gen_range(min..max),
                    rng.gen_range(min..max),
                )
            }
            Rand::UnSafe => {
                let mut rng = SmallRng::from_entropy();
                (
                    rng.gen_range(min..max),
                    rng.gen_range(min..max),
                    rng.gen_range(min..max),
                )
            }
        }
    }

    pub fn random_range<T, R>(&self, range: R) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
        R: rand::distributions::uniform::SampleRange<T>,
    {
        match &self {
            Rand::Safe => StdRng::from_entropy().gen_range(range),
            Rand::UnSafe => SmallRng::from_entropy().gen_range(range),
        }
    }
    pub fn random_rng(&self, step: usize) -> Vec<u8> {
        match &self {
            Rand::Safe => {
                let mut rng = StdRng::from_entropy();
                let mut result = vec![0u8; step];
                rng.fill(&mut result[..]);
                result
            }
            Rand::UnSafe => {
                let mut rng = SmallRng::from_entropy();
                let mut result = vec![0u8; step];
                rng.fill(&mut result[..]);
                result
            }
        }
    }
    pub fn nanoid_format(&self, alphabet: &[char], size: usize) -> String {
        assert!(
            alphabet.len() <= u8::max_value() as usize,
            "The alphabet cannot be longer than a `u8` (to comply with the `random` function)"
        );
        let mask = alphabet.len().next_power_of_two() - 1;
        let step: usize = 8 * size / 5;
        // Assert that the masking does not truncate the alphabet. (See #9)
        debug_assert!(alphabet.len() <= mask + 1);
        let mut id = String::with_capacity(size);
        loop {
            let bytes = self.random_rng(step);
            for &byte in &bytes {
                let byte = byte as usize & mask;

                if alphabet.len() > byte {
                    id.push(alphabet[byte]);

                    if id.len() == size {
                        return id;
                    }
                }
            }
        }
    }
}
