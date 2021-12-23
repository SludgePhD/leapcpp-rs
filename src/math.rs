use crate::sys;

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Vector {
    inner: sys::Leap_Vector,
}

impl Vector {
    #[inline]
    pub fn x(&self) -> f32 {
        self.inner.x
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.inner.y
    }

    #[inline]
    pub fn z(&self) -> f32 {
        self.inner.z
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Matrix {
    inner: sys::Leap_Matrix,
}
