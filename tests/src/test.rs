use macros::bitfields;

#[bitfields]
pub struct Nested {
    pub a: B1,
    #[flag(rw, flag_type = Test)]
    pub b: B3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Test {
    One = 1,
    SomeRandomName = 2,
}

#[allow(clippy::infallible_try_from)]
impl const TryFrom<u8> for Test {
    type Error = !;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Test::One),
            2 => Ok(Test::SomeRandomName),
            _ => unreachable!(),
        }
    }
}

impl const From<Test> for u8 {
    fn from(value: Test) -> u8 {
        match value {
            Test::One => 1,
            Test::SomeRandomName => 2,
        }
    }
}
