use base64::{engine::general_purpose, Engine as _};

// ─── macOS Vision OCR ──────────────────────────────────────────────────────
//
//  Call chain:
//
//    recognize_text_in_base64_image(base64_png)
//      │
//      ├── base64::decode()          Err → propagate
//      ├── PNG magic bytes check     Err → "not a PNG"
//      │
//      └── unsafe Vision block
//            ├── NSData::from_vec(bytes)
//            ├── NSDictionary::new()               (empty options)
//            ├── VNImageRequestHandler::alloc()
//            │     .initWithData_options(&data, &opts)
//            ├── VNRecognizeTextRequest::new()
//            │     .setRecognitionLevel(Accurate)
//            ├── NSArray::from_retained_slice(&[request.into VNRequest])
//            ├── handler.performRequests_error(&array)   Err → "OCR_UNAVAILABLE"
//            ├── request.results()                       None → "NO_TEXT"
//            └── observations → topCandidates(1) → join '\n'
//                  empty → "NO_TEXT"

#[cfg(target_os = "macos")]
pub fn recognize_text_in_base64_image(base64_png: &str) -> Result<String, String> {
    use objc2::AnyThread;
    use objc2_foundation::{NSArray, NSData, NSDictionary, NSString};
    use objc2_vision::{
        VNImageRequestHandler, VNRecognizeTextRequest, VNRequest,
        VNRequestTextRecognitionLevel,
    };

    // 1. Reject oversized input before decoding (50 MB raw ≈ ~67 MB base64).
    const MAX_BASE64_LEN: usize = 67_000_000;
    if base64_png.len() > MAX_BASE64_LEN {
        return Err("Image too large for OCR".to_string());
    }

    // 2. Base64 decode
    let bytes = general_purpose::STANDARD
        .decode(base64_png)
        .map_err(|e| format!("base64 decode error: {e}"))?;

    if bytes.is_empty() {
        return Err("NO_TEXT".to_string());
    }

    // 2. Validate PNG magic bytes so we return a meaningful error before
    //    calling into Vision with garbage data.
    if bytes.len() < 8 || bytes[0..8] != *b"\x89PNG\r\n\x1a\n" {
        return Err("not a PNG".to_string());
    }

    // 3. Vision FFI — objc2 0.6 wraps unsafety internally.
    // NSData takes ownership of the bytes (copies internally).
    let ns_data = NSData::from_vec(bytes);

    // Empty options dictionary — no special image metadata needed.
    let options = NSDictionary::<NSString, objc2::runtime::AnyObject>::new();

    // Build the image request handler from the raw PNG bytes.
    let handler = VNImageRequestHandler::initWithData_options(
        VNImageRequestHandler::alloc(),
        &ns_data,
        &options,
    );

    // Create the text recognition request at maximum accuracy.
    let request = VNRecognizeTextRequest::new();
    request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);

    // performRequests_error takes &NSArray<VNRequest>.
    // VNRecognizeTextRequest -> VNImageBasedRequest -> VNRequest (two into_super calls).
    let req_as_vn: objc2::rc::Retained<VNRequest> =
        request.clone().into_super().into_super();
    let requests_array = NSArray::from_retained_slice(&[req_as_vn]);

    // Run Vision synchronously on the calling thread (spawn_blocking caller).
    handler
        .performRequests_error(&requests_array)
        .map_err(|_| "OCR_UNAVAILABLE".to_string())?;

    // Collect results — None means Vision found no text regions at all.
    let observations = match request.results() {
        Some(obs) if !obs.is_empty() => obs,
        _ => return Err("NO_TEXT".to_string()),
    };

    // Extract the top candidate from each observation, trim, drop blanks.
    let mut lines: Vec<String> = Vec::new();
    for obs in &*observations {
        let candidates = obs.topCandidates(1);
        for candidate in &*candidates {
            let s = candidate.string().to_string();
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                lines.push(trimmed);
            }
            break; // topCandidates(1) — only ever one
        }
    }

    if lines.is_empty() {
        return Err("NO_TEXT".to_string());
    }

    let text = lines.join("\n");

    Ok(text)
}

#[cfg(not(target_os = "macos"))]
pub fn recognize_text_in_base64_image(_base64_png: &str) -> Result<String, String> {
    Err("OCR_UNAVAILABLE".to_string())
}

/// Tauri IPC command — runs Vision OCR on a base64-encoded PNG.
/// Uses spawn_blocking so Vision inference (~100-300ms) does not block
/// the Tokio async runtime.
#[tauri::command]
pub async fn recognize_text(image_base64: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || recognize_text_in_base64_image(&image_base64))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_base64_returns_error() {
        assert!(recognize_text_in_base64_image("not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_non_png_bytes_returns_error() {
        let b64 = general_purpose::STANDARD.encode(b"not a png at all");
        let result = recognize_text_in_base64_image(&b64);
        assert!(result.is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_blank_png_returns_no_text() {
        use image::DynamicImage;
        use std::io::Cursor;
        let img = DynamicImage::new_rgb8(100, 100);
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let b64 = general_purpose::STANDARD.encode(buf.get_ref());
        assert_eq!(
            recognize_text_in_base64_image(&b64),
            Err("NO_TEXT".to_string())
        );
    }

    /// Full pipeline test: feed a real PNG with "Hello" text, assert Vision
    /// returns Ok with non-empty recognized text.
    ///
    /// Fixture generated with:
    ///   swift -e 'import AppKit; ... "Hello".draw(at:withAttributes:) ...'
    #[cfg(target_os = "macos")]
    #[test]
    fn test_png_with_text_returns_ok() {
        let png_bytes = include_bytes!("test_fixtures/hello.png");
        let b64 = general_purpose::STANDARD.encode(png_bytes);
        let result = recognize_text_in_base64_image(&b64);
        assert!(
            result.is_ok(),
            "Expected OCR to succeed, got: {:?}",
            result
        );
        let text = result.unwrap();
        assert!(!text.is_empty(), "Expected non-empty OCR result");
    }
}
