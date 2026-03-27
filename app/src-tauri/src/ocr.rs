use base64::{engine::general_purpose, Engine as _};

/// Returns true if running macOS 13 (Ventura) or later.
/// Used to guard APIs introduced in macOS 13 (e.g. `setAutomaticallyDetectsLanguage`).
/// Cached via OnceLock — `sw_vers` is only spawned once per process lifetime.
#[cfg(target_os = "macos")]
fn is_macos_13_or_later() -> bool {
    use std::sync::OnceLock;
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        use std::process::Command;
        Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|v| v.trim().split('.').next().map(String::from))
            .and_then(|major| major.parse::<u32>().ok())
            .map_or(false, |major| major >= 13)
    })
}

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
pub fn recognize_text_in_base64_image(
    base64_png: &str,
    languages: &[String],
) -> Result<String, String> {
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

    // 3. Validate PNG magic bytes so we return a meaningful error before
    //    calling into Vision with garbage data.
    if bytes.len() < 8 || bytes[0..8] != *b"\x89PNG\r\n\x1a\n" {
        return Err("not a PNG".to_string());
    }

    // 4. Vision FFI — objc2 0.6 wraps unsafety internally.
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

    // Enable automatic language detection (macOS 13+ / revision 3).
    // This lets Vision auto-detect Chinese, Japanese, Korean, Latin, etc.
    // without the caller needing to specify languages explicitly.
    // Guard: the selector does not exist on macOS 12, so calling it would crash.
    if is_macos_13_or_later() {
        request.setAutomaticallyDetectsLanguage(true);
    }

    // If the caller specified preferred languages, set them.
    // The order defines priority during recognition.
    // Common codes: "zh-Hans" (Simplified Chinese), "zh-Hant" (Traditional Chinese),
    // "en-US", "ja-JP", "ko-KR", "de-DE", "fr-FR", "es-ES", etc.
    if !languages.is_empty() {
        let ns_langs: Vec<objc2::rc::Retained<NSString>> = languages
            .iter()
            .map(|lang| NSString::from_str(lang))
            .collect();
        let ns_refs: Vec<&NSString> = ns_langs.iter().map(|s| s.as_ref()).collect();
        let lang_array = NSArray::from_slice(&ns_refs);
        request.setRecognitionLanguages(&lang_array);
    }

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
        if let Some(candidate) = candidates.iter().next() {
            let s = candidate.string().to_string();
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                lines.push(trimmed);
            }
        }
    }

    if lines.is_empty() {
        return Err("NO_TEXT".to_string());
    }

    let text = lines.join("\n");

    Ok(text)
}

#[cfg(not(target_os = "macos"))]
pub fn recognize_text_in_base64_image(
    _base64_png: &str,
    _languages: &[String],
) -> Result<String, String> {
    Err("OCR_UNAVAILABLE".to_string())
}

/// Query the list of languages supported by macOS Vision OCR.
/// Returns ISO language codes like "en-US", "zh-Hans", "zh-Hant", "ja-JP", etc.
#[cfg(target_os = "macos")]
pub fn get_supported_languages() -> Result<Vec<String>, String> {
    use objc2_vision::VNRecognizeTextRequest;

    let request = VNRecognizeTextRequest::new();
    let langs = unsafe {
        request
            .supportedRecognitionLanguagesAndReturnError()
            .map_err(|e| format!("Failed to query supported languages: {e}"))?
    };

    let mut result = Vec::new();
    for lang in &*langs {
        result.push(lang.to_string());
    }
    Ok(result)
}

#[cfg(not(target_os = "macos"))]
pub fn get_supported_languages() -> Result<Vec<String>, String> {
    Ok(vec!["en-US".to_string()])
}

/// Tauri IPC command — runs Vision OCR on a base64-encoded PNG.
/// Accepts optional `languages` parameter: ISO language codes defining
/// preferred recognition languages (e.g. ["zh-Hans", "en-US"]).
/// When empty, automatic language detection is used.
/// Uses spawn_blocking so Vision inference (~100-300ms) does not block
/// the Tokio async runtime.
#[tauri::command]
pub async fn recognize_text(
    image_base64: String,
    languages: Option<Vec<String>>,
) -> Result<String, String> {
    let langs = languages.unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || {
        recognize_text_in_base64_image(&image_base64, &langs)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Tauri IPC command — returns the list of OCR languages supported by the system.
#[tauri::command]
pub async fn get_supported_ocr_languages() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(get_supported_languages)
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_base64_returns_error() {
        assert!(recognize_text_in_base64_image("not-valid-base64!!!", &[]).is_err());
    }

    #[test]
    fn test_non_png_bytes_returns_error() {
        let b64 = general_purpose::STANDARD.encode(b"not a png at all");
        let result = recognize_text_in_base64_image(&b64, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_oversized_input_returns_error() {
        // 67 million + 1 bytes of base64 should be rejected before decoding.
        let huge = "A".repeat(67_000_001);
        let result = recognize_text_in_base64_image(&huge, &[]);
        assert_eq!(result, Err("Image too large for OCR".to_string()));
    }

    #[test]
    fn test_empty_base64_returns_no_text() {
        // Empty string decodes to zero bytes → should return NO_TEXT (not crash).
        let result = recognize_text_in_base64_image("", &[]);
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
            recognize_text_in_base64_image(&b64, &[]),
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
        let result = recognize_text_in_base64_image(&b64, &[]);
        assert!(
            result.is_ok(),
            "Expected OCR to succeed, got: {:?}",
            result
        );
        let text = result.unwrap();
        assert!(!text.is_empty(), "Expected non-empty OCR result");
    }

    /// Test OCR with explicit language list (auto-detect still enabled).
    #[cfg(target_os = "macos")]
    #[test]
    fn test_png_with_text_and_explicit_languages() {
        let png_bytes = include_bytes!("test_fixtures/hello.png");
        let b64 = general_purpose::STANDARD.encode(png_bytes);
        let langs = vec!["en-US".to_string(), "zh-Hans".to_string()];
        let result = recognize_text_in_base64_image(&b64, &langs);
        assert!(
            result.is_ok(),
            "Expected OCR with explicit languages to succeed, got: {:?}",
            result
        );
    }

    /// Test that get_supported_languages returns a non-empty list including English.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_get_supported_languages() {
        let langs = get_supported_languages().expect("Should return supported languages");
        assert!(!langs.is_empty(), "Expected at least one supported language");
        assert!(
            langs.iter().any(|l| l.starts_with("en")),
            "Expected English in supported languages, got: {:?}",
            langs
        );
    }
}
