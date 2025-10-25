use music_station::lyrics::LyricFormat;

#[test]
fn test_plain_text_detection() {
    let plain = "This is plain text\nNo timestamps at all\nJust regular lyrics";
    assert_eq!(LyricFormat::detect_from_content(plain), LyricFormat::Plain);
}

#[test]
fn test_standard_lrc_detection() {
    let lrc = "[00:12.34]This is a line of lyrics\n[00:16.78]Another line follows";
    assert_eq!(LyricFormat::detect_from_content(lrc), LyricFormat::Lrc);
}

#[test]
fn test_extended_lrc_detection() {
    let extended = "[0,11550]Line with no word timing\n[11550,5000]Another line";
    assert_eq!(LyricFormat::detect_from_content(extended), LyricFormat::Lrc);
}

#[test]
fn test_word_level_lrc_detection() {
    let word_lrc = "[0,11550]挪(0,721)威(721,721)的(1442,721)森(2163,721)林(2884,721)";
    assert_eq!(LyricFormat::detect_from_content(word_lrc), LyricFormat::LrcWord);
}

#[test]
fn test_mixed_word_level_detection() {
    let mixed = "[11550,5000]Another(0,500) line(500,500) with(1000,300) words(1300,400)";
    assert_eq!(LyricFormat::detect_from_content(mixed), LyricFormat::LrcWord);
}

#[test]
fn test_format_from_str() {
    assert_eq!(LyricFormat::from_str("plain"), LyricFormat::Plain);
    assert_eq!(LyricFormat::from_str("lrc"), LyricFormat::Lrc);
    assert_eq!(LyricFormat::from_str("lrc_word"), LyricFormat::LrcWord);
    assert_eq!(LyricFormat::from_str("lrcword"), LyricFormat::LrcWord);
    assert_eq!(LyricFormat::from_str("word"), LyricFormat::LrcWord);
    assert_eq!(LyricFormat::from_str("extended"), LyricFormat::LrcWord);
    assert_eq!(LyricFormat::from_str("WORD"), LyricFormat::LrcWord); // case insensitive
}

#[test]
fn test_format_as_str() {
    assert_eq!(LyricFormat::Plain.as_str(), "plain");
    assert_eq!(LyricFormat::Lrc.as_str(), "lrc");
    assert_eq!(LyricFormat::LrcWord.as_str(), "lrc_word");
}

#[test]
fn test_format_serialization() {
    use serde_json;
    
    let plain = LyricFormat::Plain;
    assert_eq!(serde_json::to_string(&plain).unwrap(), "\"plain\"");
    
    let lrc = LyricFormat::Lrc;
    assert_eq!(serde_json::to_string(&lrc).unwrap(), "\"lrc\"");
    
    let word = LyricFormat::LrcWord;
    assert_eq!(serde_json::to_string(&word).unwrap(), "\"lrc_word\"");
}
