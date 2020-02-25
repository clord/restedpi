use lazy_static;

use regex::Regex;

lazy_static! {
    static ref REMOVE: Regex = Regex::new(r#"[^\w\s$*_+~.()'"!\-]"#).unwrap();
    static ref SPLIT: Regex = Regex::new(r#"(?P<token>\S+)\s*"#).unwrap();
}

// Convert arbitrary human name for something into a slug for url purposes
pub fn slugify(name: &str, inc: usize) -> String {
    if name.trim().len() == 0 {
        return format!("{}", inc);
    }
    let mut replaced = String::new();
    for c in name.trim().chars() {
        replaced.push_str(&replace_char(c))
    }

    let simplified = REMOVE.replace_all(&replaced, " ").to_lowercase();
    let mut result: Vec<String> = Vec::new();
    for cap in SPLIT.captures_iter(&simplified) {
        let mut chunk = String::new();
        for c in cap["token"].chars() {
            chunk.push_str(&replace_char(c));
        }
        result.push(chunk);
    }
    if inc != 0 {
        result.push(format!("{}", inc));
    }
    return result.join("-");
}

// Make some common unicode characters into more usable slug ascii
fn replace_char(c: char) -> String {
    match c {
        '$' => " dollar ".to_string(),
        '#' => " number ".to_string(),
        '%' => " percent ".to_string(),
        '&' => " and ".to_string(),
        '<' => " less ".to_string(),
        '@' => " at ".to_string(),
        ':' => " colon ".to_string(),
        '>' => " greater ".to_string(),
        '|' => " or ".to_string(),
        '¢' => " cent ".to_string(),
        '£' => " pound ".to_string(),
        '¤' => " currency ".to_string(),
        '¥' => " yen ".to_string(),
        '©' => "(c)".to_string(),
        'ª' => "a".to_string(),
        '®' => "(r)".to_string(),
        'º' => "o".to_string(),
        'À' => "A".to_string(),
        'Á' => "A".to_string(),
        'Â' => "A".to_string(),
        'Ã' => "A".to_string(),
        'Ä' => "A".to_string(),
        'Å' => "A".to_string(),
        'Æ' => "AE".to_string(),
        'Ç' => "C".to_string(),
        'È' => "E".to_string(),
        'É' => "E".to_string(),
        'Ê' => "E".to_string(),
        'Ë' => "E".to_string(),
        'Ì' => "I".to_string(),
        'Í' => "I".to_string(),
        'Î' => "I".to_string(),
        'Ï' => "I".to_string(),
        'Ð' => "D".to_string(),
        'Ñ' => "N".to_string(),
        'Ò' => "O".to_string(),
        'Ó' => "O".to_string(),
        'Ô' => "O".to_string(),
        'Õ' => "O".to_string(),
        'Ö' => "O".to_string(),
        'Ø' => "O".to_string(),
        'Ù' => "U".to_string(),
        'Ú' => "U".to_string(),
        'Û' => "U".to_string(),
        'Ü' => "U".to_string(),
        'Ý' => "Y".to_string(),
        'Þ' => "TH".to_string(),
        'ß' => "ss".to_string(),
        'à' => "a".to_string(),
        'á' => "a".to_string(),
        'â' => "a".to_string(),
        'ã' => "a".to_string(),
        'ä' => "a".to_string(),
        'å' => "a".to_string(),
        'æ' => "ae".to_string(),
        'ç' => "c".to_string(),
        'è' => "e".to_string(),
        'é' => "e".to_string(),
        'ê' => "e".to_string(),
        'ë' => "e".to_string(),
        'ì' => "i".to_string(),
        'í' => "i".to_string(),
        'î' => "i".to_string(),
        'ï' => "i".to_string(),
        'ð' => "d".to_string(),
        'ñ' => "n".to_string(),
        'ò' => "o".to_string(),
        'ó' => "o".to_string(),
        'ô' => "o".to_string(),
        'õ' => "o".to_string(),
        'ö' => "o".to_string(),
        'ø' => "o".to_string(),
        'ù' => "u".to_string(),
        'ú' => "u".to_string(),
        'û' => "u".to_string(),
        'ü' => "u".to_string(),
        'ý' => "y".to_string(),
        'þ' => "th".to_string(),
        'ÿ' => "y".to_string(),
        'Ā' => "A".to_string(),
        'ā' => "a".to_string(),
        'Ă' => "A".to_string(),
        'ă' => "a".to_string(),
        'Ą' => "A".to_string(),
        'ą' => "a".to_string(),
        'Ć' => "C".to_string(),
        'ć' => "c".to_string(),
        'Č' => "C".to_string(),
        'č' => "c".to_string(),
        'Ď' => "D".to_string(),
        'ď' => "d".to_string(),
        'Đ' => "DJ".to_string(),
        'đ' => "dj".to_string(),
        'Ē' => "E".to_string(),
        'ē' => "e".to_string(),
        'Ė' => "E".to_string(),
        'ė' => "e".to_string(),
        'Ę' => "e".to_string(),
        'ę' => "e".to_string(),
        'Ě' => "E".to_string(),
        'ě' => "e".to_string(),
        'Ğ' => "G".to_string(),
        'ğ' => "g".to_string(),
        'Ģ' => "G".to_string(),
        'ģ' => "g".to_string(),
        'Ĩ' => "I".to_string(),
        'ĩ' => "i".to_string(),
        'Ī' => "i".to_string(),
        'ī' => "i".to_string(),
        'Į' => "I".to_string(),
        'į' => "i".to_string(),
        'İ' => "I".to_string(),
        'ı' => "i".to_string(),
        'Ķ' => "k".to_string(),
        'ķ' => "k".to_string(),
        'Ļ' => "L".to_string(),
        'ļ' => "l".to_string(),
        'Ľ' => "L".to_string(),
        'ľ' => "l".to_string(),
        'Ł' => "L".to_string(),
        'ł' => "l".to_string(),
        'Ń' => "N".to_string(),
        'ń' => "n".to_string(),
        'Ņ' => "N".to_string(),
        'ņ' => "n".to_string(),
        'Ň' => "N".to_string(),
        'ň' => "n".to_string(),
        'Ő' => "O".to_string(),
        'ő' => "o".to_string(),
        'Œ' => "OE".to_string(),
        'œ' => "oe".to_string(),
        'Ŕ' => "R".to_string(),
        'ŕ' => "r".to_string(),
        'Ř' => "R".to_string(),
        'ř' => "r".to_string(),
        'Ś' => "S".to_string(),
        'ś' => "s".to_string(),
        'Ş' => "S".to_string(),
        'ş' => "s".to_string(),
        'Š' => "S".to_string(),
        'š' => "s".to_string(),
        'Ţ' => "T".to_string(),
        'ţ' => "t".to_string(),
        'Ť' => "T".to_string(),
        'ť' => "t".to_string(),
        'Ũ' => "U".to_string(),
        'ũ' => "u".to_string(),
        'Ū' => "u".to_string(),
        'ū' => "u".to_string(),
        'Ů' => "U".to_string(),
        'ů' => "u".to_string(),
        'Ű' => "U".to_string(),
        'ű' => "u".to_string(),
        'Ų' => "U".to_string(),
        'ų' => "u".to_string(),
        'Ŵ' => "W".to_string(),
        'ŵ' => "w".to_string(),
        'Ŷ' => "Y".to_string(),
        'ŷ' => "y".to_string(),
        'Ÿ' => "Y".to_string(),
        'Ź' => "Z".to_string(),
        'ź' => "z".to_string(),
        'Ż' => "Z".to_string(),
        'ż' => "z".to_string(),
        'Ž' => "Z".to_string(),
        'ž' => "z".to_string(),
        'ƒ' => "f".to_string(),
        'Ơ' => "O".to_string(),
        'ơ' => "o".to_string(),
        'Ư' => "U".to_string(),
        'ư' => "u".to_string(),
        'ǈ' => "LJ".to_string(),
        'ǉ' => "lj".to_string(),
        'ǋ' => "NJ".to_string(),
        'ǌ' => "nj".to_string(),
        'Ș' => "S".to_string(),
        'ș' => "s".to_string(),
        'Ț' => "T".to_string(),
        'ț' => "t".to_string(),
        '˚' => "o".to_string(),
        'Ά' => "A".to_string(),
        'Έ' => "E".to_string(),
        'Ή' => "H".to_string(),
        'Ί' => "I".to_string(),
        'Ό' => "O".to_string(),
        'Ύ' => "Y".to_string(),
        'Ώ' => "W".to_string(),
        'ΐ' => "i".to_string(),
        'Α' => "A".to_string(),
        'Β' => "B".to_string(),
        'Γ' => "G".to_string(),
        'Δ' => "D".to_string(),
        'Ε' => "E".to_string(),
        'Ζ' => "Z".to_string(),
        'Η' => "H".to_string(),
        'Θ' => "8".to_string(),
        'Ι' => "I".to_string(),
        'Κ' => "K".to_string(),
        'Λ' => "L".to_string(),
        'Μ' => "M".to_string(),
        'Ν' => "N".to_string(),
        'Ξ' => "3".to_string(),
        'Ο' => "O".to_string(),
        'Π' => "P".to_string(),
        'Ρ' => "R".to_string(),
        'Σ' => "S".to_string(),
        'Τ' => "T".to_string(),
        'Υ' => "Y".to_string(),
        'Φ' => "F".to_string(),
        'Χ' => "X".to_string(),
        'Ψ' => "PS".to_string(),
        'Ω' => "W".to_string(),
        'Ϊ' => "I".to_string(),
        'Ϋ' => "Y".to_string(),
        'ά' => "a".to_string(),
        'έ' => "e".to_string(),
        'ή' => "h".to_string(),
        'ί' => "i".to_string(),
        'ΰ' => "y".to_string(),
        'α' => "a".to_string(),
        'β' => "b".to_string(),
        'γ' => "g".to_string(),
        'δ' => "d".to_string(),
        'ε' => "e".to_string(),
        'ζ' => "z".to_string(),
        'η' => "h".to_string(),
        'θ' => "8".to_string(),
        'ι' => "i".to_string(),
        'κ' => "k".to_string(),
        'λ' => "l".to_string(),
        'μ' => "m".to_string(),
        'ν' => "n".to_string(),
        'ξ' => "3".to_string(),
        'ο' => "o".to_string(),
        'π' => "p".to_string(),
        'ρ' => "r".to_string(),
        'ς' => "s".to_string(),
        'σ' => "s".to_string(),
        'τ' => "t".to_string(),
        'υ' => "y".to_string(),
        'φ' => "f".to_string(),
        'χ' => "x".to_string(),
        'ψ' => "ps".to_string(),
        'ω' => "w".to_string(),
        'ϊ' => "i".to_string(),
        'ϋ' => "y".to_string(),
        'ό' => "o".to_string(),
        'ύ' => "y".to_string(),
        'ώ' => "w".to_string(),
        'Ё' => "Yo".to_string(),
        'Ђ' => "DJ".to_string(),
        'Є' => "Ye".to_string(),
        'І' => "I".to_string(),
        'Ї' => "Yi".to_string(),
        'Ј' => "J".to_string(),
        'Љ' => "LJ".to_string(),
        'Њ' => "NJ".to_string(),
        'Ћ' => "C".to_string(),
        'Џ' => "DZ".to_string(),
        'А' => "A".to_string(),
        'Б' => "B".to_string(),
        'В' => "V".to_string(),
        'Г' => "G".to_string(),
        'Д' => "D".to_string(),
        'Е' => "E".to_string(),
        'Ж' => "Zh".to_string(),
        'З' => "Z".to_string(),
        'И' => "I".to_string(),
        'Й' => "J".to_string(),
        'К' => "K".to_string(),
        'Л' => "L".to_string(),
        'М' => "M".to_string(),
        'Н' => "N".to_string(),
        'О' => "O".to_string(),
        'П' => "P".to_string(),
        'Р' => "R".to_string(),
        'С' => "S".to_string(),
        'Т' => "T".to_string(),
        'У' => "U".to_string(),
        'Ф' => "F".to_string(),
        'Х' => "H".to_string(),
        'Ц' => "C".to_string(),
        'Ч' => "Ch".to_string(),
        'Ш' => "Sh".to_string(),
        'Щ' => "Sh".to_string(),
        'Ъ' => "U".to_string(),
        'Ы' => "Y".to_string(),
        'Ь' => "".to_string(),
        'Э' => "E".to_string(),
        'Ю' => "Yu".to_string(),
        'Я' => "Ya".to_string(),
        'а' => "a".to_string(),
        'б' => "b".to_string(),
        'в' => "v".to_string(),
        'г' => "g".to_string(),
        'д' => "d".to_string(),
        'е' => "e".to_string(),
        'ж' => "zh".to_string(),
        'з' => "z".to_string(),
        'и' => "i".to_string(),
        'й' => "j".to_string(),
        'к' => "k".to_string(),
        'л' => "l".to_string(),
        'м' => "m".to_string(),
        'н' => "n".to_string(),
        'о' => "o".to_string(),
        'п' => "p".to_string(),
        'р' => "r".to_string(),
        'с' => "s".to_string(),
        'т' => "t".to_string(),
        'у' => "u".to_string(),
        'ф' => "f".to_string(),
        'х' => "h".to_string(),
        'ц' => "c".to_string(),
        'ч' => "ch".to_string(),
        'ш' => "sh".to_string(),
        'щ' => "sh".to_string(),
        'ъ' => "u".to_string(),
        'ы' => "y".to_string(),
        'ь' => "".to_string(),
        'э' => "e".to_string(),
        'ю' => "yu".to_string(),
        'я' => "ya".to_string(),
        'ё' => "yo".to_string(),
        'ђ' => "dj".to_string(),
        'є' => "ye".to_string(),
        'і' => "i".to_string(),
        'ї' => "yi".to_string(),
        'ј' => "j".to_string(),
        'љ' => "lj".to_string(),
        'њ' => "nj".to_string(),
        'ћ' => "c".to_string(),
        'ѝ' => "u".to_string(),
        'џ' => "dz".to_string(),
        'Ґ' => "G".to_string(),
        'ґ' => "g".to_string(),
        'Ғ' => "GH".to_string(),
        'ғ' => "gh".to_string(),
        'Қ' => "KH".to_string(),
        'қ' => "kh".to_string(),
        'Ң' => "NG".to_string(),
        'ң' => "ng".to_string(),
        'Ү' => "UE".to_string(),
        'ү' => "ue".to_string(),
        'Ұ' => "U".to_string(),
        'ұ' => "u".to_string(),
        'Һ' => "H".to_string(),
        'һ' => "h".to_string(),
        'Ә' => "AE".to_string(),
        'ә' => "ae".to_string(),
        'Ө' => "OE".to_string(),
        'ө' => "oe".to_string(),
        '฿' => "baht".to_string(),
        'ა' => "a".to_string(),
        'ბ' => "b".to_string(),
        'გ' => "g".to_string(),
        'დ' => "d".to_string(),
        'ე' => "e".to_string(),
        'ვ' => "v".to_string(),
        'ზ' => "z".to_string(),
        'თ' => "t".to_string(),
        'ი' => "i".to_string(),
        'კ' => "k".to_string(),
        'ლ' => "l".to_string(),
        'მ' => "m".to_string(),
        'ნ' => "n".to_string(),
        'ო' => "o".to_string(),
        'პ' => "p".to_string(),
        'ჟ' => "zh".to_string(),
        'რ' => "r".to_string(),
        'ს' => "s".to_string(),
        'ტ' => "t".to_string(),
        'უ' => "u".to_string(),
        'ფ' => "f".to_string(),
        'ქ' => "k".to_string(),
        'ღ' => "gh".to_string(),
        'ყ' => "q".to_string(),
        'შ' => "sh".to_string(),
        'ჩ' => "ch".to_string(),
        'ც' => "ts".to_string(),
        'ძ' => "dz".to_string(),
        'წ' => "ts".to_string(),
        'ჭ' => "ch".to_string(),
        'ხ' => "kh".to_string(),
        'ჯ' => "j".to_string(),
        'ჰ' => "h".to_string(),
        'Ẁ' => "W".to_string(),
        'ẁ' => "w".to_string(),
        'Ẃ' => "W".to_string(),
        'ẃ' => "w".to_string(),
        'Ẅ' => "W".to_string(),
        'ẅ' => "w".to_string(),
        'ẞ' => "SS".to_string(),
        'Ạ' => "A".to_string(),
        'ạ' => "a".to_string(),
        'Ả' => "A".to_string(),
        'ả' => "a".to_string(),
        'Ấ' => "A".to_string(),
        'ấ' => "a".to_string(),
        'Ầ' => "A".to_string(),
        'ầ' => "a".to_string(),
        'Ẩ' => "A".to_string(),
        'ẩ' => "a".to_string(),
        'Ẫ' => "A".to_string(),
        'ẫ' => "a".to_string(),
        'Ậ' => "A".to_string(),
        'ậ' => "a".to_string(),
        'Ắ' => "A".to_string(),
        'ắ' => "a".to_string(),
        'Ằ' => "A".to_string(),
        'ằ' => "a".to_string(),
        'Ẳ' => "A".to_string(),
        'ẳ' => "a".to_string(),
        'Ẵ' => "A".to_string(),
        'ẵ' => "a".to_string(),
        'Ặ' => "A".to_string(),
        'ặ' => "a".to_string(),
        'Ẹ' => "E".to_string(),
        'ẹ' => "e".to_string(),
        'Ẻ' => "E".to_string(),
        'ẻ' => "e".to_string(),
        'Ẽ' => "E".to_string(),
        'ẽ' => "e".to_string(),
        'Ế' => "E".to_string(),
        'ế' => "e".to_string(),
        'Ề' => "E".to_string(),
        'ề' => "e".to_string(),
        'Ể' => "E".to_string(),
        'ể' => "e".to_string(),
        'Ễ' => "E".to_string(),
        'ễ' => "e".to_string(),
        'Ệ' => "E".to_string(),
        'ệ' => "e".to_string(),
        'Ỉ' => "I".to_string(),
        'ỉ' => "i".to_string(),
        'Ị' => "I".to_string(),
        'ị' => "i".to_string(),
        'Ọ' => "O".to_string(),
        'ọ' => "o".to_string(),
        'Ỏ' => "O".to_string(),
        'ỏ' => "o".to_string(),
        'Ố' => "O".to_string(),
        'ố' => "o".to_string(),
        'Ồ' => "O".to_string(),
        'ồ' => "o".to_string(),
        'Ổ' => "O".to_string(),
        'ổ' => "o".to_string(),
        'Ỗ' => "O".to_string(),
        'ỗ' => "o".to_string(),
        'Ộ' => "O".to_string(),
        'ộ' => "o".to_string(),
        'Ớ' => "O".to_string(),
        'ớ' => "o".to_string(),
        'Ờ' => "O".to_string(),
        'ờ' => "o".to_string(),
        'Ở' => "O".to_string(),
        'ở' => "o".to_string(),
        'Ỡ' => "O".to_string(),
        'ỡ' => "o".to_string(),
        'Ợ' => "O".to_string(),
        'ợ' => "o".to_string(),
        'Ụ' => "U".to_string(),
        'ụ' => "u".to_string(),
        'Ủ' => "U".to_string(),
        'ủ' => "u".to_string(),
        'Ứ' => "U".to_string(),
        'ứ' => "u".to_string(),
        'Ừ' => "U".to_string(),
        'ừ' => "u".to_string(),
        'Ử' => "U".to_string(),
        'ử' => "u".to_string(),
        'Ữ' => "U".to_string(),
        'ữ' => "u".to_string(),
        'Ự' => "U".to_string(),
        'ự' => "u".to_string(),
        'Ỳ' => "Y".to_string(),
        'ỳ' => "y".to_string(),
        'Ỵ' => "Y".to_string(),
        'ỵ' => "y".to_string(),
        'Ỷ' => "Y".to_string(),
        'ỷ' => "y".to_string(),
        'Ỹ' => "Y".to_string(),
        'ỹ' => "y".to_string(),
        '‘' => "\"".to_string(),
        '’' => "\"".to_string(),
        '“' => "\"".to_string(),
        '”' => "\"".to_string(),
        '†' => "+".to_string(),
        '•' => "*".to_string(),
        '…' => "...".to_string(),
        '₠' => " ecu ".to_string(),
        '₢' => " cruzeiro ".to_string(),
        '₣' => " french franc ".to_string(),
        '₤' => " lira ".to_string(),
        '₥' => " mill ".to_string(),
        '₦' => " naira ".to_string(),
        '₧' => " peseta ".to_string(),
        '₨' => " rupee ".to_string(),
        '₩' => " won ".to_string(),
        '₪' => " new shequel ".to_string(),
        '₫' => " dong ".to_string(),
        '€' => " euro ".to_string(),
        '₭' => " kip ".to_string(),
        '₮' => " tugrik ".to_string(),
        '₯' => " drachma ".to_string(),
        '₰' => " penny ".to_string(),
        '₱' => " peso ".to_string(),
        '₲' => " guarani ".to_string(),
        '₳' => " austral ".to_string(),
        '₴' => " hryvnia ".to_string(),
        '₵' => " cedi ".to_string(),
        '₸' => " kazakhstani tenge ".to_string(),
        '₹' => " indian rupee ".to_string(),
        '₽' => " russian ruble ".to_string(),
        '₿' => " bitcoin ".to_string(),
        '℠' => " sm ".to_string(),
        '™' => " tm ".to_string(),
        '∂' => " d ".to_string(),
        '∆' => " delta ".to_string(),
        '∑' => " sum ".to_string(),
        '∞' => " infinity ".to_string(),
        '♥' => " love ".to_string(),
        '元' => " yuan ".to_string(),
        '円' => " yen ".to_string(),
        '﷼' => " rial ".to_string(),
        x => x.to_string(),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use crate::webapp::slugify::slugify;
    #[test]
    fn basic() {
        assert_eq!(slugify("hello world", 0), "hello-world");
        assert_eq!(slugify("hello world", 1), "hello-world-1");
        assert_eq!(slugify("hello world", 2), "hello-world-2");
        assert_eq!(slugify("hello world", 3), "hello-world-3");
        assert_eq!(slugify("hello WOrLd", 0), "hello-world");
        assert_eq!(slugify("Hello world", 0), "hello-world");
        assert_eq!(slugify("hello worlD", 0), "hello-world");
        assert_eq!(slugify("hELLo world", 0), "hello-world");
    }

    #[test]
    fn test_email() {
        assert_eq!(slugify("alice@bob.com", 0), "alice-at-bob.com");
        assert_eq!(slugify("alice@bob.fo.bar", 1), "alice-at-bob.fo.bar-1");
    }

    #[test]
    fn test_starts_with_number() {
        assert_eq!(
            slugify("10 amazing facts, #10 will shock you", 0),
            "10-amazing-facts-number-10-will-shock-you"
        );
    }

    #[test]
    fn test_contains_numbers() {
        assert_eq!(slugify("The 101 Dalmatians", 0), "the-101-dalmatians");
    }

    #[test]
    fn test_ends_with_number() {
        assert_eq!(
            slugify("lucky number 7 to win $500", 0),
            "lucky-number-7-to-win-dollar-500"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(slugify("", 0), "0");
        assert_eq!(slugify("", 1), "1");
        assert_eq!(slugify("", 2), "2");
        assert_eq!(slugify("", 3), "3");
    }
    #[test]
    fn test_numbers_only() {
        assert_eq!(slugify("101", 0), "101");
    }

    #[test]
    fn test_numbers_and_symbols() {
        assert_eq!(
            slugify("100% Is All You can do & Find #1", 0),
            "100-percent-is-all-you-can-do-and-find-number-1"
        );
    }

    #[test]
    fn test_separator() {
        assert_eq!(slugify("how much is ¥300?", 0), "how-much-is-yen-300");
    }

    #[test]
    fn test_cyrillic_text() {
        assert_eq!(slugify("Компьютер", 0), "kompyuter");
    }
}
