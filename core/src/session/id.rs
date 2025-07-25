use {rand::Rng, std::fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenericSessionId<const N: usize>([u8; N]);

impl<const N: usize> GenericSessionId<N> {
    pub fn new(code: &str) -> Option<Self> {
        if !Self::validate_code(code) {
            return None;
        }

        Some(Self(
            code.to_ascii_uppercase().as_bytes().try_into().unwrap(),
        ))
    }

    pub fn random() -> Self {
        let mut buf = [0u8; N];
        let mut rng = rand::rng();
        for x in buf.iter_mut() {
            *x = rng.random_range(b'A'..=b'Z');
        }
        Self(buf)
    }

    fn validate_code(code: &str) -> bool {
        code.is_ascii() && code.len() == N
    }
}

impl<const N: usize> fmt::Display for GenericSessionId<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = str::from_utf8(&self.0).map_err(|_| fmt::Error)?;
        write!(f, "{s}")
    }
}

pub type SessionId = GenericSessionId<4>;
