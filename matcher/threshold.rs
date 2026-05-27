/// Basic threshold tracker for future alert suppression.
/// Will be integrated with the classifier in later phases.
#[derive(Debug)]
pub struct Threshold {
    pub limit: u32,
    pub current: u32,
}

impl Threshold {
    pub fn new(limit: u32) -> Self {
        Self { limit, current: 0 }
    }

    /// Increment and check if the threshold has been reached.
    pub fn bump(&mut self) -> bool {
        self.current += 1;
        self.current >= self.limit
    }

    pub fn reset(&mut self) {
        self.current = 0;
    }
}