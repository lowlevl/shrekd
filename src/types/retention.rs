use rocket::data::ByteUnit;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("The minimum age must be lower than the maximum age, but here {0} > {1}")]
    BoundDefinition(u64, u64),
}

#[derive(Debug)]
pub struct RetentionCurve {
    min_age: u64,
    max_age: u64,
    max_size: u64,
}

impl RetentionCurve {
    /** Create a new [`RetentionCurve`] from it's parameters */
    #[inline]
    pub const fn new(min_age: u64, max_age: u64, max_size: u64) -> Result<Self, Error> {
        if min_age > max_age {
            return Err(Error::BoundDefinition(min_age, max_age));
        }

        Ok(Self {
            min_age,
            max_age,
            max_size,
        })
    }

    /** Get the max age in seconds of the [`RetentionCurve`]*/
    #[inline]
    pub const fn max(&self) -> u64 {
        self.max_age
    }

    /** Compute the expiry time from the inner parameters */
    pub fn compute_for(&self, size: u64) -> u64 {
        /* If we are on a bound, there is nothing to plot */
        if size > self.max_size {
            return self.min_age;
        }

        let window = self.max_age - self.min_age;

        let plot = f64::powi(size as f64 / self.max_size as f64, 2);
        let retention = self.max_age as f64 - window as f64 * plot;

        tracing::trace!(
            "Computed a retention of `{}` seconds for a size of {}",
            retention,
            ByteUnit::from(size)
        );

        retention as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bound_definition_error() {
        let error = RetentionCurve::new(1, 0, 0).unwrap_err();

        assert_eq!(error, Error::BoundDefinition(1, 0));
    }

    #[test]
    fn zero_size_is_max_age() {
        let curve = RetentionCurve::new(604800, 1814400, 128000000).unwrap();

        assert_eq!(curve.compute_for(0), 1814400);
    }

    #[test]
    fn max_size_is_min_age() {
        let curve = RetentionCurve::new(604800, 1814400, 128000000).unwrap();

        assert_eq!(curve.compute_for(128000000), 604800);
    }

    #[test]
    fn more_than_max_size_is_min_age() {
        let curve = RetentionCurve::new(604800, 1814400, 128000000).unwrap();

        assert_eq!(curve.compute_for(512000000), 604800);
    }

    #[test]
    fn curve_is_exponential() {
        let curve = RetentionCurve::new(0, 10000, 10000).unwrap();

        assert_eq!(curve.compute_for(2500), 9375);
        assert_eq!(curve.compute_for(5000), 7500);
        assert_eq!(curve.compute_for(7500), 4375);
    }
}
