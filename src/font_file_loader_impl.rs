#![allow(non_snake_case, non_upper_case_globals)]

use std::collections::HashMap;
use std::marker::Send;
use std::mem;
use std::sync::atomic::AtomicUsize;
use std::sync::{atomic, Arc, Mutex};
use winapi::shared::basetsd::UINT32;
use windows::core::Interface;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFontFile, IDWriteFontFileLoader,
    IDWriteFontFileStream, DWRITE_FACTORY_TYPE_SHARED,
};

struct FontFileLoader;

impl FontFileLoader {
    pub fn new() -> FontFileLoader {
        FontFileLoader
    }
}

unsafe impl Send for FontFileLoader {}
unsafe impl Sync for FontFileLoader {}

struct FontFileStream {
    refcount: atomic::AtomicUsize,
    key: usize,
    data: Arc<dyn AsRef<[u8]> + Sync + Send>,
}

impl FontFileStream {
    pub fn new(key: usize, data: Arc<dyn AsRef<[u8]> + Sync + Send>) -> FontFileStream {
        FontFileStream {
            refcount: AtomicUsize::new(1),
            key,
            data,
        }
    }
}

impl Drop for FontFileStream {
    fn drop(&mut self) {
        DataFontHelper::unregister_font_data(self.key);
    }
}

struct FontFileStreamPtr(IDWriteFontFileStream);

unsafe impl Send for FontFileStreamPtr {}

static mut FONT_FILE_KEY: atomic::AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct FontFileLoaderWrapper(IDWriteFontFileLoader);

unsafe impl Send for FontFileLoaderWrapper {}
unsafe impl Sync for FontFileLoaderWrapper {}

lazy_static! {
    static ref FONT_FILE_STREAM_MAP: Mutex<HashMap<usize, FontFileStreamPtr>> =
        Mutex::new(HashMap::new());
    static ref FONT_FILE_LOADER: Mutex<FontFileLoaderWrapper> = {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let ffl = FontFileLoader::new();
            let ffl = IDWriteFontFileLoader::from_raw(&ffl as *const _ as *mut _);
            factory.RegisterFontFileLoader(&ffl).unwrap();
            Mutex::new(FontFileLoaderWrapper(ffl))
        }
    };
}

pub(crate) struct DataFontHelper;

impl DataFontHelper {
    pub(crate) fn register_font_buffer(
        font_data: Arc<dyn AsRef<[u8]> + Sync + Send>,
    ) -> (IDWriteFontFile, IDWriteFontFileStream, usize) {
        unsafe {
            let key = FONT_FILE_KEY.fetch_add(1, atomic::Ordering::Relaxed);
            let font_file_stream_native = FontFileStream::new(key, font_data);
            let font_file_stream =
                IDWriteFontFileStream::from_raw(&font_file_stream_native as *const _ as *mut _);

            {
                let mut map = FONT_FILE_STREAM_MAP.lock().unwrap();
                map.insert(key, FontFileStreamPtr(font_file_stream.clone()));
            }

            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let loader = FONT_FILE_LOADER.lock().unwrap();
            let font_file = factory
                .CreateCustomFontFileReference(
                    mem::transmute(&key),
                    mem::size_of::<usize>() as UINT32,
                    &loader.0,
                )
                .unwrap();

            (font_file, font_file_stream, key)
        }
    }

    fn unregister_font_data(key: usize) {
        let mut map = FONT_FILE_STREAM_MAP.lock().unwrap();
        if map.remove(&key).is_none() {
            panic!("unregister_font_data: trying to unregister key that is no longer registered");
        }
    }
}
