use std::any::Any;
use std::cmp;
use std::collections::HashMap;
use std::str;
use std::sync::Arc;

use itertools;
use data_encoding::{BASE32, BASE64};

use gtmpl_value::{from_value, Value};

use utils;

gtmpl_fn!(
#[doc = r#"Base 64 encode a string."#]
fn base64encode(s: String) -> Result<String, String> {
    Ok(BASE64.encode(&s.into_bytes()))
}
);

gtmpl_fn!(
#[doc = r#"Base 64 decode a string."#]
fn base64decode(s: String) -> Result<String, String> {
    BASE64
        .decode(&s.into_bytes())
        .map_err(|e| format!("unable to decode {}", e))
        .and_then(|v| {
            str::from_utf8(&v)
                .map_err(|e| format!("unable to decode: {}", e))
                .map(|s| s.to_owned())
        })
}
);

gtmpl_fn!(
#[doc = r#"Base 32 encode a string."#]
fn base32encode(s: String) -> Result<String, String> {
    Ok(BASE32.encode(&s.into_bytes()))
}
);

gtmpl_fn!(
#[doc = r#"Base 32 decode a string."#]
fn base32decode(s: String) -> Result<String, String> {
    BASE32
        .decode(&s.into_bytes())
        .map_err(|e| format!("unable to decode {}", e))
        .and_then(|v| {
            str::from_utf8(&v)
                .map_err(|e| format!("unable to decode: {}", e))
                .map(|s| s.to_owned())
        })
}
);

gtmpl_fn!(
#[doc = r#"Truncate a string with ellipses. `abbrev 5 "hello world"` yields "he...""#]
fn abbrev(width: i64, s: String) -> Result<String, String> {
    if width < 4 || s.len() < width as usize {
        Ok(s)
    } else {
        Ok(format!("{}...", &s[..(width as usize - 3)]))
    }
}
);

gtmpl_fn!(
#[doc = r#"Abbreviate from both sides, yielding "...lo wo...""#]
fn abbrevboth(left: i64, right: i64, s: String) -> Result<String, String> {
    let offset = cmp::min(left as usize, s.len());
    let max_width = cmp::min(right as usize, s.len());
    if max_width < 4 || offset > 0 && max_width < 7 || s.len() <= max_width as usize {
        Ok(s)
    } else if offset <= 4 {
        Ok(format!("{}...", &s[..(max_width - 3)]))
    } else if (offset + max_width - 3) < s.len() {
        let len = offset + max_width - 6;
        Ok(format!("...{}...", &s[offset..len]))
    } else {
        let offset = s.len() - (max_width - 3);
        Ok(format!("...{}", &s[offset..]))
    }
}
);

gtmpl_fn!(
#[doc = r#"Given a multi-word string, return the initials. `initials "Matt Butcher"` returns "MB""#]
fn initials(s: String) -> Result<String, String> {
    Ok(
        s.split_whitespace()
            .map(|w| (&w[0..1]).to_owned())
            .collect(),
    )
}
);

gtmpl_fn!(
#[doc = r#"Given a length, generate a random alphanumeric sequence"#]
fn rand_alpha_numeric(count: u64) -> Result<String, String> {
    Ok(utils::random_utf8(count as usize))
}
);

gtmpl_fn!(
#[doc = r#"Given a length, generate an alphabetic string"#]
fn rand_alpha(count: u64) -> Result<String, String> {
    Ok(utils::random_alpha(count as usize))
}
);

gtmpl_fn!(
#[doc = r#"Given a length, generate a random ASCII string (symbols included)"#]
fn rand_ascii(count: u64) -> Result<String, String> {
    Ok(utils::random_ascii(count as usize))
}
);

gtmpl_fn!(
#[doc = r#"Given a length, generate a string of digits."#]
fn rand_numeric(count: u64) -> Result<String, String> {
    Ok(utils::random_numeric(count as usize))
}
);

gtmpl_fn!(
#[doc = r#"Remove title casing"#]
fn untitle(s: String) -> Result<String, String> {
    let mut ws = true;
    Ok(
        s.chars()
            .map(|c| if c.is_whitespace() {
                ws = true;
                c.to_string()
            } else if ws {
                ws = false;
                c.to_lowercase().to_string()
            } else {
                c.to_string()
            })
            .collect::<String>(),
    )
}
);

gtmpl_fn!(
fn replace(old: String, new: String, s: String) -> Result<String, String> {
    Ok(s.replace(&old, &new))
}
);

gtmpl_fn!(
fn plural(one: String, many: String, count: i64) -> Result<String, String> {
    Ok(if count == 1 { one } else { many })
}
);

gtmpl_fn!(
#[doc = r#"Truncate a string (no suffix). `trunc 5 "Hello World"` yields "hello"."#]
fn trunc(len: i64, s: String) -> Result<String, String> {
    if len < 0 || (len as usize) > s.len() {
        Ok(s)
    } else {
        Ok((&s[..(len as usize)]).to_string())
    }
}
);

/// Golang's strings.Join, but as `join SEP SLICE`
pub fn join(args: &[Arc<Any>]) -> Result<Arc<Any>, String> {
    if args.len() != 2 {
        return Err(String::from("two arguments required"));
    }
    let arg0 = args[0].downcast_ref::<Value>().ok_or_else(|| {
        "unable to downcast".to_owned()
    })?;
    let sep: String = from_value(arg0).ok_or_else(
        || "unable to convert from Value".to_owned(),
    )?;
    let arg1 = args[1].downcast_ref::<Value>().ok_or_else(|| {
        "unable to downcast".to_owned()
    })?;
    if let Value::Array(ref list) = *arg1 {
        Ok(Arc::new(Value::from(
            itertools::join(list.iter().map(|v| v.to_string()), &sep),
        )))
    } else {
        return Err(String::from("second argument must be of type Array"));
    }
}

gtmpl_fn!(
#[doc = r#"Golang's strings.Split, but as `split SEP STRING`. The results are returned
             as a map with the indexes set to _N, where N is an integer starting from 0.
             Use it like this: `{{$v := "foo/bar/baz" | split "/"}}{{$v._0}}` (Prints `foo`)"#]
fn split(sep: String, orig: String) -> Result<HashMap<String, String>, String> {

    let m: HashMap<String, String> = orig.split(&sep)
        .enumerate()
        .map(|(i, s)| (format!("_{}", i), s.to_owned()))
        .collect();
    Ok(m)
}
);

gtmpl_fn!(
#[doc = r#"Given string, start, and length, return a substr."#]
fn substring(start: i64, len: i64, s: String) -> Result<String, String> {
    let start = if start < 0 { 0 } else { start as usize };
    let len = if len < 0 { s.len() } else { len as usize };
    if start > len || start > s.len() || len > s.len() {
        Ok(s)
    } else {
        Ok(s[start..len].to_string())
    }
}
);

gtmpl_fn!(
#[doc=r#"Golang's strings.TrimSpace"#]
fn trim(s: String) -> Result<String, String> {
    Ok(s.trim().to_owned())
}
);

gtmpl_fn!(
#[doc=r#"Golang's strings.Trim, but with the argument order reversed
`trimAll "$" "$5.00"` or `"$5.00 | trimAll "$""#]
fn trim_all(substr: String, s: String) -> Result<String, String> {
    let x: &[_] = &substr.chars().collect::<Vec<_>>();
    Ok(s.trim_matches(x).to_owned())
}
);

gtmpl_fn!(
#[doc=r#"Golang's strings.TrimSuffix, but with the argument order reversed:
`trimSuffix "-" "ends-with-"`"#]
fn trim_suffix(substr: String, s: String) -> Result<String, String> {
    Ok(s.trim_right_matches(&substr).to_owned())
}
);

gtmpl_fn!(
#[doc=r#"Golang's strings.TrimPrefix, but with the argument order reversed `trimPrefix "$" "$5"`"#]
fn trim_prefix(substr: String, s: String) -> Result<String, String> {
    Ok(s.trim_left_matches(&substr).to_owned())
}
);

gtmpl_fn!(
#[doc=r#"Golang's strings.Contains, but with the arguments switched: `contains substr str`."#]
fn contains(substr: String, s: String) -> Result<bool, String> {
    Ok(s.contains(&substr))
}
);

gtmpl_fn!(
#[doc=r#"Golang's strings.hasSuffix, but with the arguments switched"#]
fn has_suffix(substr: String, s: String) -> Result<bool, String> {
    Ok(s.ends_with(&substr))
}
);

gtmpl_fn!(
#[doc=r#"Golang's strings.hasPrefix, but with the arguments switched"#]
fn has_prefix(substr: String, s: String) -> Result<bool, String> {
    Ok(s.starts_with(&substr))
}
);

#[cfg(test)]
mod test {
    use super::*;
    use std::any::Any;
    use std::sync::Arc;
    use gtmpl_value::Value;

    macro_rules! varc(
        ($x:expr) => { { let v: Arc<Any> = Arc::new(Value::from($x)); v } }
    );

    macro_rules! vvarc(
        ($($x:expr),*) => { { let v: Vec<Arc<Any>> = vec![$(varc!($x)),*]; v } }
    );

    macro_rules! test_fn_assert(
        ($func:ident, $args:expr, $typ:ident, $ass:ident) => {
            let v = $args;
            let ret = $func(&v).unwrap();
            let ret_ = ret.downcast_ref::<Value>();
            if let Some(&Value::$typ(ref x)) = ret_ {
                return assert!($ass(x));
            }
            assert!(false);
        }
    );

    macro_rules! test_fn(
        ($func:ident, $args:expr, $exp:expr) => {
            let v = $args;
            let ret = $func(&v).unwrap();
            let ret_ = ret.downcast_ref::<Value>();
            let expected = $exp;
            assert_eq!(ret_, Some(&Value::from(expected)));
        }
    );

    #[test]
    fn test_base64encode() {
        test_fn!(base64encode, vvarc!("Hello World!"), "SGVsbG8gV29ybGQh");
    }

    #[test]
    fn test_base64decode() {
        test_fn!(base64decode, vvarc!("SGVsbG8gV29ybGQh"), "Hello World!");
    }

    #[test]
    fn test_base32encode() {
        test_fn!(
            base32encode,
            vvarc!("Hello World!"),
            "JBSWY3DPEBLW64TMMQQQ===="
        );
    }

    #[test]
    fn test_base32decode() {
        test_fn!(
            base32decode,
            vvarc!("JBSWY3DPEBLW64TMMQQQ===="),
            "Hello World!"
        );
    }

    #[test]
    fn test_abbrv() {
        test_fn!(abbrev, vvarc!(4, "foobar"), "f...");
    }

    #[test]
    fn test_abbrvboth() {
        test_fn!(abbrevboth, vvarc!(5, 7, "foobarfoobar"), "...r...");
        test_fn!(abbrevboth, vvarc!(4, 7, "foobarfoobar"), "foob...");
        test_fn!(abbrevboth, vvarc!(6, 9, "foobarfoobar"), "...foobar");
        test_fn!(abbrevboth, vvarc!(5, 7, "foobar"), "foobar");
    }

    #[test]
    fn test_initials() {
        test_fn!(initials, vvarc!(""), "");
        test_fn!(initials, vvarc!(" "), "");
        test_fn!(initials, vvarc!("Foo Bar"), "FB");
    }

    #[test]
    fn test_rand_alpha_numeric() {
        let check = |x: &String| x.chars().count() == 20;
        test_fn_assert!(rand_alpha_numeric, vvarc!(20), String, check);
    }

    #[test]
    fn test_rand_alpha() {
        let check = |x: &String| x.len() == 20;
        test_fn_assert!(rand_alpha, vvarc!(20), String, check);
    }

    #[test]
    fn test_rand_ascii() {
        let check = |x: &String| x.len() == 20;
        test_fn_assert!(rand_ascii, vvarc!(20), String, check);
    }

    #[test]
    fn test_rand_numeric() {
        let check = |x: &String| {
            let i = x.parse::<i64>();
            x.len() == 10 && i.is_ok()
        };
        test_fn_assert!(rand_numeric, vvarc!(10), String, check);
    }

    #[test]
    fn test_untitle() {
        test_fn!(untitle, vvarc!(""), "");
        test_fn!(untitle, vvarc!(" "), " ");
        test_fn!(untitle, vvarc!("Foo Bar"), "foo bar");
        test_fn!(untitle, vvarc!("FOO BAR"), "fOO bAR");
        test_fn!(untitle, vvarc!("  F  B  "), "  f  b  ");
    }

    #[test]
    fn test_replace() {
        test_fn!(replace,
                 vvarc!("World", "Doom", "Hello World!"),
                 "Hello Doom!");
    }

    #[test]
    fn test_plural() {
        test_fn!(plural, vvarc!("mouse", "mice", 1), "mouse");
        test_fn!(plural, vvarc!("mouse", "mice", 10), "mice");
    }

    #[test]
    fn test_trunc() {
        test_fn!(trunc, vvarc!(5, "foobar"), "fooba");
    }

    #[test]
    fn test_join() {
        test_fn!(join, vvarc!("_", vec!["hello", "world"]), "hello_world");
    }

    #[test]
    fn test_split() {
        let mut m = HashMap::new();
        m.insert("_0".to_owned(), "foo".to_owned());
        m.insert("_1".to_owned(), "bar".to_owned());
        test_fn!(split, vvarc!(" ", "foo bar"), m);
    }

    #[test]
    fn test_substring() {
        test_fn!(substring, vvarc!(0, 0, ""), "");
        test_fn!(substring, vvarc!(1, 5, "foobar"), "ooba");
        test_fn!(substring, vvarc!(3, 2, "foobar"), "foobar");
        test_fn!(substring, vvarc!(8, 9, "foobar"), "foobar");
    }

    #[test]
    fn test_contains() {
        test_fn!(contains, vvarc!("oo", "foobar"), true);
    }

    #[test]
    fn test_has_suffix() {
        test_fn!(has_suffix, vvarc!("bar", "foobar"), true);
    }

    #[test]
    fn test_has_prefix() {
        test_fn!(has_prefix, vvarc!("foo", "foobar"), true);
    }

    #[test]
    fn test_trim() {
        test_fn!(trim, vvarc!("  foobar "), "foobar");
    }

    #[test]
    fn test_trim_all() {
        test_fn!(trim_all, vvarc!(" fr", "  foobar "), "ooba");
    }

    #[test]
    fn test_trim_suffix() {
        test_fn!(trim_suffix, vvarc!("bar", "foobar"), "foo");
    }

    #[test]
    fn test_trim_prefix() {
        test_fn!(trim_prefix, vvarc!("foo", "foobar"), "bar");
    }
}
