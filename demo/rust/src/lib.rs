use wavedrom_rs::wavejson::WaveJson;
use wavedrom_rs::Figure;

#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *const u8 {
    Vec::with_capacity(size).leak().as_ptr() as *const u8
}

#[no_mangle]
pub extern "C" fn free(ptr: *mut u8, size: usize) {
    unsafe { Vec::from_raw_parts(ptr, 0, size) };
}

#[repr(u8)]
enum RenderError {
    JsonDeserializeError = 1,
    JsonParseError = 2,
    ShapeError = 3,
    WriteError = 4,
    InvalidUtf8 = 5,
}

fn render_internal(json: &str) -> Result<Vec<u8>, RenderError> {
    use wavedrom_rs::ToSvg;

    let Ok(wavejson) = json5::from_str::<WaveJson>(json) else {
        return Err(RenderError::JsonDeserializeError);
    };

    let Ok(figure) = Figure::try_from(wavejson) else {
        return Err(RenderError::JsonParseError);
    };
    let Ok(rendered) = figure.assemble() else {
        return Err(RenderError::ShapeError);
    };
    let mut buffer = vec![0; 5];

    let Ok(()) = rendered.write_svg(&mut buffer) else {
        return Err(RenderError::WriteError);
    };

    let size = buffer.len() - 5;
    let [b0, b1, b2, b3] = size.to_be_bytes();

    buffer[1] = b0;
    buffer[2] = b1;
    buffer[3] = b2;
    buffer[4] = b3;

    Ok(buffer)
}

#[no_mangle]
pub extern "C" fn render(ptr: *mut u8, size: usize) -> *const u8 {
    let bytes = unsafe { Vec::from_raw_parts(ptr, size, size) };
    let Ok(json) = String::from_utf8(bytes) else {
        return Box::leak(Box::new(RenderError::InvalidUtf8 as u8)) as *const u8;
    };

    match render_internal(&json[..]) {
        Ok(svg) => svg.leak().as_ptr(),
        Err(err) => Box::leak(Box::new(err as u8)) as *const u8,
    }
}
