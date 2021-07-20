use rand::{distributions::Alphanumeric, Rng};

pub fn slug(length: u8) -> String {
    /*! Generate a random [`Alphanumeric`] slug of size `length` */

    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect()
}
