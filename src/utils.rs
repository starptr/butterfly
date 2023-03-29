use cfg_if::cfg_if;
use const_str;
use rand::Rng;
use serde::{Deserialize, Serialize};

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

const ALPHABET: [char; 33] = const_str::to_char_array!("0123456789abcdefghjkmnpqrstuvwxyz");

fn generate_random_slug(seed: &str) -> String {
    let mut rng = rand::thread_rng();
    (0..6).map(|_| {
        ALPHABET[rng.gen_range(0..ALPHABET.len())]
    }).collect()
}

#[derive(Serialize, Deserialize)]
pub struct PostAddBody {
    target: String,
    token: String,
}

impl PostAddBody {
    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub fn get_target(&self) -> &str {
        &self.target
    }
}
