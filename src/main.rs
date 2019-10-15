extern crate num;

use num::Complex;

/// limit を繰り返し回数の上限として、c がマンデルブロ集合に含まれるかを判定する
/// c がマンデルブロ集合に含まれないならSome(i)を返却する。
/// i は c が原点を中心とする半径2の円から出るまでにかかった繰り返し回数となる。
/// 
/// c がマンデルブロ集合に含まれているらしい場合
///     （正確に言うと繰り返し回数の上限に達しても c がマンデルブロ集合に含まれていることを示せなかった場合)
/// Noneを返却する。
fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex {re: 0.0, im: 0.0};
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i)
        }
    }
    None
}

#[allow(dead_code)]
fn complex_square_add_loop(c: Complex<f64>) {
    let mut z = Complex{re: 0.0, im: 0.0};
    loop {
        z = z * z + c;
    }
}

use std::str::FromStr;

/// 文字列 s は座標のペア。 "400x600" "1.0,0.5"など
/// s は　<LEFT><SEP><RIGHT>の形でなければならない。
/// <SEP>はsepalator引数で与えられる文字で<LEFT>と<RIGHT>はT::from_strでパースできる文字列
/// s が適切な形であればSome(x, y)を返す。
/// パースできなければNoneを返す。
fn parse_pair<T: FromStr> (s: &str, sepalator: char) -> Option<(T, T)> {
    match s.find(sepalator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32> ("", ','),         None);
    assert_eq!(parse_pair::<i32> ("10,", ','),      None);
    assert_eq!(parse_pair::<i32> (",10", ','),      None);
    assert_eq!(parse_pair::<i32> ("10,20", ','),    Some((10, 20)));
    assert_eq!(parse_pair::<i32> ("10,20xy", ','),  None);
    assert_eq!(parse_pair::<f64> ("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64> ("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

/// カンマで分けられた浮動小数点数のペアをパースして複素数を返す。
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex {re, im}),
        None => None
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.25,-0.0625"), Some())
}

/// 出力される画像のピクセルの位置を取り対応する複素数平面上の点を返す。
/// bounds は出力画像の幅と高さをピクセル単位で与える。
/// pixelは画面上の特定のピクセルを(行, 列)ペアの形で指定する。
/// 仮引数upper_left, lower_rightは出力画像に描画する
/// 複素平面を左上と右下で指定する。
fn pixel_to_point(bounds: (usize, usize),
                    pixel: (usize, usize),
                    upper_left: Complex<f64>,
                    lower_right: Complex<f64>) -> Complex<f64> {
    let (width, height) = (lower_right.re - upper_left.re,
                            upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100,100), 
                                (25, 75), 
                                Complex {re: -1.0, im: 1.0}, 
                                Complex {re: 1.0, im: -1.0}), 
                Complex {re: -0.5, im: -0.5});
}

/// 矩形派にのマンデルブロ集合をピクセルのバッファに描画する。
/// 仮引数 boundsはバッファpixelsの幅と高さを指定する。
/// バッファpixelsはピクセルのグレースケールの値をバイトで保持する。
/// upper_leftとlower_rightはピクセルバッファの左上と右下に対応する
/// 複素平面上の点を指定する。
fn render(pixels: &mut [u8],
            bounds: (usize, usize),
            upper_left: Complex<f64>,
            lower_right: Complex<f64>)
{
    assert!(pixels.len() == bounds.0 * bounds.1);
    for row in 0 .. bounds.1 {
        for column in 0 .. bounds.0 {
            let point = pixel_to_point(bounds, (column, row), 
                                        upper_left, lower_right);
            pixels[row * bounds.0 + column] = 
                match escape_time(point, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }
}

extern crate image;

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels, 
                    bounds.0 as u32, bounds.1 as u32, 
                    ColorType::Gray(8))?;
    Ok(())
}

use std::io::Write;
/// main部
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        writeln!(std::io::stderr(),
                "Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT")
            .unwrap();
        writeln!(std::io::stderr(),
                "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
                args[0])
            .unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x')
        .expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3])
        .expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4])
        .expect("error parsing lower  right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    render(&mut pixels, bounds, upper_left, lower_right);

    write_image(&args[1], &pixels, bounds)
        .expect("error writing PNG file");
}