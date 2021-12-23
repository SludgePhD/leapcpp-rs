//! Access to raw camera images.
//!
//! Receiving raw camera data requires enabling [`Policy::Images`][crate::Policy::Images].

use std::{fmt, mem::MaybeUninit};

use crate::{sys, Timestamp};

/// A list of raw camera images recorded by the Leap Motion Controller.
pub struct ImageList {
    raw: Box<sys::Leap_ImageList>,
}

impl ImageList {
    pub(crate) fn from_raw(raw: Box<sys::Leap_ImageList>) -> Self {
        Self { raw }
    }

    /// Returns the number of images in the list.
    pub fn len(&self) -> usize {
        unsafe { sys::Leap_ImageList_count(&*self.raw) as usize }
    }

    /// Returns whether this list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the images in the list.
    pub fn iter(&self) -> ImageListIterator<'_> {
        ImageListIterator {
            list: self,
            next: 0,
            len: self.len(),
        }
    }
}

impl Drop for ImageList {
    fn drop(&mut self) {
        // No `ImageList` destructor, call superclass dtor instead.
        unsafe {
            sys::Leap_Interface_Interface_destructor((&mut *self.raw) as *mut _ as _);
        }
    }
}

/// An iterator over the [`Image`]s in an [`ImageList`].
pub struct ImageListIterator<'a> {
    list: &'a ImageList,
    next: usize,
    len: usize,
}

impl<'a> Iterator for ImageListIterator<'a> {
    type Item = Image;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.len {
            None
        } else {
            unsafe {
                let mut image = Box::new(MaybeUninit::uninit());
                sys::Leap_RustGetImage(image.as_mut_ptr(), &*self.list.raw, self.next as i32);
                self.next += 1;
                Some(Image {
                    inner: crate::init_box(image),
                })
            }
        }
    }
}

/// A raw camera image, alongside calibration data.
pub struct Image {
    inner: Box<sys::Leap_Image>,
}

impl Image {
    pub fn is_valid(&self) -> bool {
        unsafe { sys::Leap_Image_isValid(&*self.inner) }
    }

    pub fn sequence_id(&self) -> i64 {
        unsafe { sys::Leap_Image_sequenceId(&*self.inner) }
    }

    pub fn camera(&self) -> Camera {
        let id = unsafe { sys::Leap_Image_id(&*self.inner) };

        match id {
            0 => Camera::Left,
            1 => Camera::Right,
            _ => unreachable!("encountered invalid image ID {}", id),
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        let raw = unsafe { sys::Leap_Image_timestamp(&*self.inner) };
        Timestamp::from_raw(raw)
    }

    pub fn width(&self) -> usize {
        unsafe { sys::Leap_Image_width(&*self.inner) as usize }
    }

    pub fn height(&self) -> usize {
        unsafe { sys::Leap_Image_height(&*self.inner) as usize }
    }

    pub fn bytes_per_pixel(&self) -> usize {
        unsafe { sys::Leap_Image_bytesPerPixel(&*self.inner) as usize }
    }

    pub fn data(&self) -> ImageData<'_> {
        ImageData {
            raw: self.raw_data(),
            width: self.width(),
        }
    }

    pub fn raw_data(&self) -> &[u8] {
        let len = self.width() * self.height() * self.bytes_per_pixel();

        unsafe {
            let ptr = sys::Leap_Image_data(&*self.inner);
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn distortion(&self) -> DistortionData<'_> {
        DistortionData {
            raw: self.raw_distortion(),
            stride: self.distortion_stride(),
        }
    }

    pub fn raw_distortion(&self) -> &[f32] {
        unsafe {
            let ptr = sys::Leap_Image_distortion(&*self.inner);
            std::slice::from_raw_parts(ptr, self.distortion_stride() * self.distortion_height())
        }
    }

    pub fn distortion_width(&self) -> usize {
        64
    }

    /// Returns the number of `f32` elements in each row of the distortion map.
    pub fn distortion_stride(&self) -> usize {
        self.distortion_width() * 2
    }

    pub fn distortion_height(&self) -> usize {
        64
    }
}

/// The pixel data comprising a camera image.
pub struct ImageData<'a> {
    raw: &'a [u8],
    width: usize,
}

impl<'a> ImageData<'a> {
    /// Returns the raw image data as a slice of bytes.
    pub fn raw(&self) -> &'a [u8] {
        self.raw
    }

    /// Returns the pixel value at the given coordinates.
    pub fn pixel(&self, x: usize, y: usize) -> u8 {
        self.raw[y * self.width + x]
    }

    /// Returns an iterator over the rows of image data.
    pub fn rows(&self) -> impl Iterator<Item = &'_ [u8]> {
        self.raw.chunks(self.width)
    }
}

/// The data comprising the distortion calibration data.
///
/// The distortion map is a low-resolution image where every pixel contains an image coordinate
/// pointing into the raw camera image.
pub struct DistortionData<'a> {
    raw: &'a [f32],
    stride: usize,
}

impl<'a> DistortionData<'a> {
    pub fn width(&self) -> usize {
        self.stride / 2
    }

    pub fn height(&self) -> usize {
        self.raw.len() / self.stride
    }

    /// Returns the raw distortion data as a slice of `f32`s.
    pub fn raw(&self) -> &'a [f32] {
        self.raw
    }

    pub fn rows(&self) -> impl Iterator<Item = DistortionDataRow<'a>> {
        self.raw
            .chunks(self.stride)
            .map(|row| DistortionDataRow { row })
    }
}

impl fmt::Debug for DistortionData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.rows()).finish()
    }
}

/// A row of data in the distortion map.
pub struct DistortionDataRow<'a> {
    row: &'a [f32],
}

impl<'a> DistortionDataRow<'a> {
    pub fn entries(&self) -> impl Iterator<Item = DistortionEntry> + '_ {
        self.row.chunks(2).map(|entry| match entry {
            [u, v] => DistortionEntry { u: *u, v: *v },
            _ => unreachable!(),
        })
    }
}

impl fmt::Debug for DistortionDataRow<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.entries()).finish()
    }
}

/// An entry in the distortion map.
///
/// Each entry contains the U/V texture coordinates to use for looking up the corresponding pixels
/// in the raw camera image. Since the distortion map is smaller than the camera image, the entries
/// need to be linearly interpolated.
pub struct DistortionEntry {
    pub u: f32,
    pub v: f32,
}

impl DistortionEntry {
    /// Returns whether this distortion map entry is valid.
    ///
    /// If this returns `false`, there is no valid camera data for this area of the image.
    pub fn is_valid(&self) -> bool {
        self.u >= 0.0 && self.u <= 1.0 && self.v >= 0.0 && self.v <= 1.0
    }
}

impl fmt::Debug for DistortionEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.u, self.v)
    }
}

/// Identifies one of the cameras on the Leap Motion Controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Camera {
    Left,
    Right,
}
