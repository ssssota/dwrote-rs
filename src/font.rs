/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::UnsafeCell;

use comptr::ComPtr;
use winapi::um::dwrite::{IDWriteFontFace, IDWriteLocalizedStrings, IDWriteFont};
use winapi::um::dwrite::IDWriteFontFamily;
use std::mem;

use super::*;
use helpers::*;

pub struct Font {
    native: UnsafeCell<ComPtr<IDWriteFont>>,
}

impl Font {
    pub fn take(native: ComPtr<IDWriteFont>) -> Font {
        Font {
            native: UnsafeCell::new(native),
        }
    }

    pub unsafe fn as_ptr(&self) -> *mut IDWriteFont {
        (*self.native.get()).as_ptr()
    }

    pub fn to_descriptor(&self) -> FontDescriptor {
        FontDescriptor {
            family_name: self.family_name(),
            stretch: self.stretch(),
            style: self.style(),
            weight: self.weight(),
        }
    }

    pub fn stretch(&self) -> FontStretch {
        unsafe {
            mem::transmute::<u32, FontStretch>((*self.native.get()).GetStretch())
        }
    }

    pub fn style(&self) -> FontStyle {
        unsafe {
            mem::transmute::<u32, FontStyle>((*self.native.get()).GetStyle())
        }
    }

    pub fn weight(&self) -> FontWeight {
        unsafe {
            mem::transmute::<u32, FontWeight>((*self.native.get()).GetWeight())
        }
    }

    pub fn family_name(&self) -> String {
        unsafe {
            let mut family: ComPtr<IDWriteFontFamily> = ComPtr::new();
            let hr = (*self.native.get()).GetFontFamily(family.getter_addrefs());
            assert!(hr == 0);

            FontFamily::take(family).name()
        }
    }

    pub fn face_name(&self) -> String {
        unsafe {
            let mut names: ComPtr<IDWriteLocalizedStrings> = ComPtr::new();
            let hr = (*self.native.get()).GetFaceNames(names.getter_addrefs());
            assert!(hr == 0);

            get_locale_string(&mut names)
        }
    }

    pub fn create_font_face(&self) -> FontFace {
        // FIXME create_font_face should cache the FontFace and return it,
        // there's a 1:1 relationship
        unsafe {
            let mut face: ComPtr<IDWriteFontFace> = ComPtr::new();
            let hr = (*self.native.get()).CreateFontFace(face.getter_addrefs());
            assert!(hr == 0);
            FontFace::take(face)
        }
    }
}
